//! Speculative battles lease ordinary owned storage and return it on drop.
//! Rules, geometry and caches remain shared; every mutable battle field is copied.
use super::Game;
use crate::data::*;
#[cfg(target_arch = "wasm32")]
type UnitIndices = rustc_hash::FxHashMap<u32, usize>;
#[cfg(not(target_arch = "wasm32"))]
use super::indices::UnitIndices;
use std::mem;
#[derive(Debug, Default)]
pub(super) struct Storage {
    pub owners: Vec<Option<Side>>,
    pub units: Vec<Unit>,
    pub map_id: String,
    pub log: Vec<String>,
    pub campaign: Option<CampaignState>,
    pub armies: Option<[String; 2]>,
    pub occupancy: Vec<Option<usize>>,
    pub indices: UnitIndices,
    pub marker_spare: Vec<String>,
}
impl Storage {
    pub fn copy(&mut self, source: &Game<'_>, presentation: bool) {
        self.owners.clone_from(&source.owners);
        self.units.clone_from(&source.units);
        #[cfg(target_arch = "wasm32")]
        self.units
            .reserve(source.terrain.len().saturating_sub(self.units.len()));
        self.map_id.clone_from(&source.map_id);
        if presentation {
            self.log.clone_from(&source.log);
        } else {
            self.log.clear();
        }
        if source.campaign.is_some() {
            #[cfg(target_arch = "wasm32")]
            let count = {
                let (count, bytes) = source.marker_capacity();
                let existing = self.campaign.as_ref().map_or(0, |c| c.marks.len());
                while self.marker_spare.len() + existing < count {
                    self.marker_spare.push(String::with_capacity(bytes));
                }
                count
            };
            #[cfg(not(target_arch = "wasm32"))]
            let count = source.campaign.as_ref().map_or(0, |c| c.marks.len());
            campaign(
                &mut self.campaign,
                &source.campaign,
                &mut self.marker_spare,
                count,
                presentation,
            );
        } else {
            self.campaign = None;
        }
        match (&mut self.armies, &source.armies) {
            (Some(to), Some(from)) => {
                to[0].clone_from(&from[0]);
                to[1].clone_from(&from[1]);
            }
            (to, from) => to.clone_from(from),
        }
        self.occupancy.clone_from(&source.occupancy);
        self.indices.clone_from(&source.unit_indices);
    }
    pub fn take(game: &mut Game<'_>) -> Self {
        Self {
            owners: mem::take(&mut game.owners),
            units: mem::take(&mut game.units),
            map_id: mem::take(&mut game.map_id),
            log: mem::take(&mut game.log),
            campaign: game.campaign.take(),
            armies: game.armies.take(),
            occupancy: mem::take(&mut game.occupancy),
            indices: mem::take(&mut game.unit_indices),
            marker_spare: mem::take(&mut game.marker_spare),
        }
    }
}
fn copy_items<T: Clone>(to: &mut Vec<T>, from: &[T], copy: impl Fn(&mut T, &T)) {
    for (to, from) in to.iter_mut().zip(from) {
        copy(to, from);
    }
    if to.len() < from.len() {
        to.extend_from_slice(&from[to.len()..]);
    }
    to.truncate(from.len());
}
fn veterans(to: &mut Option<Vec<Veteran>>, from: &Option<Vec<Veteran>>) {
    match (to, from) {
        (Some(to), Some(from)) => copy_items(to, from, |to, from| {
            to.r#type.clone_from(&from.r#type);
            to.name.clone_from(&from.name);
            to.experience = from.experience;
            to.role.clone_from(&from.role);
        }),
        (to, from) => to.clone_from(from),
    }
}
fn campaign(
    to: &mut Option<CampaignState>,
    from: &Option<CampaignState>,
    spare: &mut Vec<String>,
    capacity: usize,
    presentation: bool,
) {
    match (&mut *to, from) {
        (Some(to), Some(from)) => {
            to.id.clone_from(&from.id);
            to.mission = from.mission;
            if presentation {
                to.recall.clone_from(&from.recall);
                veterans(&mut to.veterans, &from.veterans);
                veterans(&mut to.story_veterans, &from.story_veterans);
            } else {
                to.recall.clear();
                to.veterans = None;
                to.story_veterans = None;
            }
            to.fallen_companions.clone_from(&from.fallen_companions);
            to.carry_gold = from.carry_gold;
            while to.marks.len() > from.marks.len() {
                spare.push(to.marks.pop().unwrap());
            }
            for (to, from) in to.marks.iter_mut().zip(&from.marks) {
                to.clone_from(from);
            }
            to.marks.reserve(capacity.saturating_sub(to.marks.len()));
            for from in &from.marks[to.marks.len()..] {
                let mut buffer = spare.pop().unwrap_or_default();
                buffer.clear();
                buffer.push_str(from);
                to.marks.push(buffer);
            }
            if presentation {
                copy_items(&mut to.dialogue, &from.dialogue, |to, from| {
                    to.speaker.clone_from(&from.speaker);
                    to.portrait.clone_from(&from.portrait);
                    to.text.clone_from(&from.text);
                });
                to.dialogue_index = from.dialogue_index;
                to.outcome.clone_from(&from.outcome);
            } else {
                to.dialogue.clear();
                to.outcome = None;
                to.dialogue_index = from.dialogue_index;
            }
            to.completed = from.completed;
            to.refuge.clone_from(&from.refuge);
            to.vigil.clone_from(&from.vigil);
            match (&mut to.holds, &from.holds) {
                (Some(to), Some(from)) => {
                    to.retain(|key, _| from.contains_key(key));
                    for (key, value) in from {
                        if let Some(counter) = to.get_mut(key) {
                            *counter = value.clone();
                        } else {
                            to.insert(key.clone(), value.clone());
                        }
                    }
                }
                (to, from) => to.clone_from(from),
            }
            to.choices.clone_from(&from.choices);
        }
        (to, Some(from)) if !presentation => {
            *to = Some(CampaignState {
                id: from.id.clone(),
                mission: from.mission,
                recall: Vec::new(),
                veterans: None,
                story_veterans: None,
                fallen_companions: from.fallen_companions.clone(),
                carry_gold: from.carry_gold,
                marks: from.marks.clone(),
                dialogue: Vec::new(),
                dialogue_index: from.dialogue_index,
                outcome: None,
                completed: from.completed,
                refuge: from.refuge.clone(),
                vigil: from.vigil.clone(),
                holds: from.holds.clone(),
                choices: from.choices.clone(),
            })
        }
        (to, from) => to.clone_from(from),
    }
    if let Some(to) = to.as_mut() {
        to.marks.reserve(capacity.saturating_sub(to.marks.len()));
        if let Some(from) = from.as_ref().filter(|_| presentation) {
            to.dialogue
                .reserve(from.dialogue.capacity().saturating_sub(to.dialogue.len()));
        }
    }
}
