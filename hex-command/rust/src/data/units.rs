//! Resolved unit rules and serializable unit snapshots.

use super::*;
use serde::{Deserialize, Serialize};
/// The two battle allegiances, with a stable index into side-specific arrays.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Player,
    Enemy,
}
impl Side {
    /// Return the stable dense-array index for this enum value.
    pub fn index(self) -> usize {
        if self == Self::Player { 0 } else { 1 }
    }
    /// Return the opposing allegiance.
    pub fn other(self) -> Self {
        if self == Self::Player {
            Self::Enemy
        } else {
            Self::Player
        }
    }
}
/// Weapon category used to select compatible retaliation attacks.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Melee,
    Ranged,
}
impl Category {
    /// Return the stable dense-array index for this enum value.
    pub fn index(self) -> usize {
        if self == Self::Melee { 0 } else { 1 }
    }
}
/// Physical damage category indexing resistance and terrain modifiers.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageType {
    Blade,
    Pierce,
    Impact,
}
impl DamageType {
    /// Return the stable dense-array index for this enum value.
    pub fn index(self) -> usize {
        match self {
            Self::Blade => 0,
            Self::Pierce => 1,
            Self::Impact => 2,
        }
    }
}
/// An authored attack option with deterministic damage, range and feedback metadata.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub damage: i32,
    pub damage_type: DamageType,
    pub category: Category,
    pub range: u8,
    pub sound: String,
}
/// Per-damage-type adjustments contributed by a defending unit.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageModifiers {
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
impl DamageModifiers {
    /// Return the defending modifier for the selected damage type.
    pub fn get(&self, damage: DamageType) -> i32 {
        match damage {
            DamageType::Blade => self.blade,
            DamageType::Pierce => self.pierce,
            DamageType::Impact => self.impact,
        }
    }
}
/// Resolved unit statistics, advancement choices and dense terrain-dependent tables.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitRule {
    pub ability: String,
    pub description: String,
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub r#move: usize,
    pub damage: i32,
    pub range: u8,
    pub cost: i32,
    pub attacks: Vec<Weapon>,
    pub damage_modifiers: DamageModifiers,
    pub race: Race,
    #[serde(default)]
    pub leader: bool,
    #[serde(default)]
    pub healing: i32,
    #[serde(default)]
    pub level: u32,
    pub family: Family,
    pub experience_needed: Option<u32>,
    #[serde(default)]
    pub advances_to: Vec<String>,
    pub costs: Vec<Option<u16>>,
    pub environment: Vec<[[i32; 3]; 2]>,
}
/// An owned unit snapshot preserving stable IDs and optional campaign metadata.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUnit {
    pub id: String,
    pub r#type: String,
    pub side: Side,
    pub tile: String,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experience: Option<u32>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retreating: Option<bool>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuilding: Option<bool>,
}
#[derive(Clone, Debug)]
pub(crate) struct Unit {
    pub id: u32,
    pub kind: usize,
    pub side: Side,
    pub tile: usize,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    pub experience: Option<u32>,
    pub name: Option<String>,
    pub role: Option<Role>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
}
/// Feedback identifying the latest attack until the next turn clears it.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastCombat {
    pub attacker_id: String,
    pub attack_id: String,
    pub sound: String,
}
/// A surviving unit carried between campaign chapters with its earned experience.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Veteran {
    pub r#type: String,
    pub experience: u32,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}
