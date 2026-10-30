//! Shared state predicates for events, lighting and terrain visibility.
use super::{Event, Side, Tile};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Condition {
    Owner {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tile: Option<String>,
        side: Side,
    },
    Proximity {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tile: Option<String>,
        radius: u16,
        side: Side,
        #[serde(default, rename = "unitType", skip_serializing_if = "Option::is_none")]
        unit_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    NoUnits {
        side: Side,
        #[serde(default, rename = "unitType", skip_serializing_if = "Option::is_none")]
        unit_type: Option<String>,
    },
    Event {
        event: String,
    },
    Round {
        round: u32,
    },
    All {
        conditions: Vec<Condition>,
    },
    Any {
        conditions: Vec<Condition>,
    },
}
/// Remembering satisfaction is independent of the predicate being evaluated.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionRule {
    pub condition: Condition,
    #[serde(default)]
    pub remember: bool,
}
impl Condition {
    pub(super) fn validate_units(
        &self,
        types: &std::collections::HashMap<String, usize>,
    ) -> Result<(), String> {
        match self {
            Self::Proximity {
                unit_type: Some(kind),
                ..
            }
            | Self::NoUnits {
                unit_type: Some(kind),
                ..
            } if !types.contains_key(kind) => {
                Err(format!("condition references unknown unit type: {kind}"))
            }
            Self::All { conditions } | Self::Any { conditions } => {
                for condition in conditions {
                    condition.validate_units(types)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub(super) fn validate(
        &self,
        tiles: &[Tile],
        events: &[Event],
        contextual: bool,
    ) -> Result<(), String> {
        match self {
            Self::Owner { tile, .. } | Self::Proximity { tile, .. } => match tile {
                Some(id) if !tiles.iter().any(|tile| tile.id == *id) => {
                    return Err(format!("condition references unknown tile: {id}"));
                }
                None if !contextual => return Err("condition requires a target tile".into()),
                _ => {}
            },
            Self::Event { event } if !events.iter().any(|candidate| candidate.id == *event) => {
                return Err(format!("condition references unknown event: {event}"));
            }
            Self::Round { round: 0 } => return Err("condition round must be positive".into()),
            Self::All { conditions } | Self::Any { conditions } => {
                if conditions.is_empty() {
                    return Err("compound condition must not be empty".into());
                }
                for condition in conditions {
                    condition.validate(tiles, events, contextual)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
