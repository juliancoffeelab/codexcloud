//! ChapterProgram progress summaries and objectives supplied to planning policies.

use super::*;
use crate::CampaignProgram;
use crate::data::Text;
use crate::logic::campaign_program::ChapterProgram;
use serde::{Deserialize, Serialize};
/// A display-ready objective label, count, completion status and tile list.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub label: Text,
    pub current: u32,
    pub total: u32,
    pub done: bool,
    pub tiles: Vec<Hex>,
}
/// A weighted planning target with optional actor, family and defense constraints.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Objective {
    pub id: Text,
    pub tiles: Vec<Hex>,
    pub weight: f64,
    pub kind: ObjectiveKind,

    pub actor: Option<Text>,

    pub family: Option<Family>,

    pub hold: Option<bool>,
}

impl Objective {
    /// Return the stable serialized planning-objective tag.
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }
}

/// A resolved objective state for browser presentation.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
pub struct CampaignObjectives {
    pub goals: Vec<ObjectiveStatus>,
    pub optional: Vec<ObjectiveStatus>,
    pub survival: Vec<ObjectiveStatus>,
}
