//! Bounded perception facts. Campaigns change these facts through ordinary actions;
//! sight queries never interpret scripts or campaign flags.
use super::{Hex, TerrainRole, TileRef, TurnPhase, UnitMatch};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(transparent, borrow))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
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
        let mut mask = Self::default();
        for &kind in kinds {
            mask.insert(kind);
        }
        mask
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SightExtent {
    #[default]
    Inherited,
    Unlimited,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SightRange {
    Extent(SightExtent),
    Range(u16),
}
impl Default for SightRange {
    fn default() -> Self {
        Self::Extent(SightExtent::Inherited)
    }
}
impl SightRange {
    pub const UNLIMITED: Self = Self::Extent(SightExtent::Unlimited);
    #[inline]
    pub fn resolve(self, inherited: Option<u16>) -> Option<u16> {
        match self {
            Self::Extent(SightExtent::Inherited) => inherited,
            Self::Extent(SightExtent::Unlimited) => None,
            Self::Range(n) => Some(n),
        }
    }
    pub fn concrete(self) -> bool {
        !matches!(self, Self::Extent(SightExtent::Inherited))
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SightProfile {
    pub range: SightRange,
    #[serde(default)]
    pub penetrates: Vec<ConcealmentKind>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SightEmission {
    pub group: String,
    pub profile: SightProfile,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentRule {
    pub kind: ConcealmentKind,
    pub reveal_within: u16,
    pub requires: ConcealmentRequirements,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentRequirements {
    #[serde(default)]
    pub terrains: Option<Vec<TerrainRole>>,
    #[serde(default)]
    pub illumination: Option<IlluminationRequirement>,
}
impl ConcealmentRule {
    #[inline]
    pub fn applies(&self, terrain: TerrainRole, illuminated: impl FnOnce() -> bool) -> bool {
        self.requires
            .terrains
            .as_ref()
            .is_none_or(|roles| roles.contains(&terrain))
            && self.requires.illumination.is_none_or(|required| {
                illuminated() == matches!(required, IlluminationRequirement::Lit)
            })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self
            .requires
            .terrains
            .as_ref()
            .is_some_and(|roles| roles.is_empty())
        {
            return Err("empty concealment terrain eligibility".into());
        }
        Ok(())
    }
}
impl Default for ConcealmentRequirements {
    fn default() -> Self {
        Self {
            terrains: None,
            illumination: None,
        }
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IlluminationRequirement {
    Lit,
    Unlit,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify)]
pub type UnitConcealment = ConcealmentRule;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentKindDefinition {
    pub id: ConcealmentKind,
    pub unit_appearance: ConcealmentVisual,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConcealmentVisual {
    None,
    Concealment,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitConcealmentAppearance {
    Normal,
    Concealed,
    Exposed,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitAppearance {
    pub unit: String,
    pub concealment: UnitConcealmentAppearance,
    /// Current-position detection by observers present in the faction's known view.
    pub detection: Vec<KnownDetection>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TilePerception {
    #[serde(default)]
    pub sight: Option<SightEmission>,
    #[serde(default)]
    pub remember: bool,
    #[serde(default)]
    pub concealment: Vec<ConcealmentRule>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LightEmission {
    pub radius: u16,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnitPerception {
    #[serde(default)]
    pub range: SightRange,
    #[serde(default)]
    pub penetrates: Vec<ConcealmentKind>,
    #[serde(default)]
    pub concealment: Vec<UnitConcealment>,
    #[serde(default)]
    pub light: Option<LightEmission>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisionGroupState {
    pub id: String,
    pub sight: SightProfile,
    pub explored: Vec<Hex>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TurnStamp {
    pub round: u32,
    pub phase: TurnPhase,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerceptionEffect {
    pub id: String,
    #[serde(default)]
    pub penetrates: Vec<ConcealmentKind>,
    #[serde(default)]
    pub concealment: Vec<UnitConcealment>,
    #[serde(default)]
    pub light: Option<LightEmission>,
    #[serde(default)]
    pub expires_at: Option<TurnStamp>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PerceptionArea {
    Map,
    Tiles(Vec<TileRef>),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentChange {
    pub area: PerceptionArea,
    #[serde(flatten)]
    pub rule: ConcealmentRule,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConcealmentRemoval {
    pub area: PerceptionArea,
    pub kind: ConcealmentKind,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerceptionGrant {
    pub units: UnitMatch,
    pub effect: PerceptionEffect,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerceptionRemoval {
    pub units: UnitMatch,
    pub effect: String,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerrainReveal {
    pub group: String,
    pub area: PerceptionArea,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisionGroupAssignment {
    pub faction: String,
    pub group: String,
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
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectionFacts {
    pub in_range: bool,
    pub revealed: bool,
    pub active: ConcealmentMask,
    pub blocking: ConcealmentMask,
}
/// Observers are disclosed only after passing through the faction's BattleView.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetectionObserver {
    Unit(String),
    Terrain(Hex),
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnownDetection {
    pub observer: DetectionObserver,
    pub facts: DetectionFacts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedSightProfile {
    pub range: Option<u16>,
    pub penetrates: ConcealmentMask,
}

/// One source must defeat every restriction. Capabilities are deliberately not
/// merged across sources; sharing happens only after a complete detection.
#[inline]
pub fn detects<'a>(
    profile: ResolvedSightProfile,
    distance: u16,
    rules: impl Iterator<Item = &'a ConcealmentRule>,
) -> DetectionFacts {
    let mut facts = DetectionFacts {
        in_range: profile.range.is_none_or(|range| distance <= range),
        ..Default::default()
    };
    for rule in rules {
        facts.active.insert(rule.kind);
        if distance > rule.reveal_within && !profile.penetrates.contains(rule.kind) {
            facts.blocking.insert(rule.kind);
        }
    }
    facts
}
impl DetectionFacts {
    #[inline]
    pub fn detected(self) -> bool {
        self.revealed || self.in_range && self.blocking == ConcealmentMask::default()
    }
}
impl PerceptionEffect {
    pub fn validate(&self) -> Result<(), String> {
        super::catalog::identifier(&self.id, "perception effect").map_err(|e| e.to_string())?;
        if self.expires_at.as_ref().is_some_and(|at| at.round == 0) {
            return Err("perception expiry requires a positive round".into());
        }
        validate_unit_perception(&self.concealment, &self.penetrates)
    }
}
pub fn validate_unit_perception(
    concealment: &[UnitConcealment],
    penetrates: &[ConcealmentKind],
) -> Result<(), String> {
    let mut seen = ConcealmentMask::default();
    for &kind in penetrates {
        if seen.contains(kind) {
            return Err("duplicate detection capability".into());
        }
        seen.insert(kind);
    }
    for c in concealment {
        c.validate()?;
    }
    Ok(())
}
pub fn validate_perception(
    groups: &[VisionGroupState],
    factions: &[super::Faction],
    tiles: &[super::Tile],
    units: &[super::PublicUnit],
) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for group in groups {
        super::catalog::identifier(&group.id, "vision group").map_err(|e| e.to_string())?;
        if !ids.insert(group.id.as_str()) || !group.sight.range.concrete() {
            return Err("vision groups require unique IDs and concrete sight ranges".into());
        }
        validate_unit_perception(&[], &group.sight.penetrates)?;
        let mut seen = std::collections::HashSet::new();
        if group
            .explored
            .iter()
            .any(|hex| !seen.insert(*hex) || !tiles.iter().any(|t| t.hex() == *hex))
        {
            return Err("invalid or duplicate explored hex".into());
        }
    }
    if factions
        .iter()
        .any(|f| !ids.contains(f.vision_group.as_str()))
    {
        return Err("unknown faction vision group".into());
    }
    for tile in tiles {
        for rule in &tile.perception.concealment {
            rule.validate()?;
        }
        if let Some(sight) = &tile.perception.sight {
            if !ids.contains(sight.group.as_str()) {
                return Err("unknown fixed sight source group".into());
            }
            validate_unit_perception(&[], &sight.profile.penetrates)?;
        }
    }
    for unit in units {
        let mut seen = std::collections::HashSet::new();
        if unit
            .revealed_to
            .iter()
            .any(|id| !ids.contains(id.as_str()) || !seen.insert(id))
        {
            return Err("unknown or duplicate combat reveal group".into());
        }
        let mut seen = std::collections::HashSet::new();
        for effect in &unit.perception_effects {
            if !seen.insert(&effect.id) {
                return Err("duplicate perception effect".into());
            }
            effect.validate()?;
        }
    }
    Ok(())
}
