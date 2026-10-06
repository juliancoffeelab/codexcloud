//! Stable identifiers and closed vocabularies shared by content and battle APIs.

use serde::{Deserialize, Serialize};
/// Stable unit identity independent of roster order.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitId(pub u32);
/// Authored tile identifier.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TileId(pub String);
impl From<&str> for TileId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored unittype identifier.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitTypeId(pub String);
impl From<&str> for UnitTypeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored weapon identifier.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponId(pub String);
impl From<&str> for WeaponId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored map identifier.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MapId(pub String);
impl From<&str> for MapId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored campaign identifier.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CampaignId(pub String);
impl From<&str> for CampaignId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Closed authored race vocabulary.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Race {
    Human,
    Merfolk,
    Undead,
    Beast,
}
impl Race {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Merfolk => "merfolk",
            Self::Undead => "undead",
            Self::Beast => "beast",
        }
    }
}
impl std::fmt::Display for Race {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed authored family vocabulary.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    Adept,
    Archer,
    Boar,
    Bonearcher,
    Commander,
    Harpooner,
    Necromancer,
    Reefguard,
    Scout,
    Skeleton,
    Spearman,
    Swordsman,
    Thorncaller,
    Tidekeeper,
    Wavemender,
    Wildkeeper,
    Wolf,
}
impl Family {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Adept => "adept",
            Self::Archer => "archer",
            Self::Boar => "boar",
            Self::Bonearcher => "bonearcher",
            Self::Commander => "commander",
            Self::Harpooner => "harpooner",
            Self::Necromancer => "necromancer",
            Self::Reefguard => "reefguard",
            Self::Scout => "scout",
            Self::Skeleton => "skeleton",
            Self::Spearman => "spearman",
            Self::Swordsman => "swordsman",
            Self::Thorncaller => "thorncaller",
            Self::Tidekeeper => "tidekeeper",
            Self::Wavemender => "wavemender",
            Self::Wildkeeper => "wildkeeper",
            Self::Wolf => "wolf",
        }
    }
}
impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed authored terrain vocabulary.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Terrain {
    Plains,
    Forest,
    Hill,
    Water,
    Village,
    Keep,
    Grove,
    Fort,
    Ruins,
    Marsh,
    Bridge,
    Ford,
}
impl Terrain {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plains => "plains",
            Self::Forest => "forest",
            Self::Hill => "hill",
            Self::Water => "water",
            Self::Village => "village",
            Self::Keep => "keep",
            Self::Grove => "grove",
            Self::Fort => "fort",
            Self::Ruins => "ruins",
            Self::Marsh => "marsh",
            Self::Bridge => "bridge",
            Self::Ford => "ford",
        }
    }
}
impl std::fmt::Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed authored role vocabulary.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Leader,
    Courier,
}
impl Role {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Leader => "leader",
            Self::Courier => "courier",
        }
    }
}
impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kind of objective selected by the planner.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectiveKind {
    Capture,
    Defeat,
    Visit,
    Seal,
    Escort,
    Escape,
    Vigil,
}
impl ObjectiveKind {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Defeat => "defeat",
            Self::Visit => "visit",
            Self::Seal => "seal",
            Self::Escort => "escort",
            Self::Escape => "escape",
            Self::Vigil => "vigil",
        }
    }
}

/// Structural campaign objective category.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalKind {
    Defeat,
    Escort,
    Escape,
    Control,
    Seal,
    Event,
    Hold,
    Vigil,
    Refuge,
}
impl GoalKind {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Defeat => "defeat",
            Self::Escort => "escort",
            Self::Escape => "escape",
            Self::Control => "control",
            Self::Seal => "seal",
            Self::Event => "event",
            Self::Hold => "hold",
            Self::Vigil => "vigil",
            Self::Refuge => "refuge",
        }
    }
}
/// Structural campaign event activation category.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerKind {
    Visit,
    Round,
    Control,
}
impl TriggerKind {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Visit => "visit",
            Self::Round => "round",
            Self::Control => "control",
        }
    }
}

/// Movement spent along the cheapest legal path, in authored movement points.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MovementCost(pub u16);
