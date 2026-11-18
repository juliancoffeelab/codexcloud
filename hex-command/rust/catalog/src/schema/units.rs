use super::*;
use serde::{Deserialize, Serialize};
/// The human and automatic turn phases. Factions define diplomatic allegiance.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnPhase {
    Player,
    Ai,
}
impl TurnPhase {
    /// Return the stable dense-array index for this enum value.
    #[inline]
    pub fn index(self) -> usize {
        if self == Self::Player { 0 } else { 1 }
    }
    /// Return the next turn phase.
    #[inline]
    pub fn other(self) -> Self {
        if self == Self::Player {
            Self::Ai
        } else {
            Self::Player
        }
    }
}
/// Weapon category used to select compatible retaliation attacks.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Melee,
    Ranged,
}
impl Category {
    /// Return the stable dense-array index for this enum value.
    #[inline]
    pub fn index(self) -> usize {
        if self == Self::Melee { 0 } else { 1 }
    }
}
/// Physical damage category indexing resistance and terrain modifiers.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DamageType {
    Blade,
    Pierce,
    Impact,
}
impl DamageType {
    /// Return the stable dense-array index for this enum value.
    #[inline]
    pub fn index(self) -> usize {
        match self {
            Self::Blade => 0,
            Self::Pierce => 1,
            Self::Impact => 2,
        }
    }
}
/// An authored attack option with independent uniform strike rolls.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Weapon {
    pub id: Text,
    pub name: Text,
    pub damage: i32,
    /// Uniform raw roll is damage ± spread, inclusive.
    pub spread: i32,
    /// Authored hits in the selected attack.
    pub count: u8,
    pub damage_type: DamageType,
    pub category: Category,
    pub range: u8,
    pub sound: Text,
    /// Percentage of actual HP removed restored to a surviving wielder.
    pub drain: u8,
}
/// Equipment owned by an individual unit, including its granted attacks.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub sight: SightContribution,
    pub effects: Vec<ScopedEffect>,
    #[serde(deserialize_with = "super::explicit_option")]
    pub armor: Option<ItemArmor>,
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub attacks: Vec<Weapon>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemArmor {
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
impl ItemArmor {
    #[inline]
    pub fn get(&self, kind: DamageType) -> i32 {
        match kind {
            DamageType::Blade => self.blade,
            DamageType::Pierce => self.pierce,
            DamageType::Impact => self.impact,
        }
    }
}
/// Per-damage-type adjustments contributed by a defending unit.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageModifiers {
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
impl DamageModifiers {
    /// Return the defending modifier for the selected damage type.
    #[inline]
    pub fn get(&self, damage: DamageType) -> i32 {
        match damage {
            DamageType::Blade => self.blade,
            DamageType::Pierce => self.pierce,
            DamageType::Impact => self.impact,
        }
    }
}
/// Authored battlefield role used for mixed composition and tactical placement.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitRole {
    Warrior,
    Flank,
    Ranger,
    Healer,
}
/// Stable display and resource identity. Mechanical values require a battle Catalogue.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitMetadata {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub ability: Text,
    pub family: Family,
    pub race: Race,
    pub role: UnitRole,
    pub level: u32,
    pub advances_to: Vec<Text>,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HealingTargets {
    OwnFaction,
    Allies,
    Enemies,
    Everyone,
}
