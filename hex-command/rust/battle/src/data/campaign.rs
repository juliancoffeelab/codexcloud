use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
/// One scripted dialogue line with its speaker, portrait and display text.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dialogue {
    pub sound: Option<Text>,
    pub speaker: Text,
    pub character: Option<Text>,
    pub portrait: Text,
    pub text: Text,
}
/// Consecutive defended turns and the last round counted for an objective.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counter {
    pub held_turns: u32,
    pub last_counted_round: u32,
}
/// Defended refuge turns and whether mediation has escalated into battle.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefugeCounter {
    pub held_turns: u32,
    pub last_counted_round: u32,
    pub escalated: bool,
}
/// A unit safely removed from the board, retaining identity and belongings.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvacuatedUnit {
    pub event: Text,
    pub unit: PublicUnit,
}
/// One explicit flag change. Stage uses the campaign's zero-based mission index.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlagChange {
    pub state: bool,
    pub stage: usize,
    pub turn: u32,
}
pub type FlagHistory = HashMap<Text, Vec<FlagChange>>;
/// Persistent chapter progress, recalled companions, story choices and dialogue.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignState {
    pub scripts: Option<EventState>,
    pub evacuated: Vec<EvacuatedUnit>,
    pub id: Text,
    pub mission: usize,
    pub recall: Vec<Text>,
    pub veterans: Option<Vec<Veteran>>,
    pub story_veterans: Option<Vec<Veteran>>,
    pub fallen_companions: Option<Vec<Text>>,
    pub carry_gold: i32,
    pub marks: Vec<Text>,
    pub dialogue: Vec<Dialogue>,
    pub dialogue_index: usize,
    pub outcome: Option<Text>,
    pub completed: bool,
    pub refuge: Option<RefugeCounter>,
    pub vigil: Option<Counter>,
    pub holds: Option<BTreeMap<Text, Counter>>,
    pub flags: HashMap<Text, Vec<FlagChange>>,
}
impl CampaignState {
    /// Check whether a once-only event or campaign marker has already been recorded.
    pub fn marked(&self, mark: &str) -> bool {
        self.marks.iter().any(|m| m == mark)
    }
    pub fn marked_prefixed(&self, prefix: &str, value: &str) -> bool {
        self.marks
            .iter()
            .any(|mark| mark.strip_prefix(prefix) == Some(value))
    }
    /// The last change is authoritative; a missing history means false.
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
    pub fn escalated(&self) -> bool {
        self.refuge.as_ref().is_some_and(|c| c.escalated)
    }
    /// Append scripted dialogue while preserving the current reading position.
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
