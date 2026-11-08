//! Mission progress summaries and objectives supplied to planning policies.

use super::*;
use serde::{Deserialize, Serialize};
/// A display-ready objective label, count, completion status and tile list.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Progress {
    pub label: String,
    pub current: u32,
    pub total: u32,
    pub done: bool,
    pub tiles: Vec<Hex>,
}
/// A weighted planning target with optional actor, family and defense constraints.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Objective {
    pub id: String,
    pub tiles: Vec<Hex>,
    pub weight: f64,
    pub kind: ObjectiveKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<Family>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

impl Objective {
    /// Return the stable serialized planning-objective tag.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }
}

/// A resolved objective state for browser presentation.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct ObjectiveStatus {
    /// Resolved battlefield marker destinations, including partial completion.
    #[serde(rename = "activeTiles")]
    pub active_tiles: Vec<Hex>,
    pub visible: bool,
    pub available: bool,
    pub progress: Progress,
    pub failed: bool,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CampaignObjectives {
    pub goals: Vec<ObjectiveStatus>,
    pub optional: Vec<ObjectiveStatus>,
    pub survival: Vec<ObjectiveStatus>,
}
