//! Shared state predicates for events, lighting and terrain visibility.
use super::{Event, Tile, TileRef, TurnPhase};
use serde::{Deserialize, Serialize};

/// Unit allegiance selectors are distinct from the two turn/ownership phases.
/// `Any` tests existence without excluding passive or rescued actors.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum ConditionSide {
    Player,
    Enemy,
    Neutral,
    Any,
    /// The exact viewing faction, never all factions acting in its phase.
    Observer,
}
impl From<TurnPhase> for ConditionSide {
    fn from(side: TurnPhase) -> Self {
        match side {
            TurnPhase::Player => Self::Player,
            TurnPhase::Ai => Self::Enemy,
        }
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Condition {
    Owner {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tile: Option<TileRef>,
        /// Exact faction identifier (the authoring key predates faction economies).
        side: String,
    },
    Proximity {
        #[serde(default, rename = "unitTypes", skip_serializing_if = "Vec::is_empty")]
        unit_types: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tile: Option<TileRef>,
        radius: u16,
        side: ConditionSide,
        #[serde(default, rename = "unitType", skip_serializing_if = "Option::is_none")]
        unit_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        character: Option<String>,
    },
    /// A matching unit stands on a tile currently visible through authored fog.
    VisibleUnits {
        side: ConditionSide,
        #[serde(default, rename = "unitTypes", skip_serializing_if = "Vec::is_empty")]
        unit_types: Vec<String>,
    },
    HasUnits {
        #[serde(default, rename = "aiGroup", skip_serializing_if = "Option::is_none")]
        ai_group: Option<String>,
        side: ConditionSide,
        #[serde(default, rename = "unitType", skip_serializing_if = "Option::is_none")]
        unit_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        character: Option<String>,
    },
    NoUnits {
        side: ConditionSide,
        #[serde(default, rename = "unitType", skip_serializing_if = "Option::is_none")]
        unit_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        character: Option<String>,
    },
    Choice {
        choice: String,
    },
    Flag {
        flag: String,
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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ConditionRule {
    pub condition: Condition,
    #[serde(default)]
    pub remember: bool,
}
impl Condition {
    /// Bind contextual tile predicates for tooling that needs explicit destinations.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn contextualized(&self, target: &TileRef) -> Self {
        let mut condition = self.clone();
        match &mut condition {
            Self::Owner { tile, .. } | Self::Proximity { tile, .. } => {
                if tile.is_none() {
                    *tile = Some(target.to_owned());
                }
            }
            Self::All { conditions } | Self::Any { conditions } => {
                *conditions = conditions
                    .iter()
                    .map(|condition| condition.contextualized(target))
                    .collect();
            }
            Self::VisibleUnits { .. }
            | Self::HasUnits { .. }
            | Self::NoUnits { .. }
            | Self::Choice { .. }
            | Self::Flag { .. }
            | Self::Event { .. }
            | Self::Round { .. } => {}
        }
        condition
    }

    #[cfg_attr(not(target_arch = "wasm32"), inline)]

    pub fn uses_proximity(&self) -> bool {
        match self {
            Self::Proximity { .. } | Self::VisibleUnits { .. } => true,
            Self::All { conditions } | Self::Any { conditions } => {
                conditions.iter().any(Self::uses_proximity)
            }
            _ => false,
        }
    }

    pub fn validate_characters(&self, characters: &[super::Character]) -> Result<(), String> {
        match self {
            Self::Proximity {
                character: Some(id),
                ..
            }
            | Self::HasUnits {
                character: Some(id),
                ..
            }
            | Self::NoUnits {
                character: Some(id),
                ..
            } => {
                if !characters.iter().any(|c| &c.id == id) {
                    return Err(format!("unknown character {id}"));
                }
            }
            Self::All { conditions } | Self::Any { conditions } => {
                for c in conditions {
                    c.validate_characters(characters)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn validate_presentation(&self) -> Result<(), String> {
        match self {
            Self::VisibleUnits { .. } => {
                Err("visibility queries cannot define fog or lighting conditions".into())
            }
            Self::All { conditions } | Self::Any { conditions } => {
                for condition in conditions {
                    condition.validate_presentation()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn validate_references(
        &self,
        types: &std::collections::HashMap<String, usize>,
        factions: &[super::Faction],
    ) -> Result<(), String> {
        match self {
            Self::Owner { side, .. }
                if !matches!(side.as_str(), "player" | "enemy")
                    && !factions.iter().any(|f| f.id == *side) =>
            {
                Err("condition references unknown owner faction".into())
            }
            Self::Proximity { unit_types, .. } | Self::VisibleUnits { unit_types, .. }
                if unit_types.iter().any(|kind| !types.contains_key(kind)) =>
            {
                Err("condition references unknown unit types".into())
            }
            Self::Proximity {
                unit_type: Some(kind),
                ..
            }
            | Self::HasUnits {
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
                    condition.validate_references(types, factions)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn validate(
        &self,
        tiles: &[Tile],
        events: &[Event],
        contextual: bool,
    ) -> Result<(), String> {
        match self {
            Self::Owner { tile, .. } | Self::Proximity { tile, .. } => match tile {
                Some(id) if !tiles.iter().any(|tile| tile.matches_reference(id)) => {
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
