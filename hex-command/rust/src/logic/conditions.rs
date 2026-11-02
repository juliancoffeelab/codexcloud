//! One evaluator shared by gameplay triggers and presentation resolution.
use super::Game;
use crate::data::*;
use serde::{Deserialize, Serialize};

/// Devtool input for the same resolver used by live Rust battles.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationInput {
    #[serde(default)]
    pub factions: Vec<Faction>,
    pub tiles: Vec<Tile>,
    pub units: Vec<StartingUnit>,
    #[serde(default)]
    pub pending: Vec<SpawnUnit>,
    pub lighting: BattlefieldLighting,
    #[serde(default)]
    pub marks: Vec<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub memory: Vec<String>,
    pub round: u32,
}
pub(crate) fn matches(
    condition: &Condition,
    input: &PresentationInput,
    target: Option<&str>,
) -> bool {
    evaluate(condition, input, target, true)
}
fn evaluate(
    condition: &Condition,
    input: &PresentationInput,
    target: Option<&str>,
    allow_visibility: bool,
) -> bool {
    match condition {
        Condition::Owner { tile, side } => input.tiles.iter().any(|t| {
            Some(t.id.as_str()) == tile.as_deref().or(target)
                && t.owner.as_deref() == Some(side.as_str())
        }),
        Condition::Proximity {
            tile,
            radius,
            side,
            unit_type,
            unit_types,
            name,
        } => {
            let Some(tile) = input
                .tiles
                .iter()
                .find(|t| Some(t.id.as_str()) == tile.as_deref().or(target))
            else {
                return false;
            };
            input.units.iter().any(|unit| {
                faction_matches_side(&input.factions, unit.faction.as_deref(), unit.side, *side)
                    && unit_type.as_ref().is_none_or(|kind| unit.r#type == *kind)
                    && (unit_types.is_empty() || unit_types.contains(&unit.r#type))
                    && name
                        .as_ref()
                        .is_none_or(|name| unit.name.as_ref() == Some(name))
                    && super::battle::hex_distance((tile.col, tile.row), (unit.col, unit.row))
                        <= *radius
            })
        }
        Condition::VisibleUnits { side, unit_types } => {
            allow_visibility
                && input.units.iter().any(|unit| {
                    faction_matches_side(&input.factions, unit.faction.as_deref(), unit.side, *side)
                        && (unit_types.is_empty() || unit_types.contains(&unit.r#type))
                        && input
                            .tiles
                            .iter()
                            .find(|tile| tile.col == unit.col && tile.row == unit.row)
                            .is_some_and(|tile| {
                                tile.fog.as_ref().is_none_or(|rule| {
                                    (rule.remember
                                        && input.memory.contains(&format!("fog:{}", tile.id)))
                                        || evaluate(&rule.condition, input, Some(&tile.id), false)
                                })
                            })
                })
        }
        Condition::HasUnits {
            side,
            unit_type,
            name,
        } => input.units.iter().any(|unit| {
            faction_matches_side(&input.factions, unit.faction.as_deref(), unit.side, *side)
                && unit_type.as_ref().is_none_or(|kind| unit.r#type == *kind)
                && name
                    .as_ref()
                    .is_none_or(|name| unit.name.as_ref() == Some(name))
        }),
        Condition::NoUnits {
            side,
            unit_type,
            name,
        } => {
            !input.units.iter().any(|unit| {
                faction_matches_side(&input.factions, unit.faction.as_deref(), unit.side, *side)
                    && unit_type.as_ref().is_none_or(|kind| unit.r#type == *kind)
                    && name
                        .as_ref()
                        .is_none_or(|name| unit.name.as_ref() == Some(name))
            }) && !input.pending.iter().any(|unit| {
                faction_matches_side(&input.factions, unit.faction.as_deref(), unit.side, *side)
                    && unit_type.as_ref().is_none_or(|kind| unit.r#type == *kind)
                    && name
                        .as_ref()
                        .is_none_or(|name| unit.name.as_ref() == Some(name))
            })
        }
        Condition::Choice { choice } => input.choices.contains(choice),
        Condition::Event { event } => input
            .marks
            .iter()
            .any(|mark| mark == &format!("event:{event}")),
        Condition::Round { round } => input.round >= *round,
        Condition::All { conditions } => conditions
            .iter()
            .all(|c| evaluate(c, input, target, allow_visibility)),
        Condition::Any { conditions } => conditions
            .iter()
            .any(|c| evaluate(c, input, target, allow_visibility)),
    }
}
/// Returns resolved visuals plus durable facts; callers decide whether to persist facts.
pub fn resolve(input: &PresentationInput) -> (BattlefieldPresentation, Vec<String>) {
    let mut memory = input.memory.clone();
    let mut rule_met = |rule: &ConditionRule, key: String, target: &str| {
        let satisfied = matches(&rule.condition, input, Some(target));
        if rule.remember {
            if memory.contains(&key) {
                return true;
            }
            if satisfied {
                memory.push(key);
            }
        }
        satisfied
    };
    let lighting = BattlefieldLighting {
        time: input.lighting.time,
        lights: input
            .lighting
            .lights
            .iter()
            .enumerate()
            .filter(|(i, light)| {
                light.when.as_ref().is_none_or(|rule| {
                    rule_met(
                        rule,
                        format!("light:{i}:{},{}", light.col, light.row),
                        &format!("{},{}", light.col, light.row),
                    )
                })
            })
            .map(|(_, light)| {
                let mut light = light.clone();
                light.when = None;
                light
            })
            .collect(),
    };
    let fog = input
        .tiles
        .iter()
        .filter(|tile| {
            tile.fog
                .as_ref()
                .is_some_and(|rule| !rule_met(rule, format!("fog:{}", tile.id), &tile.id))
        })
        .map(|tile| tile.id.clone())
        .collect();
    (BattlefieldPresentation { lighting, fog }, memory)
}
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
