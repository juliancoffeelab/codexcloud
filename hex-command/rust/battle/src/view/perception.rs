//! Permitted-information visibility queries shared by planners and learned inputs.
use super::*;
use crate::logic::Perception;

fn directly_detected(facts: DetectionFacts) -> bool {
    facts.in_range && (facts.revealed || facts.blocking == ConcealmentMask::of(&[]))
}
impl BattleView<'_, '_> {
    pub fn sight_range(&self, unit: &Unit) -> Option<u16> {
        self.engine()
            .perception(None)
            .profile(unit, self.illumination(unit.tile))
            .range
    }
    pub fn concealment_at(&self, unit: &Unit, tile: usize) -> ConcealmentFacts {
        self.engine()
            .perception(Some((unit.id, tile)))
            .concealment_at(unit, tile)
    }
    pub fn terrain_remembered(&self, tile: usize) -> bool {
        self.engine()
            .perception(None)
            .remembered(self.engine().vision_group(self.observer_faction()), tile)
    }
    pub fn can_observe_terrain(&self, observer: &Unit, tile: usize) -> bool {
        self.engine()
            .perception(None)
            .detection_facts(observer, None, tile)
            .detected()
    }
    pub fn directly_detects(&self, observer: &Unit, target: &Unit) -> bool {
        directly_detected(self.engine().perception(None).detection_facts(
            observer,
            Some(target),
            target.tile,
        ))
    }
    fn hostile_vision_group(&self, group: VisionGroupKey) -> bool {
        self.engine().factions.iter().any(|f| {
            self.engine().vision_group(&f.id) == group
                && self.faction_relation(self.observer_faction(), &f.id) == Relation::Hostile
        })
    }
    pub fn friendly_detector_count(&self, target: &Unit) -> usize {
        self.detector_count(&self.engine().perception(None), target, false)
    }
    fn detector_count(
        &self,
        perception: &Perception<'_, '_>,
        target: &Unit,
        hostile: bool,
    ) -> usize {
        let own = self.engine().vision_group(self.observer_faction());
        let relevant = |group| {
            if hostile {
                self.hostile_vision_group(group)
            } else {
                group == own
            }
        };
        self.units()
            .iter()
            .filter(|source| {
                relevant(source.vision_group)
                    && directly_detected(perception.detection_facts(
                        source,
                        Some(target),
                        target.tile,
                    ))
            })
            .count()
    }
    pub fn tile_insight_of_enemy(&self, tile: usize) -> bool {
        let perception = self.engine().perception(None);
        self.units().iter().any(|u| {
            self.hostile_to_faction(u, self.observer_faction())
                && perception.detection_facts(u, None, tile).detected()
        })
    }
    pub fn possible_enemy_hide_place(&self, tile: usize) -> bool {
        let game = self.engine();
        let terrain = game.terrain_definition(tile);
        let illumination = std::cell::OnceCell::new();
        let applies = |rule: &ConcealmentRule| {
            terrain.matches_concealment_requirements(&rule.requires)
                && match &rule.requires.illumination {
                    None => true,
                    Some(levels) => levels.contains(
                        illumination.get_or_init(|| game.perception(None).illumination(tile)),
                    ),
                }
        };
        game.tile_perception[tile].concealment.iter().any(applies)
            || game.catalogue.units().any(|unit| {
                unit.effects(&[], &[]).into_iter().any(|effect| {
                matches!(effect.value(), EffectValue::Concealment { rule } if applies(rule))
            })
            })
            || game.catalogue.abilities().any(|ability| {
                ability.effects().any(|effect| {
                matches!(effect.value(), EffectValue::Concealment { rule } if applies(rule))
            })
            })
    }

    /// Current and projected values use known units; discovery of an unseen
    /// occupant cannot be predicted without leaking its position.
    pub fn visibility_action_features(&self, command: &crate::RuntimeCommand) -> [f32; 14] {
        let mut x = [0.; 14];
        let (actor, destination, target, weapon) = match command {
            crate::RuntimeCommand::Move { unit, tile } => (unit.0, Some(tile.0), None, None),
            crate::RuntimeCommand::Attack {
                unit,
                target,
                weapon,
            } => (unit.0, None, Some(target.0), Some(weapon)),
            crate::RuntimeCommand::Advance { unit, .. } => (unit.0, None, None, None),
            _ => return x,
        };
        let Some(actor) = self.unit(actor) else {
            return x;
        };
        let target = target.and_then(|id| self.unit(id));
        let before = self.engine().perception(None);
        let mut after = self
            .engine()
            .perception(destination.map(|tile| (actor.id, tile)));
        let mut moved = *actor;
        if let Some(tile) = destination {
            moved.tile = tile;
        }
        if let Some(target) = target {
            after = after.revealing(actor.id, target.vision_group);
            if self
                .preview(
                    actor,
                    target,
                    self.engine()
                        .weapon_index(actor, *weapon.unwrap())
                        .expect("candidate weapon belongs to actor"),
                    actor.tile,
                )
                .2
                .is_some()
            {
                after = after.revealing(target.id, actor.vision_group);
            }
        }
        let cover_before = before.concealment_at(actor, actor.tile);
        let cover_after = after.concealment_at(&moved, moved.tile);
        x[0] = f32::from(cover_before.available);
        x[1] = f32::from(cover_after.available);
        x[2] = f32::from(cover_before.reveal_distance) / 20.;
        x[3] = f32::from(cover_after.reveal_distance) / 20.;
        x[4] = self.detector_count(&before, actor, true) as f32 / 20.;
        x[5] = self.detector_count(&after, &moved, true) as f32 / 20.;
        if let Some(target) = target {
            x[6] = f32::from(directly_detected(before.detection_facts(
                actor,
                Some(target),
                target.tile,
            )));
            x[7] = f32::from(directly_detected(after.detection_facts(
                &moved,
                Some(target),
                target.tile,
            )));
            x[8] = self.detector_count(&before, target, false) as f32 / 20.;
            x[9] = self.detector_count(&after, target, false) as f32 / 20.;
            x[10] = f32::from(directly_detected(before.detection_facts(
                target,
                Some(actor),
                actor.tile,
            )));
            x[11] = f32::from(directly_detected(after.detection_facts(
                target,
                Some(&moved),
                moved.tile,
            )));
        }
        let group = self.engine().vision_group(self.observer_faction());
        for enemy in self
            .units()
            .iter()
            .filter(|u| self.hostile_to_faction(u, self.observer_faction()))
        {
            x[12] += f32::from(before.sees_unit(group, enemy)) / 20.;
            x[13] += f32::from(after.sees_unit(group, enemy)) / 20.;
        }
        x
    }
}
