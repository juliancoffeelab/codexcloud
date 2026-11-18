//! Ordered, general event execution. Conditions inspect facts; actions create durable facts.
use super::{LiveBattle, battle::Shared, event_queries::EventFacts, event_state::EventProgress};
use crate::CampaignProgram;
use crate::data::*;
use crate::logic::campaign_program::ChapterProgram;
use crate::logic::conditions::LiveContext;
use crate::logic::event_conditions::{ConditionTree, ScriptTree};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(crate) struct DeathOccurrence {
    pub unit: Unit,
    pub initiated_by: String,
    pub killed_by: Option<String>,
    pub defending: bool,
}

#[derive(Clone, Default)]
struct Bindings {
    observer: Option<String>,
    unit: Option<Unit>,
    mark: Option<MapMark>,
    death: Option<DeathOccurrence>,
}

impl LiveBattle<'_> {
    fn reveal_objectives(&mut self, state: &mut Shared<EventProgress>) {
        let plan = state.plan();
        let unrevealed = state.unrevealed();
        for &index in unrevealed.iter() {
            let objective = plan.scripted_events[index].objective.as_ref().unwrap();
            if objective.visible_when.as_ref().is_some_and(|when| {
                EventFacts::for_faction(self, &objective.faction).when_matches(
                    when,
                    Some(state),
                    None,
                    false,
                    None,
                )
            }) {
                state.reveal(index);
                self.story.planning_revision = self.story.planning_revision.wrapping_add(1);
            }
        }
    }

    fn release_event_spawns(&mut self, state: &mut Shared<EventProgress>) {
        if !state.pending.iter().any(|group| {
            group.units.iter().any(|member| {
                let kind = self.kind(&member.unit_type);
                group.area.iter().any(|id| {
                    let tile = self.site(id);
                    self.at(tile).is_none()
                        && !self
                            .arrival_tiles(
                                kind,
                                self.runtime
                                    .names
                                    .find(&group.faction)
                                    .expect("known spawn faction"),
                                &member.items,
                                &[tile],
                            )
                            .is_empty()
                })
            })
        }) {
            return;
        }
        for group in state.pending_mut() {
            let mut pending = vec![];
            for member in std::mem::take(&mut group.units) {
                let kind = self.kind(&member.unit_type);
                let tile = group.area.iter().map(|id| self.site(id)).find(|&t| {
                    self.at(t).is_none()
                        && !self
                            .arrival_tiles(
                                kind,
                                self.runtime
                                    .names
                                    .find(&group.faction)
                                    .expect("known spawn faction"),
                                &member.items,
                                &[t],
                            )
                            .is_empty()
                });
                if let Some(tile) = tile {
                    // A deferred arrival can obstruct an already-built move
                    // sequence even when it belongs to the acting faction.
                    self.story.planning_revision = self.story.planning_revision.wrapping_add(1);
                    let faction = self.intern_name(&group.faction);
                    let id = self.position.add(kind, faction, tile);
                    let index = self.index(id).unwrap();
                    let vision_group = self.vision_group(&group.faction);
                    let ai_group = group
                        .ai_group
                        .as_deref()
                        .map(|value| self.intern_name(value));
                    let character = member
                        .character
                        .as_deref()
                        .map(|value| self.intern_name(value));
                    let items = member.items.iter().map(|id| id.clone()).collect();
                    self.set_inventory(id, items);
                    let unit = &mut self.position.runtime.units[index];
                    unit.faction = faction;
                    unit.vision_group = vision_group;
                    unit.ai_group = ai_group;
                    unit.character = character;
                    unit.role = member.role;
                } else {
                    pending.push(member);
                }
            }
            group.units = pending;
        }
        state.pending_mut().retain(|p| !p.units.is_empty());
    }
    fn event_actions(
        &mut self,
        actions: &[EventAction],
        state: &mut Shared<EventProgress>,
        bindings: &Bindings,
    ) {
        for action in actions {
            self.story.planning_revision = self.story.planning_revision.wrapping_add(1);
            match action {
                EventAction::SetFlag(update) => {
                    self.story.campaign.as_mut().unwrap().set_flag(
                        match update {
                            FlagUpdate::Name(id) | FlagUpdate::Value { id, .. } => id,
                        },
                        match update {
                            FlagUpdate::Name(_) => true,
                            FlagUpdate::Value { state, .. } => *state,
                        },
                        self.position.runtime.round,
                    );
                }
                EventAction::Say(nodes) => {
                    let mut lines = vec![];
                    LiveContext::new(self, None, None).script_speech(nodes, &mut lines);
                    self.story.campaign.as_mut().unwrap().say(lines);
                }
                EventAction::Sound(sound) => {
                    self.story.campaign.as_mut().unwrap().say([Dialogue {
                        character: None,
                        speaker: String::new(),
                        portrait: "worker".into(),
                        text: String::new(),
                        sound: Some(sound.clone()),
                    }])
                }
                EventAction::Spawn(spec) => {
                    let area = if let Some(value) = &spec.tile {
                        let tile = bindings.tile(value, &self.board);
                        let origin = self.site(&tile);
                        let mut slots: Vec<_> = (0..self.position.runtime.terrain.len())
                            .filter(|&t| {
                                self.position.runtime.board.distance(origin, t) <= spec.radius
                            })
                            .collect();
                        slots
                            .sort_by_key(|&t| (self.position.runtime.board.distance(origin, t), t));
                        slots
                            .into_iter()
                            .map(|t| self.position.runtime.board.ids[t].clone())
                            .collect()
                    } else {
                        spec.area
                            .iter()
                            .map(|v| bindings.tile(v, &self.board))
                            .collect()
                    };
                    state.pending_mut().push(PendingEventSpawn {
                        area,
                        faction: spec.faction.clone(),
                        ai_group: spec.ai_group.clone(),
                        units: spec
                            .units
                            .iter()
                            .map(|entry| match entry {
                                SpawnEntry::Member(member) => member.clone(),
                                SpawnEntry::Type(unit_type) => SpawnMember {
                                    unit_type: unit_type.clone(),
                                    character: None,
                                    role: None,
                                    items: Vec::new(),
                                },
                            })
                            .collect(),
                    });
                    self.release_event_spawns(state);
                }
                EventAction::Give(spec) => {
                    let tile = spec.tile.as_ref().map(|t| bindings.tile(t, &self.board));
                    let selected: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            EventFacts::for_context(self, bindings.observer.as_deref())
                                .script_unit_matches(&spec.units, u)
                                && tile
                                    .as_ref()
                                    .is_none_or(|t| &self.position.runtime.board.ids[u.tile] == t)
                        })
                        .map(|u| u.id)
                        .collect();
                    for id in selected {
                        let index = self.index(id).unwrap();
                        let item = self.catalogue.item(&spec.item);
                        let mut equipment: Vec<_> = self.inventory(&self.units[index]).collect();
                        equipment.push(item);
                        if self.can_equip(&self.units[index], &equipment) {
                            self.give_item(id, spec.item.clone());
                        } else {
                            let item =
                                std::sync::Arc::make_mut(&mut self.position.runtime.inventories)
                                    .add_item(spec.item.clone());
                            self.position.runtime.ground_items.push(GroundItemState {
                                tile: self.position.runtime.units[index].tile,
                                item,
                            });
                        }
                    }
                }
                EventAction::PlaceItem(spec) => {
                    let tile = bindings.tile(&spec.tile, &self.board);
                    let item = std::sync::Arc::make_mut(&mut self.position.runtime.inventories)
                        .add_item(spec.item.clone());
                    self.position.runtime.ground_items.push(GroundItemState {
                        tile: self.site(&tile),
                        item,
                    });
                    if let Some(unit) = self.at(self.site(&tile)).map(|u| u.id) {
                        self.collect_items(unit);
                    }
                }
                EventAction::ChangeAllegiance(spec) => {
                    let phase = self
                        .faction(&spec.faction)
                        .expect("validated allegiance")
                        .phase;
                    let ids: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            EventFacts::for_context(self, bindings.observer.as_deref())
                                .script_unit_matches(&spec.units, u)
                        })
                        .map(|u| u.id)
                        .collect();
                    let faction = self.intern_name(&spec.faction);
                    let vision_group = self.vision_group(&spec.faction);
                    for id in ids {
                        if self
                            .traversal
                            .as_ref()
                            .is_some_and(|(actor, _)| *actor == id)
                        {
                            self.position.runtime.traversal = None;
                        }
                        let index = self.index(id).unwrap();
                        let unit = &mut self.position.runtime.units[index];
                        unit.phase = phase;
                        unit.faction = faction;
                        unit.vision_group = vision_group;
                        unit.ai_group = None;
                        unit.retreating = None;
                        unit.rebuilding = None;
                    }
                    self.roster_changed();
                }
                EventAction::DeleteUnit(reference) => {
                    let ids: Vec<_> = match reference {
                        UnitReference::Bound(_) => {
                            bindings.unit.as_ref().map(|u| u.id).into_iter().collect()
                        }
                        UnitReference::Selection(s) => self
                            .units
                            .iter()
                            .filter(|u| {
                                EventFacts::for_context(self, bindings.observer.as_deref())
                                    .script_unit_matches(s, u)
                            })
                            .map(|u| u.id)
                            .collect(),
                    };
                    self.position.runtime.units.retain(|u| !ids.contains(&u.id));
                    self.roster_changed();
                }
                EventAction::CreateMark(spec) => {
                    let id = state.allocate_mark();
                    state.marks_mut().push(MapMark {
                        id,
                        kind: spec.kind.clone(),
                        light_remembered: false,
                        tile: bindings.tile(&spec.tile, &self.board),
                        light: spec.light.clone(),
                    });
                }
                EventAction::RemoveMark(_) => {
                    let id = bindings.mark.as_ref().expect("validated mark").id;
                    state.marks_mut().retain(|m| m.id != id);
                }
                EventAction::ForEachUnit(spec) => {
                    let tile = spec
                        .select
                        .tile
                        .as_ref()
                        .map(|v| bindings.tile(v, &self.board));
                    let units: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            EventFacts::for_context(self, bindings.observer.as_deref())
                                .script_unit_matches(&spec.select.units, u)
                                && tile
                                    .as_ref()
                                    .is_none_or(|t| &self.position.runtime.board.ids[u.tile] == t)
                        })
                        .cloned()
                        .collect();
                    for unit in units {
                        let mut b = bindings.clone();
                        b.unit = Some(unit);
                        self.event_actions(&spec.actions, state, &b);
                    }
                }
                EventAction::ForEachMark(spec) => {
                    let marks: Vec<_> = state
                        .marks
                        .iter()
                        .filter(|m| m.kind == spec.select.kind)
                        .cloned()
                        .collect();
                    for mark in marks {
                        let mut b = bindings.clone();
                        b.mark = Some(mark);
                        self.event_actions(&spec.actions, state, &b);
                    }
                }
                EventAction::SetMapModifiers { modifiers } => {
                    self.position.runtime.map_modifiers = std::rc::Rc::new(modifiers.clone());
                    self.position.invalidate_terrain_queries();
                    self.refresh_planning_facts();
                }
                EventAction::SetTimeOfDay { time } => {
                    std::rc::Rc::make_mut(&mut self.position.runtime.lighting).time = *time;
                    self.position.refresh_ambient_illumination();
                    self.refresh_planning_facts();
                }
                EventAction::AddConcealment(spec) => {
                    let tiles = bindings.area(&spec.area, &self.board);
                    self.add_concealment(&tiles, &spec.rule);
                }
                EventAction::RemoveConcealment(spec) => {
                    let tiles = bindings.area(&spec.area, &self.board);
                    self.remove_concealment(&tiles, spec.kind);
                }
                EventAction::GrantPerception(spec) => {
                    let selected: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            EventFacts::for_context(self, bindings.observer.as_deref())
                                .script_unit_matches(&spec.units, u)
                        })
                        .map(|u| u.id)
                        .collect();
                    for id in selected {
                        self.grant_perception(id, spec.effect.clone());
                    }
                }
                EventAction::RemovePerception(spec) => {
                    let selected: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            EventFacts::for_context(self, bindings.observer.as_deref())
                                .script_unit_matches(&spec.units, u)
                        })
                        .map(|u| u.id)
                        .collect();
                    for id in selected {
                        let slot = self.unit(id).unwrap().perception.0;
                        self.position.runtime.unit_perception[slot]
                            .effects
                            .retain(|e| e.id != spec.effect);
                    }
                }
                EventAction::RevealTerrain(spec) => {
                    let tiles = bindings.area(&spec.area, &self.board);
                    self.reveal_terrain(&spec.group, &tiles);
                }
                EventAction::SetVisionGroup(spec) => {
                    self.assign_vision_group(&spec.faction, &spec.group)
                }
                EventAction::SetTerrain(spec) => {
                    let tile = bindings.tile(&spec.tile, &self.board);
                    let site = self.site(&tile);
                    let terrain = self.definitions.terrain_name(&spec.terrain);
                    self.set_terrain(site, terrain);
                }
                EventAction::Gold(spec) => self.position.runtime.gold[&spec.faction] += spec.amount,
                EventAction::Recruits(spec) => {
                    std::rc::Rc::make_mut(&mut self.position.runtime.resolved_recruitment)
                        .insert(spec.faction.clone(), spec.units.clone());
                    state
                        .recruitment_mut()
                        .insert(spec.faction.clone(), spec.units.clone());
                }
                EventAction::FailObjective(id) | EventAction::CompleteObjective(id) => {
                    let (index, event) = self
                        .story
                        .mission
                        .as_ref()
                        .unwrap()
                        .scripted_events
                        .iter()
                        .enumerate()
                        .find(|(_, e)| e.objective.is_some() && e.objective_id() == id)
                        .expect("validated objective ID");
                    let achieved = state.completed.contains(id);
                    if !state.failed.contains(id) {
                        match action {
                            EventAction::FailObjective(_)
                                if !achieved
                                    || event
                                        .objective
                                        .as_ref()
                                        .unwrap()
                                        .satisfied_when
                                        .is_some() =>
                            {
                                state.fail(id.clone());
                                state.retire(index);
                            }
                            EventAction::CompleteObjective(_) if !achieved => {
                                state.complete(id.clone())
                            }
                            _ => {}
                        }
                    }
                }
                EventAction::UnlockRecruits(spec) => {
                    let base = self
                        .faction(&spec.faction)
                        .and_then(|f| f.recruits.clone())
                        .unwrap_or_default();
                    let pool = state
                        .recruitment_mut()
                        .entry(spec.faction.clone())
                        .or_insert(base);
                    for unit in &spec.units {
                        if !pool.iter().any(|u| u.unit == unit.unit) {
                            pool.push(unit.clone());
                        }
                    }
                    if let Some(pool) = state.recruitment.get(&spec.faction) {
                        std::rc::Rc::make_mut(&mut self.position.runtime.resolved_recruitment)
                            .insert(spec.faction.clone(), pool.clone());
                    }
                }
                EventAction::Log(message) => self.record(message.clone()),
                EventAction::Victory(message) => self.finish_scripted_campaign(true, message),
                EventAction::Defeat(message) => self.finish_scripted_campaign(false, message),
            }
            self.refresh_perception_sources_with_marks(&state.marks);
        }
    }
    fn finish_scripted_campaign(&mut self, victory: bool, message: &str) {
        let definition = self.campaign_definition();
        let ending = definition.ending().to_vec();
        let last = definition.chapters().len() - 1;
        let mission = self.story.mission.as_ref().unwrap().clone();
        self.position.runtime.winner = Some(if victory {
            BattleResult::PlayerVictory
        } else {
            BattleResult::PlayerDefeat
        });
        let campaign = self.story.campaign.as_mut().unwrap();
        campaign.outcome = Some(message.into());
        campaign.completed = victory && campaign.mission == last;
        if victory {
            let complete = campaign.completed;
            let mut lines = vec![];
            LiveContext::new(self, None, None).script_speech(&mission.victory, &mut lines);
            if complete {
                LiveContext::new(self, None, None).script_speech(&ending, &mut lines);
            }
            self.story.campaign.as_mut().unwrap().say(lines);
        } else {
            campaign.say([Dialogue {
                character: None,
                speaker: "The company".into(),
                portrait: "worker".into(),
                text: message.into(),
                sound: None,
            }]);
        }
        self.record(message.into());
    }
    pub(crate) fn evaluate_scripted_events(&mut self, tick: bool) {
        let mut state = self
            .story
            .scripts
            .take()
            .expect("scripted chapter has event state");
        let mission = state.plan();
        let events = &mission.scripted_events;
        let deaths = std::mem::take(&mut self.story.event_deaths);
        self.release_event_spawns(&mut state);
        EventProgress::update_holds(&mut state, self, tick);
        let mut fired = HashSet::new();
        loop {
            self.reveal_objectives(&mut state);
            let mut changed = false;
            let active = state.scan_indices();
            for &index in active.iter() {
                if !state.active_mask[index] {
                    continue;
                }
                let event = &events[index];
                let ordinary = EventFacts::for_event(self, event).when_matches(
                    &event.when,
                    Some(&state),
                    Some(index),
                    tick,
                    None,
                );
                let contexts: Vec<_> = if ordinary {
                    vec![None]
                } else if event.when.has_death() {
                    deaths
                        .iter()
                        .filter(|u| {
                            EventFacts::for_event(self, event).when_matches(
                                &event.when,
                                Some(&state),
                                Some(index),
                                tick,
                                Some(u),
                            )
                        })
                        .map(|u| Some(u.clone()))
                        .collect()
                } else {
                    vec![]
                };
                for death in contexts {
                    if !fired.insert((index, death.as_ref().map(|u| u.unit.id))) {
                        continue;
                    }
                    if event.remembers {
                        state.retire(index);
                    }
                    self.event_actions(
                        &event.actions,
                        &mut state,
                        &Bindings {
                            observer: event.condition_observer().map(str::to_owned),
                            death,
                            ..Default::default()
                        },
                    );
                    if event.objective.is_some() {
                        let id = event.objective_id();
                        if !state.completed.iter().chain(&state.failed).any(|v| v == id) {
                            state.complete(id.into());
                        }
                    }
                    changed = true;
                    if self.position.runtime.winner.is_some() || event.remembers {
                        break;
                    }
                }
                if self.position.runtime.winner.is_some() {
                    break;
                }
            }
            drop(active);
            if changed && state.needs_compaction() {
                state.compact();
            }
            EventProgress::update_holds(&mut state, self, false);
            if !changed || self.position.runtime.winner.is_some() {
                break;
            }
        }
        self.release_event_spawns(&mut state);
        let remembered: Vec<_> = state
            .marks
            .iter()
            .enumerate()
            .filter_map(|(index, mark)| {
                (!mark.light_remembered
                    && mark
                        .light
                        .as_ref()
                        .and_then(|light| light.when.as_ref())
                        .is_some_and(|rule| {
                            rule.remember
                                && crate::data::presentation::rule_matches(
                                    rule,
                                    &self.condition_context(),
                                    Some(&TileRef::Hex(mark.tile)),
                                )
                        }))
                .then_some(index)
            })
            .collect();
        for index in remembered {
            state.marks_mut()[index].light_remembered = true;
        }
        self.reveal_objectives(&mut state);
        self.story.scripts = Some(state);
        self.story.event_departure = None;
        self.story.event_opening = false;
    }
}

pub(super) fn validate_event_state(
    rules: &Catalogue,
    battle: &State,
    mission: &ChapterProgram,
) -> Result<(), String> {
    let campaign = battle.campaign.as_ref().expect("campaign validation");
    let Some(state) = &campaign.scripts else {
        return if mission.scripted_events.is_empty() {
            Ok(())
        } else {
            Err("missing active event state".into())
        };
    };
    if mission.scripted_events.is_empty() {
        return Err("unexpected active event state".into());
    }
    let unique: HashSet<_> = state.active.iter().collect();
    if unique.len() != state.active.len()
        || state
            .active
            .iter()
            .any(|id| !mission.scripted_events.iter().any(|e| &e.id == id))
    {
        return Err("invalid active event IDs".into());
    }
    if mission
        .scripted_events
        .iter()
        .any(|e| !e.remembers && !state.active.contains(&e.id))
    {
        return Err("repeating event cannot be retired".into());
    }
    let mut failed_ids = HashSet::new();
    for id in &state.failed {
        let Some(event) = mission
            .scripted_events
            .iter()
            .find(|e| e.objective.is_some() && e.objective_id() == id)
        else {
            return Err("unknown failed objective".into());
        };
        if !failed_ids.insert(id) || state.active.contains(&event.id) {
            return Err("invalid failed objective state".into());
        }
    }
    for ids in [&state.completed, &state.revealed] {
        let mut seen = HashSet::new();
        for id in ids {
            if !seen.insert(id)
                || !mission
                    .scripted_events
                    .iter()
                    .any(|e| e.objective.is_some() && e.objective_id() == id)
            {
                return Err("invalid objective state".into());
            }
        }
    }
    if state.completed.iter().any(|id| {
        state.failed.contains(id)
            && mission
                .scripted_events
                .iter()
                .find(|event| event.objective.is_some() && event.objective_id() == id)
                .is_some_and(|event| event.objective.as_ref().unwrap().satisfied_when.is_none())
    }) {
        return Err("permanent achievement cannot be completed and failed".into());
    }
    let tiles: HashSet<_> = battle.tiles.iter().map(Tile::hex).collect();
    let mut counters = HashSet::new();
    for clock in &state.holds {
        let Some(event) = mission.scripted_events.iter().find(|e| e.id == clock.event) else {
            return Err("unknown held event".into());
        };
        let predicates = event.when.holds();
        let Some(hold) = predicates
            .iter()
            .find(|hold| hold.predicate == clock.predicate)
        else {
            return Err("unknown held predicate".into());
        };
        if clock.turns > hold.turns
            || clock.last_round > battle.round
            || !counters.insert((&clock.event, clock.predicate))
        {
            return Err("invalid held progress".into());
        }
    }
    let mut ids = HashSet::new();
    if state.next_mark == 0 {
        return Err("invalid next mark ID".into());
    }
    for mark in &state.marks {
        if mark.id == 0
            || mark.id >= state.next_mark
            || !ids.insert(mark.id)
            || !tiles.contains(&mark.tile)
        {
            return Err("invalid map mark".into());
        }
        super::snapshot_checks::identifier(&mark.kind)?;
        if let Some(light) = &mark.light {
            super::snapshot_checks::light(light, rules, battle)?;
        }
    }
    for pending in &state.pending {
        if pending.area.is_empty()
            || pending.units.is_empty()
            || pending.area.iter().any(|t| !tiles.contains(t))
            || !battle.factions.iter().any(|f| f.id == pending.faction)
        {
            return Err("invalid pending event spawn".into());
        }
        for unit in &pending.units {
            super::snapshot_checks::equipment(rules, battle, &unit.unit_type, &unit.items)?;
            if let Some(id) = &unit.character {
                if !rules
                    .campaign(&campaign.id)
                    .characters()
                    .iter()
                    .any(|character| &character.id == id)
                {
                    return Err("unknown pending character".into());
                }
            }
        }
    }
    for (faction, units) in &state.recruitment {
        if !battle.factions.iter().any(|f| &f.id == faction) {
            return Err("unknown unlocked faction".into());
        }
        let mut types = HashSet::new();
        for unit in units {
            if !rules.units().any(|definition| definition.id() == unit.unit)
                || !types.insert(&unit.unit)
                || unit.cost.is_some_and(|c| c < 0)
            {
                return Err("invalid unlocked recruitment".into());
            }
        }
    }
    Ok(())
}

impl Bindings {
    fn area(&self, area: &PerceptionArea, board: &super::battle::Board) -> Vec<usize> {
        match area {
            PerceptionArea::Map => (0..board.ids.len()).collect(),
            PerceptionArea::Tiles(tiles) => tiles
                .iter()
                .map(|tile| board.indices[&self.tile(tile, board)])
                .collect(),
        }
    }

    fn tile(&self, value: &TileRef, board: &super::battle::Board) -> Hex {
        match value {
            TileRef::Binding(TileBinding::Mark) => {
                self.mark.as_ref().expect("validated mark binding").tile
            }
            TileRef::Binding(TileBinding::Unit) => {
                board.ids[self.unit.as_ref().expect("validated unit binding").tile]
            }
            TileRef::Binding(TileBinding::Death) => {
                board.ids[self
                    .death
                    .as_ref()
                    .expect("validated death binding")
                    .unit
                    .tile]
            }
            _ => board.ids[super::event_queries::reference_site(board, value)],
        }
    }
}
