//! Persistent campaign progress, companions and dialogue state.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// One scripted dialogue line with its speaker, portrait and display text.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialogue {
    pub speaker: String,
    pub portrait: String,
    pub text: String,
}
/// Consecutive defended turns and the last round counted for an objective.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counter {
    pub held_turns: u32,
    pub last_counted_round: u32,
}
/// Defended refuge turns and whether mediation has escalated into battle.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefugeCounter {
    pub held_turns: u32,
    pub last_counted_round: u32,
    pub escalated: bool,
}
/// Persistent chapter progress, recalled companions, story choices and dialogue.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignState {
    pub id: String,
    pub mission: usize,
    pub recall: Vec<String>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub veterans: Option<Vec<Veteran>>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_veterans: Option<Vec<Veteran>>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallen_companions: Option<Vec<String>>,
    pub carry_gold: i32,
    pub marks: Vec<String>,
    pub dialogue: Vec<Dialogue>,
    pub dialogue_index: usize,
    pub outcome: Option<String>,
    pub completed: bool,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refuge: Option<RefugeCounter>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vigil: Option<Counter>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holds: Option<BTreeMap<String, Counter>>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
}
impl CampaignState {
    /// Check whether a once-only event or campaign marker has already been recorded.
    pub fn marked(&self, mark: &str) -> bool {
        self.marks.iter().any(|m| m == mark)
    }
    pub(crate) fn marked_prefixed(&self, prefix: &str, value: &str) -> bool {
        self.marks
            .iter()
            .any(|mark| mark.strip_prefix(prefix) == Some(value))
    }
    /// Check whether the persistent story choices contain this identifier.
    pub fn chosen(&self, choice: &str) -> bool {
        self.choices
            .as_ref()
            .is_some_and(|list| list.iter().any(|c| c == choice))
    }
    /// Report whether refuge mediation has escalated into battle.
    pub fn escalated(&self) -> bool {
        self.refuge.as_ref().is_some_and(|c| c.escalated)
    }
    /// Append scripted dialogue while preserving the current reading position.
    pub fn say(&mut self, lines: impl IntoIterator<Item = Dialogue>) {
        self.dialogue.extend(lines);
    }
}
