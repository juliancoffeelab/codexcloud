//! Resolved unit rules and serializable unit snapshots.

use super::*;
use serde::{Deserialize, Serialize};
/// The human and automatic turn phases. Factions define diplomatic allegiance.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Side {
    Player,
    Enemy,
}
impl Side {
    /// Return the stable dense-array index for this enum value.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn index(self) -> usize {
        if self == Self::Player { 0 } else { 1 }
    }
    /// Return the opposing allegiance.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn other(self) -> Self {
        if self == Self::Player {
            Self::Enemy
        } else {
            Self::Player
        }
    }
}
/// Weapon category used to select compatible retaliation attacks.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Category {
    Melee,
    Ranged,
}
impl Category {
    /// Return the stable dense-array index for this enum value.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn index(self) -> usize {
        if self == Self::Melee { 0 } else { 1 }
    }
}
/// Physical damage category indexing resistance and terrain modifiers.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum DamageType {
    Blade,
    Pierce,
    Impact,
}
impl DamageType {
    /// Return the stable dense-array index for this enum value.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn index(self) -> usize {
        match self {
            Self::Blade => 0,
            Self::Pierce => 1,
            Self::Impact => 2,
        }
    }
}
/// An authored attack option with independent uniform strike rolls.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub damage: i32,
    /// Uniform raw roll is damage ± spread, inclusive; omitted means fixed damage.
    #[serde(default)]
    pub spread: i32,
    /// Hits in the selected attack; old definitions retain one hit.
    #[serde(default = "one_attack")]
    pub count: u8,
    pub damage_type: DamageType,
    pub category: Category,
    pub range: u8,
    pub sound: String,
    /// Percentage of actual HP removed restored to a surviving wielder.
    #[serde(default)]
    pub drain: u8,
}

/// Equipment owned by an individual unit, including its granted attacks.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Item {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub armor: Option<ItemArmor>,
    pub id: String,
    pub name: String,
    pub description: String,
    pub attacks: Vec<Weapon>,
    /// A carried aura, applied at the start of the carrier's allied turn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub healing: Option<ItemHealing>,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ItemHealing {
    pub amount: i32,
    pub radius: u16,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ItemArmor {
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
impl ItemArmor {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn get(&self, kind: DamageType) -> i32 {
        match kind {
            DamageType::Blade => self.blade,
            DamageType::Pierce => self.pierce,
            DamageType::Impact => self.impact,
        }
    }
}
impl Weapon {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn validate(&self, path: &str) -> Result<(), ContentError> {
        super::catalog::identifier(&self.id, path)?;
        if self.name.is_empty()
            || self.sound.is_empty()
            || self.range == 0
            || self.damage < 0
            || self.damage > 1000
            || self.spread < 0
            || self.spread > self.damage
            || !(1..=16).contains(&self.count)
            || self.drain > 100
        {
            return Err(ContentError::new(
                path,
                format!("invalid weapon {}", self.id),
            ));
        }
        Ok(())
    }
}
impl Item {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn validate(&self, path: &str) -> Result<(), ContentError> {
        super::catalog::identifier(&self.id, path)?;
        if self.name.is_empty()
            || self.description.is_empty()
            || (self.attacks.is_empty() && self.healing.is_none() && self.armor.is_none())
        {
            return Err(ContentError::new(
                path,
                "item name, description and at least one effect are required",
            ));
        }
        if self.armor.as_ref().is_some_and(|a| {
            [a.blade, a.pierce, a.impact]
                .iter()
                .any(|v| !(-3..=0).contains(v))
                || [a.blade, a.pierce, a.impact] == [0, 0, 0]
        }) {
            return Err(ContentError::new(
                path,
                "armor needs a reduction between one and three",
            ));
        }
        if self
            .healing
            .as_ref()
            .is_some_and(|h| h.amount <= 0 || h.amount > 100 || h.radius > 10)
        {
            return Err(ContentError::new(path, "invalid item healing aura"));
        }
        super::catalog::unique(self.attacks.iter().map(|a| a.id.as_str()), path)?;
        for attack in &self.attacks {
            attack.validate(path)?;
        }
        Ok(())
    }
}

/// Per-damage-type adjustments contributed by a defending unit.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct DamageModifiers {
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
impl DamageModifiers {
    /// Return the defending modifier for the selected damage type.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn get(&self, damage: DamageType) -> i32 {
        match damage {
            DamageType::Blade => self.blade,
            DamageType::Pierce => self.pierce,
            DamageType::Impact => self.impact,
        }
    }
}
/// Authored battlefield role used for mixed composition and tactical placement.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum UnitRole {
    Warrior,
    Flank,
    Ranger,
    Healer,
}
impl UnitRole {
    /// All composition roles in stable dense-array order.
    pub const ALL: [Self; 4] = [Self::Warrior, Self::Flank, Self::Ranger, Self::Healer];
    /// Index into per-role counts and target weights.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub const fn index(self) -> usize {
        match self {
            Self::Warrior => 0,
            Self::Flank => 1,
            Self::Ranger => 2,
            Self::Healer => 3,
        }
    }
    /// Relative mixed-army target; normalize across roles present in the roster.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub const fn weight(self) -> f64 {
        if matches!(self, Self::Warrior) {
            0.3
        } else {
            0.2
        }
    }
    /// Whether recruitment evaluates the unit's ranged attack options.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub const fn is_ranged(self) -> bool {
        matches!(self, Self::Ranger | Self::Healer)
    }
}
/// Resolved unit statistics, advancement choices and dense terrain-dependent tables.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
    pub role: UnitRole,
    #[serde(default)]
    pub healing: i32,
    /// Damage reflected per incoming melee strike, including killing hits.
    #[serde(default)]
    pub thorns: i32,
    /// Adjacent allies receive the strongest aura, added to their own thorns.
    #[serde(default)]
    pub thorn_aura: i32,
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct PublicUnit {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub faction: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub items: Vec<Item>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub ai_group: Option<String>,
    pub id: String,
    pub r#type: String,
    pub side: Side,
    pub tile: Hex,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experience: Option<u32>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retreating: Option<bool>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuilding: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Unit {
    pub faction: Option<String>,
    pub items: Vec<Item>,
    pub ai_group: Option<String>,
    pub id: u32,
    pub kind: usize,
    pub side: Side,
    pub tile: usize,
    pub hp: i32,
    pub moved: bool,
    pub attacked: bool,
    pub experience: Option<u32>,
    pub character: Option<String>,
    pub role: Option<Role>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
}
impl Unit {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn is_leader(&self) -> bool {
        self.role == Some(Role::Leader)
    }
}
/// Feedback identifying the latest attack until the next turn clears it.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct LastCombat {
    pub attacker_id: String,
    pub attack_id: String,
    pub sound: String,
}
/// A surviving unit carried between campaign chapters with its earned experience.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Veteran {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub items: Vec<Item>,
    pub r#type: String,
    pub experience: u32,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
}

fn one_attack() -> u8 {
    1
}

/// Inventory validation also protects stable weapon IDs used by commands.
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn validate_equipment(items: &[Item], base: &[Weapon], path: &str) -> Result<(), ContentError> {
    super::catalog::unique(items.iter().map(|i| i.id.as_str()), path)?;
    super::catalog::unique(
        base.iter()
            .chain(items.iter().flat_map(|i| &i.attacks))
            .map(|a| a.id.as_str()),
        path,
    )?;
    for item in items {
        item.validate(path)?;
    }
    Ok(())
}
