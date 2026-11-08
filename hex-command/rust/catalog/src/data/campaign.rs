//! Persistent campaign progress, companions and dialogue state.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
/// Stable campaign-local identity; display names are presentation data.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Character {
    pub id: String,
    pub name: String,
}
/// One scripted dialogue line with its speaker, portrait and display text.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Dialogue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(default)]
    pub speaker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    pub portrait: String,
    pub text: String,
}
/// Consecutive defended turns and the last round counted for an objective.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Counter {
    pub held_turns: u32,
    pub last_counted_round: u32,
}
/// Defended refuge turns and whether mediation has escalated into battle.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct RefugeCounter {
    pub held_turns: u32,
    pub last_counted_round: u32,
    pub escalated: bool,
}
/// A unit safely removed from the board, retaining identity and belongings.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct EvacuatedUnit {
    pub event: String,
    pub unit: PublicUnit,
}
/// One explicit flag change. Stage uses the campaign's zero-based mission index.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct FlagChange {
    pub state: bool,
    pub stage: usize,
    pub turn: u32,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify)]
pub type FlagHistory = HashMap<String, Vec<FlagChange>>;

/// Persistent chapter progress, recalled companions, story choices and dialogue.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct CampaignState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<EventState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evacuated: Vec<EvacuatedUnit>,
    pub id: String,
    pub mission: usize,
    pub recall: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub veterans: Option<Vec<Veteran>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub story_veterans: Option<Vec<Veteran>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallen_companions: Option<Vec<String>>,
    pub carry_gold: i32,
    pub marks: Vec<String>,
    pub dialogue: Vec<Dialogue>,
    pub dialogue_index: usize,
    pub outcome: Option<String>,
    pub completed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refuge: Option<RefugeCounter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vigil: Option<Counter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holds: Option<BTreeMap<String, Counter>>,
    pub flags: HashMap<String, Vec<FlagChange>>,
}
impl CampaignState {
    /// Check whether a once-only event or campaign marker has already been recorded.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn marked(&self, mark: &str) -> bool {
        self.marks.iter().any(|m| m == mark)
    }
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn marked_prefixed(&self, prefix: &str, value: &str) -> bool {
        self.marks
            .iter()
            .any(|mark| mark.strip_prefix(prefix) == Some(value))
    }
    /// The last change is authoritative; a missing history means false.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn chosen(&self, id: &str) -> bool {
        flag_is_set(&self.flags, id)
    }
    pub fn set_flag(&mut self, id: &str, state: bool, turn: u32) {
        if self.chosen(id) == state {
            return;
        }
        self.flags.entry(id.into()).or_default().push(FlagChange {
            state,
            stage: self.mission,
            turn,
        });
    }
    /// Turn zero denotes incoming facts whose historical timestamp is unavailable.
    pub fn retry_flags(&self) -> FlagHistory {
        self.flags
            .iter()
            .filter_map(|(id, history)| {
                let prior: Vec<_> = history
                    .iter()
                    .take_while(|v| {
                        v.stage < self.mission || (v.stage == self.mission && v.turn == 0)
                    })
                    .cloned()
                    .collect();
                (!prior.is_empty()).then_some((id.clone(), prior))
            })
            .collect()
    }
    /// Report whether refuge mediation has escalated into battle.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn escalated(&self) -> bool {
        self.refuge.as_ref().is_some_and(|c| c.escalated)
    }
    /// Append scripted dialogue while preserving the current reading position.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn say(&mut self, lines: impl IntoIterator<Item = Dialogue>) {
        self.dialogue.extend(lines);
    }
}

/// Read campaign outcomes without borrowing unrelated saved event progress.
#[inline]
pub fn flag_is_set(flags: &FlagHistory, id: &str) -> bool {
    flags
        .get(id)
        .and_then(|h| h.last())
        .is_some_and(|v| v.state)
}
