use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisionGroupState {
    pub id: Text,
    pub sight: SightContribution,
    pub explored: Vec<Hex>,
}
/// Dense group identity is independent of phase, control and diplomatic relations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VisionGroupKey(pub usize);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PerceptionId(pub usize);
/// Indirect storage keeps the hot unit row small and Copy, like its inventory.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UnitPerceptionFacts {
    pub revealed_to: Vec<VisionGroupKey>,
    pub effects: Vec<PerceptionEffect>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainKnowledge {
    Unexplored,
    Remembered,
    Visible,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectionFacts {
    pub in_range: bool,
    pub revealed: bool,
    pub active: ConcealmentMask,
    pub blocking: ConcealmentMask,
}
/// Observers are disclosed only after passing through the faction's BattleView.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetectionObserver {
    Unit(Text),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnownDetection {
    pub observer: DetectionObserver,
    pub facts: DetectionFacts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedSightProfile {
    pub range: Option<u16>,
    pub penetrates: ConcealmentMask,
    pub modifiers: SightModifiers,
}
impl DetectionFacts {
    pub fn unobserved() -> Self {
        Self {
            in_range: false,
            revealed: false,
            active: ConcealmentMask::of(&[]),
            blocking: ConcealmentMask::of(&[]),
        }
    }

    #[inline]
    pub fn detected(self) -> bool {
        self.revealed || self.in_range && self.blocking == ConcealmentMask::of(&[])
    }
}
/// Observer-independent applicable cover. Penetration is evaluated separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConcealmentFacts {
    pub available: bool,
    pub reveal_distance: u16,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitConcealmentAppearance {
    Normal,
    Concealed,
    Exposed,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitAppearance {
    pub unit: Text,
    pub concealment: UnitConcealmentAppearance,
    /// Current-position detection by observers present in the faction's known view.
    pub detection: Vec<KnownDetection>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TilePerception {
    /// Resolved authored ambient illumination, independent of rendering metadata.
    #[serde(deserialize_with = "super::explicit_option")]
    pub illumination: Option<Illumination>,
    pub remember: bool,
    pub concealment: Vec<ConcealmentRule>,
}
