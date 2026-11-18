use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapMark {
    #[serde(rename = "lightRemembered")]
    pub light_remembered: bool,
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: Text,
    pub tile: Hex,
    pub light: Option<TileLight>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventHold {
    pub event: Text,
    pub predicate: u32,
    pub turns: u32,
    pub last_round: u32,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingEventSpawn {
    pub area: Vec<Hex>,
    pub faction: Text,
    #[serde(rename = "aiGroup")]
    pub ai_group: Option<Text>,
    pub units: Vec<SpawnMember>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventState {
    #[serde(skip)]
    pub active_indices: Vec<usize>,
    #[serde(skip)]
    pub active_mask: Vec<bool>,
    pub failed: Vec<Text>,
    pub completed: Vec<Text>,
    pub revealed: Vec<Text>,
    pub recruitment: std::collections::BTreeMap<Text, Vec<FactionRecruit>>,
    pub active: Vec<Text>,
    pub holds: Vec<EventHold>,
    pub marks: Vec<MapMark>,
    pub pending: Vec<PendingEventSpawn>,
    pub next_mark: u32,
}
pub struct EventClock<'a> {
    pub predicate: u32,
    pub tile: &'a TileRef,
    pub turns: u32,
    pub units: Option<&'a UnitMatch>,
    pub faction: Option<&'a str>,
    pub when: Option<&'a When>,
}
impl EventState {
    pub fn index_active(&mut self, events: &[ScriptEvent]) {
        let active: std::collections::HashSet<_> = self.active.iter().map(Text::as_str).collect();
        self.active_indices = events
            .iter()
            .enumerate()
            .filter(|(_, event)| active.contains(event.id.as_str()))
            .map(|(index, _)| index)
            .collect();
        self.active_mask = vec![false; events.len()];
        for &index in &self.active_indices {
            self.active_mask[index] = true;
        }
    }
    pub fn retire(&mut self, index: usize) {
        self.active_mask[index] = false;
    }
    pub fn compact(&mut self, events: &[ScriptEvent]) {
        self.active_indices.retain(|&index| self.active_mask[index]);
        let active: std::collections::HashSet<_> = self
            .active_indices
            .iter()
            .map(|&index| events[index].id.as_str())
            .collect();
        self.active.retain(|id| active.contains(id.as_str()));
    }
    pub fn new(events: &[ScriptEvent]) -> Self {
        Self {
            active_indices: (0..events.len()).collect(),
            active_mask: vec![true; events.len()],
            failed: vec![],
            completed: vec![],
            revealed: vec![],
            recruitment: Default::default(),
            active: events.iter().map(|e| e.id.clone()).collect(),
            holds: vec![],
            marks: vec![],
            pending: vec![],
            next_mark: 1,
        }
    }
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedStory {
    pub briefing: Text,
    pub debrief: Option<Text>,
    pub hints: Vec<Text>,
}
