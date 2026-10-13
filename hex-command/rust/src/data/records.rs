//! Owned action, combat and loss records produced by simulation sessions.

use super::*;
use serde::{Deserialize, Serialize};
/// Commands recorded by simulations in their established wire shape.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ActionRecord {
    Move {
        unit: String,
        tile: String,
    },
    Attack {
        unit: String,
        target: String,
        weapon: String,
    },
    Recruit {
        unit: String,
        #[serde(rename = "type")]
        unit_type: String,
        tile: String,
    },
    Advance {
        unit: String,
        #[serde(rename = "type")]
        unit_type: String,
    },
}
/// The positional and damage details of a resolved attack for native reports.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatRecord {
    pub unit: String,
    pub target: String,
    pub weapon: String,
    pub from: String,
    pub target_tile: String,
    pub damage: i32,
    pub retaliation: i32,
    #[serde(default)]
    pub count: u8,
    #[serde(default)]
    pub damage_per_hit: i32,
    #[serde(default)]
    pub retaliation_count: u8,
    #[serde(default)]
    pub retaliation_per_hit: i32,
    pub mediation_at_risk: bool,
}
/// Exclusively owned action, casualty and combat histories for one simulation session.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Records {
    pub actions: Vec<ActionRecord>,
    pub losses: Vec<PublicUnit>,
    pub combats: Vec<CombatRecord>,
}
