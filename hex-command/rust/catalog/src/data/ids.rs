//! Stable identifiers and closed vocabularies shared by content and battle APIs.

use crate::data::Text;
use serde::{Deserialize, Serialize};
/// Stable unit identity independent of roster order.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct UnitId(pub u32);
/// A geometric cell address. Never an authored location name.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Hex {
    pub col: i32,
    pub row: i32,
}
impl Hex {
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }
}
impl std::fmt::Display for Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{}", self.col, self.row)
    }
}
/// An explicit authored reference. Runtime positions never use this type.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum TileRef {
    Location(Text),
    Hex(Hex),
    Binding(TileBinding),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum TileBinding {
    Unit,
    Mark,
    Death,
}
impl std::fmt::Display for TileRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
/// Geometric command address, separate from authored references.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TileId(pub Hex);
impl From<Hex> for TileId {
    fn from(value: Hex) -> Self {
        Self(value)
    }
}
/// Authored unittype identifier.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct UnitTypeId(pub Text);
impl From<&str> for UnitTypeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Authored weapon identifier.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct WeaponId(pub Text);
impl From<&str> for WeaponId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Closed authored race vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Race {
    Human,
    Merfolk,
    Skeletons,
    Spirit,
    Beast,
    Zombies,
}
impl Race {
    /// Return the stable authored identifier used at serialization boundaries.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Merfolk => "merfolk",
            Self::Skeletons => "skeletons",
            Self::Spirit => "spirit",
            Self::Beast => "beast",
            Self::Zombies => "zombies",
        }
    }
}
impl std::fmt::Display for Race {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed authored family vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Family {
    #[serde(rename = "zombie-walker")]
    ZombieWalker,
    #[serde(rename = "zombie-decay-caster")]
    ZombieDecayCaster,
    Captives,
    Bear,
    #[serde(rename = "giant-bat")]
    GiantBat,
    #[serde(rename = "giant-spider")]
    GiantSpider,
    #[serde(rename = "giant-snake")]
    GiantSnake,
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
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Captives => "captives",
            Self::ZombieWalker => "zombie-walker",
            Self::ZombieDecayCaster => "zombie-decay-caster",
            Self::Bear => "bear",
            Self::GiantBat => "giant-bat",
            Self::GiantSpider => "giant-spider",
            Self::GiantSnake => "giant-snake",
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

/// Gameplay meaning of terrain, independent of its authored/artwork identifier.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum TerrainRole {
    Bridge,
    BurnedVillage,
    MountainMine,
    CaveWall,
    Church,
    Ferry,
    Ford,
    Forest,
    Fort,
    Fortification,
    Graveyard,
    Grove,
    Hill,
    House,
    Keep,
    Landing,
    Marsh,
    Monastery,
    Mountain,
    Plains,
    RitualSpent,
    RitualStar,
    Road,
    Ruins,
    Signpost,
    Village,
    Water,
}

impl TerrainRole {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn capturable(self) -> bool {
        match self {
            Self::Fort
            | Self::Grove
            | Self::Keep
            | Self::Monastery
            | Self::Ruins
            | Self::Village => true,
            Self::Bridge
            | Self::BurnedVillage
            | Self::MountainMine
            | Self::CaveWall
            | Self::Church
            | Self::Ferry
            | Self::Ford
            | Self::Forest
            | Self::Fortification
            | Self::Graveyard
            | Self::Hill
            | Self::House
            | Self::Landing
            | Self::Marsh
            | Self::Mountain
            | Self::Plains
            | Self::RitualSpent
            | Self::RitualStar
            | Self::Road
            | Self::Signpost
            | Self::Water => false,
        }
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn blocks_shots(self) -> bool {
        match self {
            Self::MountainMine
            | Self::CaveWall
            | Self::Forest
            | Self::Grove
            | Self::Hill
            | Self::Keep
            | Self::Mountain => true,
            Self::Bridge
            | Self::BurnedVillage
            | Self::Church
            | Self::Ferry
            | Self::Ford
            | Self::Fort
            | Self::Fortification
            | Self::Graveyard
            | Self::House
            | Self::Landing
            | Self::Marsh
            | Self::Monastery
            | Self::Plains
            | Self::RitualSpent
            | Self::RitualStar
            | Self::Road
            | Self::Ruins
            | Self::Signpost
            | Self::Village
            | Self::Water => false,
        }
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn income(self) -> i32 {
        match self {
            Self::Village => 2,
            Self::Bridge
            | Self::BurnedVillage
            | Self::MountainMine
            | Self::CaveWall
            | Self::Church
            | Self::Ferry
            | Self::Ford
            | Self::Forest
            | Self::Fort
            | Self::Fortification
            | Self::Graveyard
            | Self::Grove
            | Self::Hill
            | Self::House
            | Self::Keep
            | Self::Landing
            | Self::Marsh
            | Self::Monastery
            | Self::Mountain
            | Self::Plains
            | Self::RitualSpent
            | Self::RitualStar
            | Self::Road
            | Self::Ruins
            | Self::Signpost
            | Self::Water => 0,
        }
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn healing(self) -> i32 {
        match self {
            Self::Grove | Self::Village => 4,
            Self::Bridge
            | Self::BurnedVillage
            | Self::MountainMine
            | Self::CaveWall
            | Self::Church
            | Self::Ferry
            | Self::Ford
            | Self::Forest
            | Self::Fort
            | Self::Fortification
            | Self::Graveyard
            | Self::Hill
            | Self::House
            | Self::Keep
            | Self::Landing
            | Self::Marsh
            | Self::Monastery
            | Self::Mountain
            | Self::Plains
            | Self::RitualSpent
            | Self::RitualStar
            | Self::Road
            | Self::Ruins
            | Self::Signpost
            | Self::Water => 0,
        }
    }
}

/// Closed authored terrain vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Terrain {
    #[serde(rename = "cave-passage")]
    CavePassage,
    #[serde(rename = "cave-wall")]
    CaveWall,
    Monastery,
    Church,
    Graveyard,
    #[serde(rename = "ritual-star")]
    RitualStar,
    #[serde(rename = "ritual-spent")]
    RitualSpent,
    #[serde(rename = "castle-bell")]
    CastleBell,
    #[serde(rename = "castle-wall")]
    CastleWall,
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
    #[serde(rename = "mountain-mine")]
    MountainMine,
    #[serde(rename = "burned-village")]
    BurnedVillage,
    Signpost,
}
impl Terrain {
    /// Return the stable authored identifier used at serialization boundaries.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Monastery => "monastery",
            Self::Church => "church",
            Self::Graveyard => "graveyard",
            Self::RitualStar => "ritual-star",
            Self::RitualSpent => "ritual-spent",
            Self::CastleBell => "castle-bell",
            Self::CastleWall => "castle-wall",
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
            Self::MountainMine => "mountain-mine",
            Self::CavePassage => "cave-passage",
            Self::CaveWall => "cave-wall",
            Self::BurnedVillage => "burned-village",
            Self::Signpost => "signpost",
        }
    }
}
impl std::fmt::Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Closed authored role vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Role {
    Leader,
    Courier,
    Captive,
}
impl Role {
    pub fn is_escort(self) -> bool {
        matches!(self, Self::Courier | Self::Captive)
    }
    /// Return the stable authored identifier used at serialization boundaries.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Leader => "leader",
            Self::Courier => "courier",
            Self::Captive => "captive",
        }
    }
}
impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kind of objective selected by the planner.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum TriggerKind {
    Held,
    Condition,
    Visit,
    Round,
    Control,
}
impl TriggerKind {
    /// Return the stable authored identifier used at serialization boundaries.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Held => "held",
            Self::Condition => "condition",
            Self::Visit => "visit",
            Self::Round => "round",
            Self::Control => "control",
        }
    }
}
