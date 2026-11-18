//! Observer geometry and knowledge over Catalogue-composed unit capabilities.
use super::{
    BattlePosition,
    effect_index::{EffectKind, SightKey, Snapshot},
};
use crate::data::*;

pub(crate) struct Perception<'a, 'catalog> {
    game: &'a BattlePosition<'catalog>,
    positions: [Option<(u32, usize)>; 3],
    projected_units: [Option<Unit>; 3],
    reveals: [Option<(u32, VisionGroupKey)>; 2],
}
impl<'a, 'catalog> Perception<'a, 'catalog> {
    pub(crate) fn new(game: &'a BattlePosition<'catalog>, position: Option<(u32, usize)>) -> Self {
        Self {
            game,
            positions: [position, None, None],
            projected_units: [None, None, None],
            reveals: [None, None],
        }
    }
    pub(crate) fn with_position(mut self, id: u32, tile: usize) -> Self {
        let slot = self
            .positions
            .iter()
            .position(|p| p.is_some_and(|(unit, _)| unit == id))
            .or_else(|| self.positions.iter().position(Option::is_none))
            .expect("three projected positions");
        self.positions[slot] = Some((id, tile));
        self
    }
    pub(crate) fn with_unit(mut self, unit: &Unit, tile: usize) -> Self {
        let slot = self
            .projected_units
            .iter()
            .position(|u| u.is_some_and(|u| u.id == unit.id))
            .or_else(|| self.projected_units.iter().position(Option::is_none))
            .expect("three projected units");
        self.projected_units[slot] = Some(*unit);
        self.with_position(unit.id, tile)
    }
    pub(crate) fn revealing(mut self, id: u32, group: VisionGroupKey) -> Self {
        let slot = self
            .reveals
            .iter()
            .position(|v| v.is_none() || *v == Some((id, group)))
            .expect("two combat participants");
        self.reveals[slot] = Some((id, group));
        self
    }
    fn tile(&self, unit: &Unit) -> usize {
        match self
            .positions
            .iter()
            .flatten()
            .find(|(id, _)| *id == unit.id)
        {
            Some((_, tile)) => *tile,
            None => unit.tile,
        }
    }
    fn units(&self) -> impl Iterator<Item = &Unit> {
        self.game
            .units
            .iter()
            .filter(|u| !self.projected_units.iter().flatten().any(|p| p.id == u.id))
            .chain(self.projected_units.iter().flatten())
    }
    fn revealed(&self, unit: &Unit, group: VisionGroupKey) -> bool {
        self.game.unit_perception[unit.perception.0]
            .revealed_to
            .contains(&group)
            || self.reveals.contains(&Some((unit.id, group)))
    }
    fn with_capabilities<R>(
        &self,
        unit: &Unit,
        tile: usize,
        kind: EffectKind,
        query: impl FnOnce(&UnitCapabilities<'_>) -> R,
    ) -> R {
        let projected: Vec<_> = self
            .units()
            .filter(|u| {
                self.projected_units.iter().flatten().any(|p| p.id == u.id)
                    || self.positions.iter().flatten().any(|(id, _)| *id == u.id)
            })
            .map(|u| (*u, self.tile(u)))
            .collect();
        self.game
            .with_domain_capabilities_at(unit, tile, &self.game.units, &projected, kind, query)
    }
    pub(crate) fn profile(&self, unit: &Unit, target_illumination: Illumination) -> ResolvedSight {
        // With no live sight/stat emitters, geometry cannot change these facts.
        // A changed hypothetical peer may introduce an emitter, so it stays on
        // the general path. A changed recipient's own emissions are still composed.
        let independent = self.game.effect_sources.units[EffectKind::Sight as usize].is_empty()
            && self.game.effect_sources.terrain[EffectKind::Sight as usize].is_empty()
            && self.projected_units.iter().flatten().all(|peer| {
                peer.id == unit.id
                    || self.game.index(peer.id).is_some_and(|index| {
                        let live = &self.game.units[index];
                        live.kind == peer.kind && live.inventory == peer.inventory
                    })
            });
        if independent {
            let faction = self.game.unit_faction(unit);
            let key = SightKey {
                kind: unit.kind,
                equipment: self.game.runtime.inventories.identity(unit.inventory),
                own_relation: (
                    self.game.faction_relation(faction, faction) == Relation::Allied,
                    self.game.faction_relation(faction, faction) == Relation::Hostile,
                ),
                group: self.game.vision_groups[unit.vision_group.0].sight.clone(),
                modifiers: Snapshot(std::rc::Rc::clone(&self.game.map_modifiers)),
                perception: self.game.unit_perception[unit.perception.0].effects.clone(),
                illumination: target_illumination,
            };
            if let Some((range, penetrates)) =
                self.game.effect_sources.sight_profiles.borrow().get(&key)
            {
                return ResolvedSight {
                    range: *range,
                    penetrates: penetrates.clone(),
                };
            }
            let result = self.with_capabilities(unit, self.tile(unit), EffectKind::Sight, |c| {
                c.sight(target_illumination)
            });
            if self.game.effect_sources.sight_profiles.borrow().len() >= 4096 {
                self.game.effect_sources.sight_profiles.borrow_mut().clear();
            }
            self.game
                .effect_sources
                .sight_profiles
                .borrow_mut()
                .insert(key, (result.range, result.penetrates.clone()));
            return result;
        }
        self.with_capabilities(unit, self.tile(unit), EffectKind::Sight, |c| {
            c.sight(target_illumination)
        })
    }
    fn personal_rules(&self, unit: &Unit, tile: usize) -> Vec<ConcealmentRule> {
        self.with_capabilities(unit, tile, EffectKind::Concealment, |c| {
            c.concealment_rules().cloned().collect()
        })
    }
    fn applies(&self, rule: &ConcealmentRule, tile: usize) -> bool {
        self.game
            .terrain_definition(tile)
            .matches_concealment_requirements(&rule.requires)
            && match &rule.requires.illumination {
                None => true,
                Some(levels) => levels.contains(&self.illumination(tile)),
            }
    }
    pub(crate) fn concealment_at(&self, unit: &Unit, tile: usize) -> ConcealmentFacts {
        let personal = self.personal_rules(unit, tile);
        let mut result = ConcealmentFacts {
            available: false,
            reveal_distance: 0,
        };
        for rule in self.game.tile_perception[tile]
            .concealment
            .iter()
            .chain(&personal)
        {
            if self.applies(rule, tile) {
                result.reveal_distance = if result.available {
                    result.reveal_distance.min(rule.reveal_within)
                } else {
                    rule.reveal_within
                };
                result.available = true;
            }
        }
        result
    }
    pub(crate) fn visual_cover(&self, unit: &Unit) -> (bool, bool) {
        let tile = self.tile(unit);
        let personal = self.personal_rules(unit, tile);
        let mut eligible = false;
        let mut active = false;
        for rule in self.game.tile_perception[tile]
            .concealment
            .iter()
            .chain(&personal)
        {
            if let ConcealmentVisual::Concealment = self
                .game
                .catalogue
                .concealment_kind(rule.kind)
                .unit_appearance()
            {
                eligible = true;
                active |= self.applies(rule, tile);
            }
        }
        (eligible, active)
    }
    fn owns_light(&self, unit: &Unit) -> bool {
        self.game.effect_sources.units[EffectKind::Light as usize].contains(&unit.id)
            || self.game.index(unit.id).is_none_or(|index| {
                let live = &self.game.units[index];
                live.kind != unit.kind || live.inventory != unit.inventory
            })
    }
    pub(crate) fn unit_lights(&self, unit: &Unit, mut visit: impl FnMut(&LightEmission)) {
        if self.owns_light(unit) {
            let owned = self
                .game
                .definitions
                .owned(unit, &self.game.runtime.inventories);
            for &effect in &owned.effects {
                if self.game.effect_applies(effect.scope(), unit, unit, 0) {
                    if let EffectValue::Light { radius } = effect.value() {
                        visit(&LightEmission { radius: *radius });
                    }
                }
            }
        }
        for effect in &self.game.unit_perception[unit.perception.0].effects {
            if let Some(light) = &effect.light {
                visit(light);
            }
        }
    }
    pub(crate) fn visit_light_sources(
        &self,
        mut visit: impl FnMut(
            Option<&Unit>,
            usize,
            Option<&LightEmission>,
            Option<&BattlefieldLight>,
        ) -> bool,
    ) {
        for light in self.game.fixed_lights.iter() {
            if visit(
                None,
                light.tile,
                light.emission.as_ref(),
                Some(&light.appearance),
            ) {
                return;
            }
        }
        let live = self.game.effect_sources.units[EffectKind::Light as usize]
            .iter()
            .filter(|id| !self.projected_units.iter().flatten().any(|p| p.id == **id))
            .filter_map(|id| self.game.unit(*id));
        let projected = self.projected_units.iter().flatten().filter(|unit| {
            self.owns_light(unit)
                || self.game.unit_perception[unit.perception.0]
                    .effects
                    .iter()
                    .any(|effect| effect.light.is_some())
        });
        for unit in live.chain(projected) {
            let mut stopped = false;
            self.unit_lights(unit, |emission| {
                if !stopped {
                    stopped = visit(Some(unit), self.tile(unit), Some(emission), None);
                }
            });
            if stopped {
                return;
            }
        }
    }
    pub(crate) fn illumination(&self, tile: usize) -> Illumination {
        let ambient = self.game.ambient_illumination[tile];
        if ambient == Illumination::Lit || self.game.observed_light.contains(tile) {
            return Illumination::Lit;
        }
        let mut lit = false;
        self.visit_light_sources(|_, origin, emission, _| {
            lit = emission
                .is_some_and(|light| self.game.board.distance(origin, tile) <= light.radius);
            lit
        });
        if lit { Illumination::Lit } else { ambient }
    }
    pub(crate) fn remembered(&self, group: VisionGroupKey, tile: usize) -> bool {
        self.game.tile_perception[tile].remember
            && self.game.fog_memory[&self.game.vision_groups[group.0].id]
                .get(tile / 64)
                .is_some_and(|bits| bits & (1 << (tile % 64)) != 0)
    }
    pub(crate) fn detection_facts(
        &self,
        source: &Unit,
        target: Option<&Unit>,
        tile: usize,
    ) -> DetectionFacts {
        let group = self.game.unit_vision_group(source);
        let distance = self.game.board.distance(self.tile(source), tile);
        let profile = self.profile(source, self.illumination(tile));
        let mut facts = DetectionFacts::unobserved();
        facts.in_range = match profile.range {
            None => true,
            Some(range) => distance <= range,
        };
        facts.revealed = target.is_some_and(|unit| self.revealed(unit, group));
        if !facts.in_range {
            return facts;
        }
        let personal = match target {
            Some(unit) => self.personal_rules(unit, tile),
            None => Vec::new(),
        };
        let environmental = self.game.tile_perception[tile]
            .concealment
            .iter()
            .filter(|rule| !self.remembered(group, tile) && self.applies(rule, tile));
        for rule in environmental.chain(personal.iter().filter(|rule| self.applies(rule, tile))) {
            facts.active.insert(rule.kind);
            if distance > rule.reveal_within && !profile.penetrates.contains(&rule.kind) {
                facts.blocking.insert(rule.kind);
            }
        }
        facts
    }
    // The source index proves all live observers have unrestricted sight and
    // there are no live concealment/light/stat sources. Hypothetical loadouts
    // can introduce those rules, so inspect them before using that proof.
    fn unrestricted(&self, target: Option<&Unit>) -> bool {
        !self.game.perception_sensitive()
            && self
                .projected_units
                .iter()
                .flatten()
                .chain(target)
                .all(|unit| {
                    self.game.unit_perception[unit.perception.0]
                        .effects
                        .is_empty()
                        && self
                            .game
                            .definitions
                            .owned(unit, &self.game.runtime.inventories)
                            .effects
                            .iter()
                            .all(|effect| {
                                !matches!(
                                    effect.value(),
                                    EffectValue::Sight { .. }
                                        | EffectValue::Concealment { .. }
                                        | EffectValue::Light { .. }
                                        | EffectValue::StatModifiers { .. }
                                )
                            })
                })
    }
    pub(crate) fn sees_terrain(&self, group: VisionGroupKey, tile: usize) -> bool {
        if self.unrestricted(None) {
            return self.units().any(|unit| unit.vision_group == group);
        }
        self.units().any(|unit| {
            unit.vision_group == group && self.detection_facts(unit, None, tile).detected()
        })
    }
    pub(crate) fn sees_unit(&self, group: VisionGroupKey, unit: &Unit) -> bool {
        unit.vision_group == group
            || self.revealed(unit, group)
            || (self.unrestricted(Some(unit))
                && self.units().any(|source| source.vision_group == group))
            || self.units().any(|source| {
                source.vision_group == group
                    && self
                        .detection_facts(source, Some(unit), self.tile(unit))
                        .detected()
            })
    }
    pub(crate) fn terrain_knowledge(&self, group: VisionGroupKey, tile: usize) -> TerrainKnowledge {
        if self.sees_terrain(group, tile) {
            TerrainKnowledge::Visible
        } else if self.remembered(group, tile) {
            TerrainKnowledge::Remembered
        } else {
            TerrainKnowledge::Unexplored
        }
    }
}
