//! Shared perception for units with limited visibility.
//!
//! The intent is to put players, rule-based AI and neural-network AI on roughly
//! equal footing through the same visibility rules and permitted knowledge.
//!
//! AI nudges may prevent stalls or trivial play under obstructed visibility, but
//! they are explicit exceptions. A human must authorize each exception and the
//! way it obtains and uses information beyond ordinary perception.
//!
//! Visibility rules and authorized exceptions are authored in JSON. Rust
//! interprets those definitions and executes the shared mechanics, rather than
//! embedding unit-, map- or controller-specific visibility policy.
//!
//! Detection operates on physical facts without evaluating campaign flags or
//! callbacks.
use super::BattlePosition;
use super::Perception;
use super::effect_index::EffectKind;
use crate::data::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedFixedLight {
    pub tile: usize,
    pub emission: Option<LightEmission>,
    pub appearance: BattlefieldLight,
}

impl<'catalog> BattlePosition<'catalog> {
    #[inline]
    pub(crate) fn vision_group(&self, faction: &str) -> VisionGroupKey {
        let id = &self
            .faction(faction)
            .expect("validated faction")
            .vision_group;
        VisionGroupKey(
            self.vision_groups
                .iter()
                .position(|g| &g.id == id)
                .expect("validated vision group"),
        )
    }
    #[inline]
    pub(crate) fn unit_vision_group(&self, unit: &Unit) -> VisionGroupKey {
        unit.vision_group
    }
    #[inline]
    pub(crate) fn perception(&self, position: Option<(u32, usize)>) -> Perception<'_, 'catalog> {
        Perception::new(self, position)
    }

    pub(crate) fn reveal_weapon_user(&mut self, id: u32, recipient: VisionGroupKey) {
        self.reveal_contact(id, recipient);
    }
    pub(crate) fn reveal_contact(&mut self, id: u32, recipient: VisionGroupKey) {
        let slot = self.unit(id).expect("weapon user").perception.0;
        if !self.unit_perception[slot].revealed_to.contains(&recipient) {
            self.runtime.unit_perception[slot]
                .revealed_to
                .push(recipient);
        }
    }
    pub(crate) fn fresh_perception(&mut self) -> PerceptionId {
        let id = PerceptionId(self.unit_perception.len());
        self.runtime
            .unit_perception
            .push(UnitPerceptionFacts::default());
        id
    }
}
impl BattlePosition<'_> {
    pub(crate) fn add_concealment(&mut self, tiles: &[usize], rule: &ConcealmentRule) {
        self.runtime.perception_sensitive = true;
        let perception = std::rc::Rc::make_mut(&mut self.runtime.tile_perception);
        for &tile in tiles {
            if !perception[tile].concealment.contains(rule) {
                perception[tile].concealment.push(rule.clone());
            }
        }
    }
    pub(crate) fn remove_concealment(&mut self, tiles: &[usize], kind: ConcealmentKind) {
        if tiles.iter().any(|&t| {
            self.tile_perception[t]
                .concealment
                .iter()
                .any(|r| r.kind == kind)
        }) {
            let perception = std::rc::Rc::make_mut(&mut self.runtime.tile_perception);
            for &tile in tiles {
                perception[tile].concealment.retain(|r| r.kind != kind);
            }
        }
    }
    pub(crate) fn grant_perception(&mut self, id: u32, effect: PerceptionEffect) {
        self.runtime.perception_sensitive = true;
        let unit = self.index(id).expect("effect recipient");
        if self.units[unit].perception.0 == 0 {
            let fresh = self.fresh_perception();
            self.runtime.units[unit].perception = fresh;
        }
        let slot = self.units[unit].perception.0;
        let effects = &mut self.runtime.unit_perception[slot].effects;
        if let Some(previous) = effects.iter_mut().find(|e| e.id == effect.id) {
            *previous = effect;
        } else {
            effects.push(effect);
        }
        self.refresh_effect_sources();
    }
    pub(crate) fn reveal_terrain(&mut self, id: &str, tiles: &[usize]) {
        let group = self
            .vision_groups
            .iter()
            .position(|g| g.id == id)
            .expect("validated reveal group");
        let memory = std::rc::Rc::make_mut(&mut self.runtime.fog_memory)
            .get_mut(id)
            .unwrap();
        let groups = std::rc::Rc::make_mut(&mut self.runtime.vision_groups);
        for &tile in tiles {
            if memory[tile / 64] & (1 << (tile % 64)) == 0 {
                memory[tile / 64] |= 1 << (tile % 64);
                groups[group].explored.push(self.runtime.board.ids[tile]);
            }
        }
    }
    pub(crate) fn assign_vision_group(&mut self, faction: &str, destination: &str) {
        self.runtime.perception_sensitive = true;
        let source = self.vision_group(faction);
        let target = VisionGroupKey(
            self.vision_groups
                .iter()
                .position(|g| g.id == destination)
                .expect("validated destination group"),
        );
        if source == target {
            return;
        }
        let history: Vec<_> = self.vision_groups[source.0]
            .explored
            .iter()
            .map(|tile| self.board.indices[tile])
            .collect();
        self.reveal_terrain(destination, &history);
        for facts in &mut *self.runtime.unit_perception {
            if facts.revealed_to.contains(&source) && !facts.revealed_to.contains(&target) {
                facts.revealed_to.push(target);
            }
        }
        std::rc::Rc::make_mut(&mut self.runtime.factions)
            .iter_mut()
            .find(|f| f.id == faction)
            .unwrap()
            .vision_group = destination.to_owned();
        let key = self.runtime.names.find(faction).unwrap();
        for unit in &mut self.runtime.units {
            if unit.faction == key {
                unit.vision_group = target;
            }
        }
    }
}
impl BattlePosition<'_> {
    /// Fast rejection for ordinary arenas. No visibility work or extra threat
    /// caches are needed when there is no spatial perception restriction.
    #[inline]
    pub(crate) fn perception_sensitive(&self) -> bool {
        self.runtime.perception_sensitive
    }
    pub(crate) fn refresh_perception_flags(&mut self) {
        self.runtime.perception_sensitive = self.calculate_perception_sensitive();
    }
    fn calculate_perception_sensitive(&self) -> bool {
        if self
            .vision_groups
            .iter()
            .any(|group| !group.sight.unlimited)
            || self
                .tile_perception
                .iter()
                .any(|tile| !tile.concealment.is_empty())
            || [
                EffectKind::Sight,
                EffectKind::Concealment,
                EffectKind::Light,
                EffectKind::Stats,
            ]
            .iter()
            .any(|kind| {
                !self.effect_sources.terrain[*kind as usize].is_empty()
                    || !self.effect_sources.units[*kind as usize].is_empty()
            })
        {
            return true;
        }
        self.units.iter().any(|unit| {
            !self.unit_perception[unit.perception.0].effects.is_empty()
                || self.with_capabilities_at(unit, unit.tile, &self.units, &[], |caps| {
                    caps.concealment_rules().next().is_some()
                        || [
                            Illumination::Lit,
                            Illumination::Unlit,
                            Illumination::Darkness,
                        ]
                        .into_iter()
                        .any(|light| caps.sight(light).range.is_some())
                })
        })
    }

    /// The caller already checked geometric coverage. Preserve the particular
    /// firing origin while checking group spotting; one attacker cannot stand
    /// beside its target and fire from a different hex in the same projection.
    pub(crate) fn can_threaten_from(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        origins: &super::geometry::TileSet,
    ) -> bool {
        if !self.perception_sensitive() {
            return true;
        }
        let (category, range) = self.with_attack(attacker, weapon, |attack| {
            (attack.category(), attack.range())
        });
        let firing = self.attack_origins(category, range);
        let group = attacker.vision_group;
        let mut detected = false;
        origins.for_each(|origin| {
            if detected || !firing[target.tile].contains(origin) {
                return;
            }
            let projected = || {
                self.perception(Some((attacker.id, origin)))
                    .with_position(target.id, target.tile)
            };
            if projected().sees_unit(group, target) {
                detected = true;
                return;
            }
            // Other already-known group members may move to provide a sighting.
            // Ignore competing friendly destinations for a conservative threat.
            for spotter in self
                .units
                .iter()
                .filter(|u| u.id != attacker.id && u.vision_group == group)
            {
                let mut blocked = self.occupied_tiles.clone();
                blocked.remove(target.tile);
                blocked.remove(attacker.tile);
                let reach = self.threat_reach_blocked(spotter, blocked);
                reach.for_each(|tile| {
                    if !detected
                        && tile != origin
                        && projected()
                            .with_position(spotter.id, tile)
                            .detection_facts(spotter, Some(target), target.tile)
                            .detected()
                    {
                        detected = true;
                    }
                });
                if detected {
                    break;
                }
            }
        });
        detected
    }
}
