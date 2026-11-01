//! One evaluator shared by gameplay triggers and presentation resolution.
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
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn matches(condition: &Condition, input: &PresentationInput, target: Option<&str>) -> bool {
    match condition {
        Condition::Owner { tile, side } => input
            .tiles
            .iter()
            .any(|t| Some(t.id.as_str()) == tile.as_deref().or(target) && t.owner == Some(*side)),
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
                    && hex_distance((tile.col, tile.row), (unit.col, unit.row)) <= *radius
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
        Condition::All { conditions } => conditions.iter().all(|c| matches(c, input, target)),
        Condition::Any { conditions } => conditions.iter().any(|c| matches(c, input, target)),
    }
}
/// Returns resolved visuals plus durable facts; callers decide whether to persist facts.
#[cfg_attr(not(target_arch = "wasm32"), inline)]
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

fn hex_distance(first: (i32, i32), second: (i32, i32)) -> u16 {
    let q = first.0 - first.1.div_euclid(2) - second.0 + second.1.div_euclid(2);
    let r = first.1 - second.1;
    ((q.abs() + r.abs() + (q + r).abs()) / 2) as u16
}
