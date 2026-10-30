//! Typed authored content, immutable catalog data and owned battle snapshots.
pub use crate::storage::{Seed, Table};
use std::borrow::Cow;
mod ids;
mod recruitment;
pub use ids::*;
pub use recruitment::{RecruitmentProfiles, RecruitmentWeights};
mod commands;
pub use commands::*;
mod units;
pub use units::*;
mod campaign;
pub use campaign::*;
mod state;
pub use state::*;
mod scenario;
pub use scenario::*;
mod objectives;
pub use objectives::*;
mod records;
pub use records::*;
pub mod catalog;
pub use catalog::{
    ArenaDefaults, ArmyRoster, CampaignDefaults, ContentError, EnvironmentalModifiers, JsonBundle,
    LandscapeConfig, RaceDefinition, TerrainDefinition, TerrainModifier,
};
#[derive(Clone, Debug)]
/// Loaded, validated content with inheritance resolved into runtime tables.
///
/// A host owns this value and lends it to battle operations. Authored array order
/// determines the dense unit and terrain indices used inside each match.
pub struct Rules {
    /// SHA-256 of relative document paths and their exact JSON bytes.
    pub content_fingerprint: Cow<'static, str>,
    pub recruitment: RecruitmentProfiles,
    /// Ordered simulation recruitment rosters, independent of campaign rosters.
    pub rosters: Cow<'static, [ArmyRoster]>,
    /// Native arena defaults and landscape definitions.
    #[cfg(not(target_arch = "wasm32"))]
    pub arena: ArenaDefaults,
    /// Native campaign workload defaults.
    #[cfg(not(target_arch = "wasm32"))]
    pub campaign_defaults: CampaignDefaults,
    /// Terrain identifiers in dense runtime order.
    pub terrain_names: Cow<'static, [Cow<'static, str>]>,
    /// Closed terrain variants in the same runtime order.
    pub terrain_types: Cow<'static, [Terrain]>,
    /// Display labels corresponding to terrain identifiers.
    pub terrain_labels: Cow<'static, [Cow<'static, str>]>,
    /// Precomputed Unicode lowercase labels used in battle logs.
    pub unit_log_names: Cow<'static, [Cow<'static, str>]>,
    pub weapon_log_names: Cow<'static, [Cow<'static, [Cow<'static, str>]>]>,
    pub terrain_log_labels: Cow<'static, [Cow<'static, str>]>,
    /// Display metadata from the same validated definitions used by the engine.
    pub terrain_definitions: Cow<'static, [TerrainDefinition]>,
    /// Racial display metadata from the validated catalog.
    pub race_definitions: Cow<'static, [RaceDefinition]>,
    /// Current authored map descriptions and layouts.
    pub map_definitions: Cow<'static, [Seed<MapDefinition>]>,
    /// Effective unit rules after terrain, race and unit inheritance.
    pub units: Cow<'static, [UnitRule]>,
    pub movement_classes: Cow<'static, [usize]>,
    pub route_classes: Cow<'static, [usize]>,
    /// Authored unit identifiers mapped to dense runtime indices.
    pub types: Table<Cow<'static, str>, usize>,
    /// Authored terrain identifiers mapped to dense runtime indices.
    pub terrains: Table<Cow<'static, str>, usize>,
    /// Fresh initial states indexed by authored map and revision.
    pub maps: Table<(Cow<'static, str>, u32), Seed<State>>,
    /// Arena layouts kept outside playable scenarios and map metadata.
    #[cfg(not(target_arch = "wasm32"))]
    pub arena_maps: Table<(Cow<'static, str>, u32), Seed<State>>,
    /// Campaign definitions indexed by their authored identifier.
    pub campaigns: Table<Cow<'static, str>, Campaign>,
}

/// Immutable authored content shared by battles and scoped simulation workers.
pub type Catalog = Rules;

mod templates;
pub use templates::*;
