use super::Text;
use serde::{Deserialize, Serialize};
/// A geometric cell address. Never an authored location name.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
pub enum TileRef {
    Location(Text),
    Hex(Hex),
    Binding(TileBinding),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
/// Authored unittype identifier.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnitTypeId(pub Text);
impl From<&str> for UnitTypeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
/// Closed authored race vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
/// Closed authored terrain vocabulary.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    #[serde(rename = "shallow-water")]
    ShallowWater,
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
            Self::ShallowWater => "shallow-water",
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
