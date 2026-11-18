use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
/// Relative document paths mapped to their complete UTF-8 JSON source.
pub type JsonBundle = BTreeMap<Text, Text>;
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Debug)]
/// A catalog error carrying the authoring document and a concrete diagnostic.
pub struct ContentError {
    pub path: Text,
    pub message: Text,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Category and damage-type contributions; both apply to an incoming hit.
pub struct EnvironmentalModifiers {
    pub melee: i32,
    pub ranged: i32,
    pub blade: i32,
    pub pierce: i32,
    pub impact: i32,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// A race or unit deviation from base terrain movement and damage rules.
pub struct TerrainStatModifiers {
    pub cost: i32,
    #[serde(deserialize_with = "super::explicit_option")]
    pub passable: Option<bool>,
    pub damage_modifiers: EnvironmentalModifiers,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored terrain source, composed by the battle Catalogue for mechanical use.
pub struct TerrainDefinition {
    pub capturable: bool,
    pub blocks_shots: bool,
    pub income: i32,
    pub effects: Vec<ScopedEffect>,
    pub role: TerrainRole,
    pub id: Terrain,
    pub name: Text,
    pub cost: u16,
    pub passable: bool,
    pub description: Text,
    pub damage_modifiers: EnvironmentalModifiers,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Authored racial contributions stacked with unit and active effects.
pub struct RaceDefinition {
    pub id: Race,
    pub name: Text,
    pub abilities: Vec<Text>,
    pub sight: SightContribution,
    pub terrain_modifiers: HashMap<TerrainRole, TerrainStatModifiers>,
    pub damage_modifiers: DamageModifiers,
}
impl ContentError {
    pub fn new(path: impl Into<Text>, message: impl Into<Text>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}
impl std::error::Error for ContentError {}
