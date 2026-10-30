//! Stable identifiers and closed vocabularies shared by content and battle APIs.

use serde::{Deserialize, Serialize};
/// Stable unit identity independent of roster order.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitId(pub u32);
/// Authored tile identifier.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TileId(pub String);
impl From<&str> for TileId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored unittype identifier.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitTypeId(pub String);
impl From<&str> for UnitTypeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored weapon identifier.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeaponId(pub String);
impl From<&str> for WeaponId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Closed authored race vocabulary.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Race {
    Human,
    Merfolk,
    Skeletons,
    Spirit,
    Beast,
}
impl Race {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Merfolk => "merfolk",
            Self::Skeletons => "skeletons",
            Self::Spirit => "spirit",
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    Bear,
    #[serde(rename = "giant-bat")]
    GiantBat,
    #[serde(rename = "giant-spider")]
    GiantSpider,
    Adept,
    Archer,
    Boar,
    Bonearcher,
    Commander,
    Harpooner,
    #[serde(rename = "horse-archer")]
    HorseArcher,
    #[serde(rename = "skeleton-rider")]
    SkeletonRider,
    #[serde(rename = "skeleton-axeman")]
    SkeletonAxeman,
    #[serde(rename = "fox-spearguard")]
    FoxSpearguard,
    Treant,
    #[serde(rename = "merfolk-maceguard")]
    MerfolkMacer,
    Necromancer,
    Reefguard,
    Rider,
    Scout,
    Ghost,
    Skeleton,
    #[serde(rename = "skeleton-impact-caster")]
    SkeletonMage,
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
            Self::Bear => "bear",
            Self::GiantBat => "giant-bat",
            Self::GiantSpider => "giant-spider",
            Self::Adept => "adept",
            Self::Archer => "archer",
            Self::Boar => "boar",
            Self::Bonearcher => "bonearcher",
            Self::Commander => "commander",
            Self::Harpooner => "harpooner",
            Self::HorseArcher => "horse-archer",
            Self::SkeletonRider => "skeleton-rider",
            Self::SkeletonAxeman => "skeleton-axeman",
            Self::FoxSpearguard => "fox-spearguard",
            Self::Treant => "treant",
            Self::MerfolkMacer => "merfolk-maceguard",
            Self::Necromancer => "necromancer",
            Self::Reefguard => "reefguard",
            Self::Rider => "rider",
            Self::Scout => "scout",
            Self::Ghost => "ghost",
            Self::Skeleton => "skeleton",
            Self::SkeletonMage => "skeleton-impact-caster",
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Terrain {
    Plains,
    Road,
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
    Landing,
    Ferry,
    House,
    Mountain,
    Cave,
}
impl Terrain {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plains => "plains",
            Self::Road => "road",
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
            Self::Landing => "landing",
            Self::Ferry => "ferry",
            Self::House => "house",
            Self::Mountain => "mountain",
            Self::Cave => "cave",
        }
    }
}
impl std::fmt::Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed authored role vocabulary.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerKind {
    Condition,
    Visit,
    Round,
    Control,
}
impl TriggerKind {
    /// Return the stable authored identifier used at serialization boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Condition => "condition",
            Self::Visit => "visit",
            Self::Round => "round",
            Self::Control => "control",
        }
    }
}
