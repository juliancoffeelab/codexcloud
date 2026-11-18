use super::*;
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum ConcealmentKind {
    Fog,
    Darkness,
    ForestCamouflage,
}
impl ConcealmentKind {
    pub const ALL: [Self; 3] = [Self::Fog, Self::Darkness, Self::ForestCamouflage];
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }
}
/// Fixed mask used only after resolving authored capabilities.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConcealmentMask(u8);
impl ConcealmentMask {
    #[inline]
    pub fn contains(self, kind: ConcealmentKind) -> bool {
        self.0 & (1 << kind.index()) != 0
    }
    #[inline]
    pub fn insert(&mut self, kind: ConcealmentKind) {
        self.0 |= 1 << kind.index();
    }
    #[inline]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    #[inline]
    pub fn of(kinds: &[ConcealmentKind]) -> Self {
        let mut mask = Self(0);
        for &kind in kinds {
            mask.insert(kind);
        }
        mask
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentRule {
    pub kind: ConcealmentKind,
    pub reveal_within: u16,
    pub requires: ConcealmentRequirements,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentRequirements {
    #[serde(deserialize_with = "super::explicit_option")]
    pub terrains: Option<Vec<TerrainRole>>,
    #[serde(deserialize_with = "super::explicit_option")]
    pub illumination: Option<Vec<Illumination>>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Illumination {
    Lit,
    Unlit,
    Darkness,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentKindDefinition {
    pub id: ConcealmentKind,
    pub unit_appearance: ConcealmentVisual,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConcealmentVisual {
    None,
    Concealment,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightEmission {
    pub radius: u16,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerceptionEffect {
    pub id: Text,
    pub penetrates: Vec<ConcealmentKind>,
    pub concealment: Vec<ConcealmentRule>,
    #[serde(deserialize_with = "super::explicit_option")]
    pub light: Option<LightEmission>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PerceptionArea {
    Map,
    Tiles(Vec<TileRef>),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentChange {
    pub area: PerceptionArea,
    #[serde(flatten)]
    pub rule: ConcealmentRule,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentRemoval {
    pub area: PerceptionArea,
    pub kind: ConcealmentKind,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerceptionGrant {
    pub units: UnitMatch,
    pub effect: PerceptionEffect,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerceptionRemoval {
    pub units: UnitMatch,
    pub effect: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerrainReveal {
    pub group: Text,
    pub area: PerceptionArea,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisionGroupAssignment {
    pub faction: Text,
    pub group: Text,
}
/// Catalog-only adjustments based on illumination at the target, not the observer.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SightModifiers {
    pub lit: i16,
    pub unlit: i16,
    pub darkness: i16,
}
