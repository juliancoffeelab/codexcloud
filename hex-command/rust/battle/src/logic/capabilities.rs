//! Battle selects contextual effect occurrences; Catalogue alone combines their values.
use super::{BattlePosition, effect_index::EffectKind};
use crate::data::*;

impl BattlePosition<'_> {
    pub(crate) fn effect_applies(
        &self,
        scope: &EffectScope,
        provider: &Unit,
        target: &Unit,
        distance: u16,
    ) -> bool {
        if distance > scope.radius || (provider.id == target.id && !scope.include_self) {
            return false;
        }
        match scope.targets {
            HealingTargets::Everyone => true,
            HealingTargets::OwnFaction => self.unit_faction(provider) == self.unit_faction(target),
            HealingTargets::Allies => self.allied(provider, target),
            HealingTargets::Enemies => self.hostile(provider, target),
        }
    }
    fn tile_effect_applies(
        &self,
        scope: &EffectScope,
        origin: usize,
        target: &Unit,
        destination: usize,
    ) -> bool {
        if self.board.distance(origin, destination) > scope.radius {
            return false;
        }
        let capturing = destination != target.tile
            && origin == destination
            && self.terrain_definition(origin).capturable()
            && self.captures(origin, self.unit_faction(target));
        let owner = if capturing {
            Some(self.unit_faction(target))
        } else {
            self.owners[origin].as_deref()
        };
        match scope.targets {
            HealingTargets::Everyone => true,
            HealingTargets::OwnFaction => owner == Some(self.unit_faction(target)),
            HealingTargets::Allies => owner.is_some_and(|owner| {
                self.faction_relation(owner, self.unit_faction(target)) == Relation::Allied
            }),
            HealingTargets::Enemies => owner.is_some_and(|owner| {
                self.faction_relation(owner, self.unit_faction(target)) == Relation::Hostile
            }),
        }
    }
    /// Callers provide their permitted source observations and any hypothetical provider overrides.
    /// The callback cannot retain a capability borrowing the temporary input selections.
    pub(crate) fn with_capabilities_at<R>(
        &self,
        target: &Unit,
        destination: usize,
        providers: &[Unit],
        projected: &[(Unit, usize)],
        query: impl FnOnce(&UnitCapabilities<'_>) -> R,
    ) -> R {
        self.with_selected_capabilities_at(
            target,
            destination,
            providers,
            projected,
            |_, _, _| true,
            query,
        )
    }

    /// Source selection also supports diagnostics restricted to an emitter subset.
    pub(crate) fn with_selected_capabilities_at<R>(
        &self,
        target: &Unit,
        destination: usize,
        providers: &[Unit],
        projected: &[(Unit, usize)],
        select: impl FnMut(Option<&Unit>, usize, EffectHandle<'_>) -> bool,
        query: impl FnOnce(&UnitCapabilities<'_>) -> R,
    ) -> R {
        self.with_capability_domain_at(
            target,
            destination,
            providers,
            projected,
            None,
            true,
            select,
            query,
        )
    }

    pub(crate) fn with_domain_capabilities_at<R>(
        &self,
        target: &Unit,
        destination: usize,
        providers: &[Unit],
        projected: &[(Unit, usize)],
        kind: EffectKind,
        query: impl FnOnce(&UnitCapabilities<'_>) -> R,
    ) -> R {
        self.with_capability_domain_at(
            target,
            destination,
            providers,
            projected,
            Some(kind),
            true,
            |_, _, _| true,
            query,
        )
    }

    fn with_capability_domain_at<R>(
        &self,
        target: &Unit,
        destination: usize,
        providers: &[Unit],
        projected: &[(Unit, usize)],
        domain: Option<EffectKind>,
        include_terrain: bool,
        select: impl FnMut(Option<&Unit>, usize, EffectHandle<'_>) -> bool,
        query: impl FnOnce(&UnitCapabilities<'_>) -> R,
    ) -> R {
        self.with_effect_selection_at(
            target,
            destination,
            providers,
            projected,
            domain,
            include_terrain,
            select,
            |effects| {
                let owned = self.definitions.owned(target, &self.runtime.inventories);
                let perception: Vec<_> = self.unit_perception[target.perception.0]
                    .effects
                    .iter()
                    .collect();
                let capabilities = self.definitions.unit(target.kind).compose(UnitInputs {
                    vision_group_sight: &self.vision_groups[target.vision_group.0].sight,
                    equipment: &owned.equipment,
                    gained_abilities: &[],
                    selected_effects: effects,
                    map_modifiers: &self.map_modifiers,
                    perception_contributions: &perception,
                });
                query(&capabilities)
            },
        )
    }
    pub(crate) fn with_selected_domain_effects_at<R>(
        &self,
        target: &Unit,
        destination: usize,
        providers: &[Unit],
        projected: &[(Unit, usize)],
        kind: EffectKind,
        include_terrain: bool,
        select: impl FnMut(Option<&Unit>, usize, EffectHandle<'_>) -> bool,
        query: impl FnOnce(SelectedEffects<'_>) -> R,
    ) -> R {
        self.with_effect_selection_at(
            target,
            destination,
            providers,
            projected,
            Some(kind),
            include_terrain,
            select,
            |effects| query(SelectedEffects::new(effects)),
        )
    }
    /// Terrain-only incoming support does not depend on a unit loadout.
    pub(crate) fn terrain_healing_at(&self, target: &Unit, destination: usize) -> i32 {
        let effects: Vec<_> = self.effect_sources.terrain[EffectKind::Healing as usize]
            .iter()
            .flat_map(|&origin| {
                self.terrain_definition(origin)
                    .effects()
                    .chain(
                        self.tile_effects[origin]
                            .iter()
                            .map(EffectHandle::from_value),
                    )
                    .filter(move |effect| {
                        matches!(effect.value(), EffectValue::Healing { .. })
                            && self.tile_effect_applies(effect.scope(), origin, target, destination)
                    })
            })
            .collect();
        SelectedEffects::new(&effects).healing()
    }

    /// Single-provider diagnostics/forecasts select only that provider's emissions.
    /// There is no recipient composition or scan of unrelated providers here.
    pub(crate) fn with_provider_effects_at<R>(
        &self,
        provider: &Unit,
        target: &Unit,
        origin: usize,
        kind: EffectKind,
        query: impl FnOnce(SelectedEffects<'_>) -> R,
    ) -> R {
        let destination = if provider.id == target.id {
            origin
        } else {
            target.tile
        };
        let distance = self.board.distance(origin, destination);
        let owned = self.definitions.owned(provider, &self.runtime.inventories);
        let mut eligible = owned.effects.iter().copied().filter(|effect| {
            matches!(
                (kind, effect.value()),
                (EffectKind::Healing, EffectValue::Healing { .. })
                    | (EffectKind::Thorns, EffectValue::Thorns { .. })
            ) && self.effect_applies(effect.scope(), provider, target, distance)
        });
        let Some(first) = eligible.next() else {
            return query(SelectedEffects::new(&[]));
        };
        let Some(second) = eligible.next() else {
            return query(SelectedEffects::new(std::slice::from_ref(&first)));
        };
        let effects: Vec<_> = [first, second].into_iter().chain(eligible).collect();
        query(SelectedEffects::new(&effects))
    }

    fn with_effect_selection_at<R>(
        &self,
        target: &Unit,
        destination: usize,
        providers: &[Unit],
        projected: &[(Unit, usize)],
        domain: Option<EffectKind>,
        include_terrain: bool,
        mut select: impl FnMut(Option<&Unit>, usize, EffectHandle<'_>) -> bool,
        query: impl FnOnce(&[EffectHandle<'_>]) -> R,
    ) -> R {
        let relevant = |effect: EffectHandle<'_>| match domain {
            None => true,
            Some(EffectKind::Sight) => matches!(
                effect.value(),
                EffectValue::Sight { .. } | EffectValue::StatModifiers { .. }
            ),
            Some(EffectKind::Concealment) => {
                matches!(effect.value(), EffectValue::Concealment { .. })
            }
            Some(EffectKind::Stats) => matches!(
                effect.value(),
                EffectValue::StatModifiers { .. } | EffectValue::Thorns { .. }
            ),
            Some(EffectKind::Thorns) => matches!(effect.value(), EffectValue::Thorns { .. }),
            Some(EffectKind::Healing) => matches!(effect.value(), EffectValue::Healing { .. }),
            _ => unreachable!("selected capability domain"),
        };
        let mask = domain.map(|kind| {
            (1u8 << kind as usize)
                | if matches!(kind, EffectKind::Stats) {
                    1u8 << EffectKind::Thorns as usize
                } else {
                    0
                }
        });
        let live_providers = std::ptr::eq(providers, self.units.as_slice());
        let skip_live =
            live_providers && mask.is_some_and(|mask| self.effect_sources.unit_mask & mask == 0);
        let mut effects = Vec::new();
        let mut visit = |provider: &Unit, origin| {
            let owned = self.definitions.owned(provider, &self.runtime.inventories);
            for &effect in &owned.effects {
                if relevant(effect)
                    && self.effect_applies(
                        effect.scope(),
                        provider,
                        target,
                        self.board.distance(origin, destination),
                    )
                    && select(Some(provider), origin, effect)
                {
                    effects.push(effect);
                }
            }
        };
        if !skip_live {
            for provider in providers {
                if provider.id == target.id || projected.iter().any(|(p, _)| p.id == provider.id) {
                    continue;
                }
                let indexed = self
                    .effect_sources
                    .unit_domains
                    .get(&provider.id)
                    .is_some_and(|bits| mask.is_none_or(|mask| bits & mask != 0));
                // The live slice cannot contain a changed hypothetical loadout.
                // Caller-supplied observations may, so only those need this test.
                if !indexed
                    && (live_providers
                        || self.index(provider.id).is_some_and(|index| {
                            let live = &self.runtime.units[index];
                            live.kind == provider.kind && live.inventory == provider.inventory
                        }))
                {
                    continue;
                }
                visit(provider, provider.tile);
            }
        }
        // Overrides are already explicit; probing the live roster again only
        // to discover a different kind adds no eligibility information.
        for (provider, _) in projected {
            if provider.id != target.id {
                let origin = projected
                    .iter()
                    .find(|(p, _)| p.id == provider.id)
                    .expect("projected source")
                    .1;
                visit(provider, origin);
            }
        }
        visit(target, destination);
        let terrain = if include_terrain {
            self.effect_sources.all_terrain.as_slice()
        } else {
            &[]
        };
        for &origin in terrain {
            if mask.is_some_and(|mask| self.effect_sources.terrain_domains[origin] & mask == 0) {
                continue;
            }
            for effect in self.terrain_definition(origin).effects().chain(
                self.tile_effects[origin]
                    .iter()
                    .map(EffectHandle::from_value),
            ) {
                if relevant(effect)
                    && self.tile_effect_applies(effect.scope(), origin, target, destination)
                    && select(None, origin, effect)
                {
                    effects.push(effect);
                }
            }
        }
        query(&effects)
    }
}
