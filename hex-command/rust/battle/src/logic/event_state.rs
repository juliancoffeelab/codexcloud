//! Compiled mission clocks and private runtime indices; saved identities stay unchanged.
use super::{battle::Shared, event_queries::EventFacts};
use crate::data::*;
use std::{collections::BTreeMap, ops::Deref, rc::Rc};

#[derive(Clone, Copy, Debug)]
struct ClockSlot(usize);
#[derive(Clone, Debug)]
struct ClockDescriptor {
    slot: ClockSlot,
    predicate: u32,
    tile: TileRef,
    turns: u32,
    units: Option<UnitMatch>,
    faction: Option<String>,
    when: Option<When>,
}
#[derive(Debug)]
pub(crate) struct CompiledMission {
    mission: Mission,
    clocks: Vec<Vec<ClockDescriptor>>,
    clock_count: usize,
    event_indices: rustc_hash::FxHashMap<String, usize>,
    objective_indices: rustc_hash::FxHashMap<String, usize>,
}
impl CompiledMission {
    pub(super) fn new(mut mission: Mission) -> Self {
        for event in &mut mission.scripted_events {
            event.when.index_holds();
        }
        let mut count = 0;
        let clocks = mission
            .scripted_events
            .iter()
            .map(|event| {
                event
                    .when
                    .holds()
                    .into_iter()
                    .map(|hold| {
                        let slot = ClockSlot(count);
                        count += 1;
                        ClockDescriptor {
                            slot,
                            predicate: hold.predicate,
                            tile: hold.tile.clone(),
                            turns: hold.turns,
                            units: hold.units.cloned(),
                            faction: hold.faction.map(str::to_owned),
                            when: hold.when.cloned(),
                        }
                    })
                    .collect()
            })
            .collect();
        let event_indices = mission
            .scripted_events
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect();
        let objective_indices = mission
            .scripted_events
            .iter()
            .enumerate()
            .filter(|(_, e)| e.objective.is_some())
            .map(|(i, e)| (e.objective_id().to_owned(), i))
            .collect();
        Self {
            mission,
            clocks,
            clock_count: count,
            event_indices,
            objective_indices,
        }
    }
    fn clocks(&self, event: usize) -> &[ClockDescriptor] {
        &self.clocks[event]
    }
}
impl Deref for CompiledMission {
    type Target = Mission;
    fn deref(&self) -> &Mission {
        &self.mission
    }
}

/// Only these methods may change clock storage or the active index list.
/// Callers can inspect the wire state, but cannot mutate it through Deref.
#[derive(Clone, Debug)]
pub(crate) struct EventProgress {
    saved: EventState,
    plan: Rc<CompiledMission>,
    counter_indices: Rc<Vec<Option<usize>>>,
    active_indices: Rc<Vec<usize>>,
    unrevealed: Rc<Vec<usize>>,
}
impl Deref for EventProgress {
    type Target = EventState;
    fn deref(&self) -> &EventState {
        &self.saved
    }
}
impl EventProgress {
    pub(super) fn update_holds(state: &mut Shared<Self>, facts: &EventFacts<'_, '_>, tick: bool) {
        let plan = state.plan();
        let active = state.scan_indices();
        for &event in active.iter() {
            if !state.active_mask[event] {
                continue;
            }
            for hold in plan.clocks(event) {
                let standing = hold
                    .units
                    .as_ref()
                    .is_none_or(|u| facts.stands(u, &hold.tile, false))
                    && hold
                        .faction
                        .as_ref()
                        .is_none_or(|f| facts.owned_by(&hold.tile, f))
                    && hold.when.as_ref().is_none_or(|w| {
                        facts.when_matches(w, Some(state), Some(event), false, None)
                    });
                let Some(counter) = state.clock_at(hold.slot) else {
                    state.initialize_clock(event, hold, facts.round());
                    continue;
                };
                let (turns, last_round) = if !standing {
                    (0, facts.round())
                } else if tick && facts.turn() == Side::Player && facts.round() > counter.last_round
                {
                    ((counter.turns + 1).min(hold.turns), facts.round())
                } else {
                    (counter.turns, counter.last_round)
                };
                if (turns, last_round) != (counter.turns, counter.last_round) {
                    state.set_clock(hold.slot, turns, last_round);
                }
            }
        }
    }

    pub(super) fn restore(plan: Rc<CompiledMission>, mut saved: EventState) -> Self {
        saved.index_active(&plan.scripted_events);
        let mut counter_indices = vec![None; plan.clock_count];
        for (index, counter) in saved.holds.iter().enumerate() {
            let event = plan.event_indices[counter.event.as_str()];
            let clock = &plan.clocks[event][counter.predicate as usize];
            counter_indices[clock.slot.0] = Some(index);
        }
        let active_indices = Rc::new(std::mem::take(&mut saved.active_indices));
        let unrevealed = Rc::new(
            plan.scripted_events
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.objective.is_some() && !saved.revealed.iter().any(|id| id == e.objective_id())
                })
                .map(|(index, _)| index)
                .collect(),
        );
        Self {
            saved,
            plan,
            counter_indices: Rc::new(counter_indices),
            active_indices,
            unrevealed,
        }
    }
    pub(super) fn objective(&self, id: &str) -> Option<&ScriptEvent> {
        self.plan
            .objective_indices
            .get(id)
            .map(|&index| &self.plan.scripted_events[index])
    }
    pub(crate) fn saved(&self) -> &EventState {
        &self.saved
    }
    #[cfg(feature = "wasm")]
    pub(super) fn into_saved(self) -> EventState {
        self.saved
    }
    pub(super) fn plan(&self) -> Rc<CompiledMission> {
        self.plan.clone()
    }
    pub(super) fn scan_indices(&self) -> Rc<Vec<usize>> {
        self.active_indices.clone()
    }
    pub(super) fn clock(&self, event: usize, predicate: u32) -> Option<&EventHold> {
        let clock = self.plan.clocks.get(event)?.get(predicate as usize)?;
        self.clock_at(clock.slot)
    }
    fn clock_at(&self, slot: ClockSlot) -> Option<&EventHold> {
        self.counter_indices[slot.0].map(|index| &self.saved.holds[index])
    }
    fn initialize_clock(&mut self, event: usize, clock: &ClockDescriptor, round: u32) {
        debug_assert!(self.counter_indices[clock.slot.0].is_none());
        Rc::make_mut(&mut self.counter_indices)[clock.slot.0] = Some(self.saved.holds.len());
        self.saved.holds.push(EventHold {
            event: self.plan.scripted_events[event].id.clone(),
            predicate: clock.predicate,
            turns: 0,
            last_round: round,
        });
    }
    fn set_clock(&mut self, slot: ClockSlot, turns: u32, last_round: u32) {
        let counter =
            &mut self.saved.holds[self.counter_indices[slot.0].expect("initialized clock")];
        counter.turns = turns;
        counter.last_round = last_round;
    }
    pub(super) fn retire(&mut self, index: usize) {
        self.saved.retire(index);
    }
    pub(super) fn needs_compaction(&self) -> bool {
        self.active_indices
            .iter()
            .any(|&i| !self.saved.active_mask[i])
    }
    pub(super) fn compact(&mut self) {
        Rc::make_mut(&mut self.active_indices).retain(|&i| self.saved.active_mask[i]);
        self.saved
            .active
            .retain(|id| self.saved.active_mask[self.plan.event_indices[id]]);
    }

    pub(super) fn pending_mut(&mut self) -> &mut Vec<PendingEventSpawn> {
        &mut self.saved.pending
    }
    pub(super) fn marks_mut(&mut self) -> &mut Vec<MapMark> {
        &mut self.saved.marks
    }
    pub(super) fn recruitment_mut(&mut self) -> &mut BTreeMap<String, Vec<FactionRecruit>> {
        &mut self.saved.recruitment
    }
    pub(super) fn complete(&mut self, id: String) {
        self.saved.completed.push(id);
    }
    pub(super) fn unrevealed(&self) -> Rc<Vec<usize>> {
        self.unrevealed.clone()
    }
    pub(super) fn reveal(&mut self, index: usize) {
        self.saved
            .revealed
            .push(self.plan.scripted_events[index].objective_id().into());
        Rc::make_mut(&mut self.unrevealed).retain(|&i| i != index);
    }
    pub(super) fn fail(&mut self, id: String) {
        self.saved.failed.push(id);
    }
    pub(super) fn allocate_mark(&mut self) -> u32 {
        let id = self.saved.next_mark;
        self.saved.next_mark = id.checked_add(1).expect("mark identity exhausted");
        id
    }
}
