//! Typed authored content, immutable catalog data and owned battle snapshots.
use std::collections::HashMap;
mod ids;
mod recruitment;
pub use ids::*;
pub use recruitment::{RecruitmentProfiles, RecruitmentWeights};
mod commands;
pub use commands::*;
mod factions;
pub use factions::*;
mod units;
pub use units::*;
mod campaign;
pub use campaign::*;
mod state;
pub use state::*;
mod scenario;
pub use scenario::*;
mod lighting;
pub use lighting::*;
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
    pub content_fingerprint: String,
    pub recruitment: RecruitmentProfiles,
    /// Ordered simulation recruitment rosters, independent of campaign rosters.
    pub rosters: Vec<ArmyRoster>,
    /// Native arena defaults and landscape definitions.
    pub arena: ArenaDefaults,
    /// Native campaign workload defaults.
    pub campaign_defaults: CampaignDefaults,
    /// Terrain identifiers in dense runtime order.
    pub terrain_names: Vec<String>,
    /// Closed terrain variants in the same runtime order.
    pub terrain_types: Vec<Terrain>,
    /// Dense gameplay roles from explicit authored terrain definitions.
    pub terrain_roles: Vec<TerrainRole>,
    /// Display labels corresponding to terrain identifiers.
    pub terrain_labels: Vec<String>,
    /// Display metadata from the same validated definitions used by the engine.
    pub terrain_definitions: Vec<TerrainDefinition>,
    /// Racial display metadata from the validated catalog.
    pub race_definitions: Vec<RaceDefinition>,
    /// Current authored map descriptions and layouts.
    pub map_definitions: Vec<MapDefinition>,
    /// Effective unit rules after terrain, race and unit inheritance.
    pub units: Vec<UnitRule>,
    pub(crate) movement_classes: Vec<usize>,
    pub(crate) route_classes: Vec<usize>,
    /// Authored unit identifiers mapped to dense runtime indices.
    pub types: HashMap<String, usize>,
    /// Authored terrain identifiers mapped to dense runtime indices.
    pub terrains: HashMap<String, usize>,
    /// Fresh initial states indexed by authored map and revision.
    pub maps: HashMap<(String, u32), State>,
    /// Arena layouts kept outside playable scenarios and map metadata.
    pub arena_maps: HashMap<(String, u32), State>,
    /// Campaign definitions indexed by their authored identifier.
    pub campaigns: HashMap<String, Campaign>,
}

/// Immutable authored content shared by battles and scoped simulation workers.
pub type Catalog = Rules;

mod conditions;
pub use conditions::*;
