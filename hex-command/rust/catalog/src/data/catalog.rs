//! Runtime catalog definitions shared by baked content and native authoring tools.
use super::*;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, fmt};

/// Relative document paths mapped to their complete UTF-8 JSON source.
pub type JsonBundle = BTreeMap<String, String>;

#[derive(Debug)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
/// A catalog error carrying the authoring document and a concrete diagnostic.
pub struct ContentError {
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(deny_unknown_fields)]
/// An arena roster, authored independently of story campaign recruitment.
pub struct ArmyRoster {
    pub id: Cow<'static, str>,
    pub leader: Cow<'static, str>,
    pub pool: Cow<'static, [Cow<'static, str>]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A generated arena landscape and its required territory hold duration.
pub struct LandscapeConfig {
    pub id: Cow<'static, str>,
    pub hold_turns: u32,
    /// Optional authored scenario layout, shared with the playable map.
    pub map: Option<Cow<'static, str>>,
    /// Capture targets for this layout; omitted uses the generated-arena defaults.
    pub territory: Option<Cow<'static, [Cow<'static, str>]>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored arena dimensions and default per-match execution limits.
pub struct ArenaDefaults {
    pub landscapes: Cow<'static, [LandscapeConfig]>,
    pub territory: Cow<'static, [Cow<'static, str>]>,
    /// Minimum total gold value per side, including its leader.
    pub starting_budget: i32,
    pub default_max_rounds: u32,
    pub policies: Cow<'static, [Cow<'static, str>]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Campaign schedule dimensions and default attempt execution limits.
pub struct CampaignDefaults {
    pub default_max_rounds: u32,
    pub diplomacy: Cow<'static, [Cow<'static, str>]>,
    pub policies: Cow<'static, [Cow<'static, str>]>,
    pub campaigns: Cow<'static, [Cow<'static, str>]>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(default, deny_unknown_fields)]
/// Category and damage-type contributions; both apply to an incoming hit.
pub struct EnvironmentalModifiers {
    pub melee: i32,
    pub ranged: i32,
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A race or unit deviation from base terrain movement and damage rules.
pub struct TerrainModifier {
    #[serde(default)]
    pub cost: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passable: Option<bool>,
    #[serde(default)]
    pub damage_modifiers: EnvironmentalModifiers,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored terrain defaults, also exposed to the browser for display.
pub struct TerrainDefinition {
    pub id: Terrain,
    pub name: Cow<'static, str>,
    pub cost: u16,
    pub passable: bool,
    pub description: Cow<'static, str>,
    #[serde(default)]
    pub damage_modifiers: EnvironmentalModifiers,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::data))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored racial defaults inherited by every unit of that race.
pub struct RaceDefinition {
    pub id: Race,
    pub name: Cow<'static, str>,
    #[cfg_attr(feature = "wasm", tsify(type = "Record<Terrain, TerrainModifier>"))]
    pub terrain_modifiers: Table<Terrain, TerrainModifier>,
    pub damage_modifiers: DamageModifiers,
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

impl std::error::Error for ContentError {}

#[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
#[path = "catalog_loader.rs"]
mod loader;
#[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
pub use loader::content_fingerprint;
#[cfg(not(target_arch = "wasm32"))]
pub use loader::load_bundle_from;
