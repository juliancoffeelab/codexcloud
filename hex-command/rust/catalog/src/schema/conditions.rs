use super::*;
use serde::{Deserialize, Serialize};
/// Unit allegiance selectors are distinct from the two turn/ownership phases.
/// `Any` tests existence without excluding passive or rescued actors.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionSide {
    Player,
    Enemy,
    Neutral,
    Any,
    /// The exact viewing faction, never all factions acting in its phase.
    Observer,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Condition {
    Owner {
        tile: Option<TileRef>,
        /// Exact faction identifier (the authoring key predates faction economies).
        side: Text,
    },
    Proximity {
        #[serde(rename = "unitTypes")]
        unit_types: Vec<Text>,
        tile: Option<TileRef>,
        radius: u16,
        side: ConditionSide,
        #[serde(rename = "unitType")]
        unit_type: Option<Text>,
        character: Option<Text>,
    },
    /// A matching unit stands on a tile currently visible through authored fog.
    VisibleUnits {
        side: ConditionSide,
        #[serde(rename = "unitTypes")]
        unit_types: Vec<Text>,
    },
    HasUnits {
        #[serde(rename = "aiGroup")]
        ai_group: Option<Text>,
        side: ConditionSide,
        #[serde(rename = "unitType")]
        unit_type: Option<Text>,
        character: Option<Text>,
    },
    NoUnits {
        side: ConditionSide,
        #[serde(rename = "unitType")]
        unit_type: Option<Text>,
        character: Option<Text>,
    },
    Choice {
        choice: Text,
    },
    Flag {
        flag: Text,
    },
    Event {
        event: Text,
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
pub struct ConditionRule {
    pub observer: Option<Text>,
    pub condition: Condition,
    pub remember: bool,
}
