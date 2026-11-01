//! Live condition state, using the catalog's shared evaluator.
use super::Game;
pub use crate::data::presentation::{PresentationInput, matches, resolve};
use crate::data::*;

impl Game<'_> {
    pub(crate) fn presentation_input(&self) -> PresentationInput {
        PresentationInput {
            factions: self.factions.as_ref().clone(),
            tiles: self.tiles(),
            units: self
                .units
                .iter()
                .map(|u| StartingUnit {
                    items: vec![],
                    faction: u.faction.clone(),
                    ai_group: u.ai_group.clone(),
                    col: self.board.positions[u.tile].0,
                    row: self.board.positions[u.tile].1,
                    r#type: self.rules.units[u.kind].id.clone(),
                    side: u.side,
                    name: u.name.clone(),
                    role: u.role.clone(),
                })
                .collect(),
            pending: self
                .mission
                .as_ref()
                .map(|mission| {
                    mission
                        .events
                        .iter()
                        .filter(|event| {
                            self.campaign
                                .as_ref()
                                .is_some_and(|state| state.marked(&format!("event:{}", event.id)))
                        })
                        .flat_map(|event| {
                            event.spawn.iter().flat_map(move |group| {
                                group
                                    .units
                                    .iter()
                                    .filter(move |unit| {
                                        !self
                                            .campaign
                                            .as_ref()
                                            .unwrap()
                                            .marked(&format!("spawn:{}:{}", event.id, unit.id))
                                    })
                                    .map(move |unit| {
                                        let mut unit = unit.clone();
                                        unit.faction =
                                            unit.faction.or_else(|| group.faction.clone());
                                        unit
                                    })
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            lighting: self
                .rules
                .map_definitions
                .iter()
                .find(|map| map.id == self.map_id)
                .map(|map| map.lighting.clone())
                .unwrap_or_default(),
            choices: self
                .campaign
                .as_ref()
                .and_then(|state| state.choices.clone())
                .unwrap_or_default(),
            marks: self
                .campaign
                .as_ref()
                .map(|c| c.marks.clone())
                .unwrap_or_default(),
            memory: self.condition_memory.clone(),
            round: self.round,
        }
    }
    pub(crate) fn event_condition_met(&self, condition: &Condition) -> bool {
        let mut input = self.presentation_input();
        if matches(condition, &input, None) {
            return true;
        }
        if let Some((actor, path)) = &self.traversal {
            let Some(index) = self.units.iter().position(|u| u.id == *actor) else {
                return false;
            };
            for &tile in path {
                input.units[index].col = self.board.positions[tile].0;
                input.units[index].row = self.board.positions[tile].1;
                if matches(condition, &input, None) {
                    return true;
                }
            }
        }
        false
    }
    pub(crate) fn group_active(&self, group: &EnemyGroup) -> bool {
        group.when.as_ref().is_none_or(|rule| {
            (rule.remember
                && self
                    .condition_memory
                    .contains(&format!("group:{}", group.id)))
                || matches(&rule.condition, &self.presentation_input(), None)
        })
    }
    pub(crate) fn presentation(&self) -> BattlefieldPresentation {
        let mut presentation = resolve(&self.presentation_input()).0;
        presentation.lighting.lights.extend(self.ritual_lights());
        presentation
    }
    pub(crate) fn remember_conditions(&mut self) {
        let group_memory = self.mission.as_ref().is_some_and(|m| {
            m.enemy_groups
                .iter()
                .any(|g| g.when.as_ref().is_some_and(|r| r.remember))
        });
        let light_memory = self.rules.map_definitions.iter().any(|map| {
            map.id == self.map_id
                && map
                    .lighting
                    .lights
                    .iter()
                    .any(|light| light.when.as_ref().is_some_and(|rule| rule.remember))
        });
        if !group_memory && !light_memory && !self.fog.iter().any(Option::is_some) {
            return;
        }
        let mut input = self.presentation_input();
        let mut positions = vec![None];
        if let Some((actor, path)) = &self.traversal {
            if let Some(index) = self.units.iter().position(|u| u.id == *actor) {
                positions.extend(
                    path.iter()
                        .map(|&tile| Some((index, self.board.positions[tile]))),
                );
            }
        }
        for position in positions {
            if let Some((index, (col, row))) = position {
                input.units[index].col = col;
                input.units[index].row = row;
            }
            input.memory = self.condition_memory.clone();
            self.condition_memory = resolve(&input).1;
            if let Some(mission) = &self.mission {
                for group in &mission.enemy_groups {
                    if group
                        .when
                        .as_ref()
                        .is_some_and(|rule| rule.remember && matches(&rule.condition, &input, None))
                    {
                        let key = format!("group:{}", group.id);
                        if !self.condition_memory.contains(&key) {
                            self.condition_memory.push(key);
                        }
                    }
                }
            }
        }
    }
}
