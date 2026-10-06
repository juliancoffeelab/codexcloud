//! Mission progress summaries and objectives supplied to planning policies.

use super::*;
use serde::{Deserialize, Serialize};
/// A display-ready objective label, count, completion status and tile list.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub label: String,
    pub current: u32,
    pub total: u32,
    pub done: bool,
    pub tiles: Vec<String>,
}
/// A weighted planning target with optional actor, family and defense constraints.
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Objective {
    pub id: String,
    pub tiles: Vec<String>,
    pub weight: f64,
    pub kind: ObjectiveKind,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<Family>,
    #[cfg_attr(feature = "bindings", ts(optional))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
}

impl Objective {
    /// Return the stable serialized planning-objective tag.
    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }
}
