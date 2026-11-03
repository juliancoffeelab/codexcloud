//! Ordered, general event execution. Conditions inspect facts; actions create durable facts.
use super::Game;
use crate::data::presentation::ConditionContext;
use crate::data::*;
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
    unit: Option<Unit>,
    mark: Option<MapMark>,
    death: Option<DeathOccurrence>,
}

impl Game<'_> {
    pub(crate) fn script_unit_matches(&self, select: &UnitMatch, unit: &Unit) -> bool {
        select
            .character
            .as_ref()
            .is_none_or(|id| unit.character.as_ref() == Some(id))
            && select
                .retreating
                .is_none_or(|value| unit.retreating.unwrap_or(false) == value)
            && select.role.is_none_or(|r| unit.role == Some(r))
            && select
                .faction
                .as_ref()
                .is_none_or(|id| self.unit_faction(unit) == id)
            && select.side.is_none_or(|s| match s {
                ConditionSide::Any => true,
                ConditionSide::Player => unit.side == Side::Player,
                ConditionSide::Enemy => self.hostile_to_phase(unit, Side::Player),
                ConditionSide::Neutral => self
                    .faction(self.unit_faction(unit))
                    .is_some_and(|f| f.controller == Controller::Passive),
            })
            && select
                .unit_type
                .as_ref()
                .is_none_or(|id| &self.rules.units[unit.kind].id == id)
            && (select.unit_types.is_empty()
                || select.unit_types.contains(&self.rules.units[unit.kind].id))
            && select
                .ai_group
                .as_ref()
                .is_none_or(|id| unit.ai_group.as_ref() == Some(id))
    }
    fn pending_unit_matches(
        &self,
        s: &UnitMatch,
        group: &PendingEventSpawn,
        u: &SpawnMember,
    ) -> bool {
        let faction = self
            .faction(&group.faction)
            .expect("validated pending faction");
        s.retreating.is_none_or(|v| !v)
            && s.character
                .as_ref()
                .is_none_or(|id| u.character.as_ref() == Some(id))
            && s.role.is_none_or(|r| u.role == Some(r))
            && s.faction.as_ref().is_none_or(|id| id == &group.faction)
            && s.unit_type.as_ref().is_none_or(|id| id == &u.unit_type)
            && (s.unit_types.is_empty() || s.unit_types.contains(&u.unit_type))
            && s.ai_group
                .as_ref()
                .is_none_or(|id| group.ai_group.as_ref() == Some(id))
            && s.side.is_none_or(|side| match side {
                ConditionSide::Any => true,
                ConditionSide::Player => faction.phase == Side::Player,
                ConditionSide::Enemy => {
                    self.faction_relation("player", &group.faction) == Relation::Hostile
                }
                ConditionSide::Neutral => faction.controller == Controller::Passive,
            })
    }
    fn script_positions<'a>(&'a self, unit: &'a Unit, through: bool) -> &'a [usize] {
        if through {
            if let Some((actor, path)) = &self.traversal {
                if *actor == unit.id {
                    return path;
                }
            }
        }
        std::slice::from_ref(&unit.tile)
    }
    fn stands(&self, select: &UnitMatch, tile: &str, through: bool) -> bool {
        let target = self.site(tile);
        self.units.iter().any(|u| {
            self.script_unit_matches(select, u)
                && self.script_positions(u, through).contains(&target)
        })
    }
    fn when_matches(
        &self,
        when: &When,
        state: &EventState,
        event: &str,
        tick: bool,
        death: Option<&DeathOccurrence>,
        hold_index: &mut u32,
    ) -> bool {
        match when {
            When::All(values) | When::Any(values) => {
                // Visit every predicate, including short-circuited holds, to retain stable clock slots.
                let all = matches!(when, When::All(_));
                let mut result = all;
                for value in values {
                    let matches = self.when_matches(value, state, event, tick, death, hold_index);
                    if all {
                        result &= matches;
                    } else {
                        result |= matches;
                    }
                }
                result
            }
            When::Not(c) => !self.when_matches(c, state, event, tick, death, hold_index),
            When::Stand(v) => self.stands(&v.units, &v.tile, v.through),
            When::Proximity(v) => {
                let targets: Vec<_> = if let Some(tile) = &v.tile {
                    vec![self.site(tile)]
                } else {
                    self.units
                        .iter()
                        .filter(|u| self.script_unit_matches(v.target.as_ref().unwrap(), u))
                        .map(|u| u.tile)
                        .collect()
                };
                self.units
                    .iter()
                    .filter(|u| self.script_unit_matches(&v.units, u))
                    .any(|u| {
                        self.script_positions(u, v.through).iter().any(|&p| {
                            targets
                                .iter()
                                .any(|&t| self.board.distance(p, t) <= v.radius)
                        })
                    })
            }
            When::Killed(v) => death.is_some_and(|d| {
                self.script_unit_matches(&v.units, &d.unit)
                    && v.initiated_by.as_ref().is_none_or(|f| f == &d.initiated_by)
                    && v.killed_by
                        .as_ref()
                        .is_none_or(|f| Some(f) == d.killed_by.as_ref())
                    && v.defending.is_none_or(|value| value == d.defending)
            }),
            When::Left(v) => self.event_departure.as_ref().is_some_and(|u| {
                self.board.ids[u.tile] == v.tile && self.script_unit_matches(&v.units, u)
            }),
            When::Turn(turn) => {
                tick && self.round == turn.number()
                    && match turn {
                        TurnSelection::Number(_) => {
                            self.turn == Side::Player && self.event_turn_faction.is_none()
                        }
                        TurnSelection::Faction { faction, .. } => {
                            self.event_turn_faction.as_ref() == Some(faction)
                        }
                    }
            }
            When::TurnAtLeast(turn) => self.round >= *turn,
            When::ChapterStarted(value) => self.event_opening == *value,
            When::Flag(id) => self.campaign.as_ref().is_some_and(|c| c.chosen(id)),
            When::HasUnits(v) => self.units.iter().any(|u| self.script_unit_matches(v, u)),
            When::NoUnits(v) => {
                !self.units.iter().any(|u| self.script_unit_matches(v, u))
                    && !state
                        .pending
                        .iter()
                        .any(|g| g.units.iter().any(|u| self.pending_unit_matches(v, g, u)))
            }
            When::VisibleUnits(v) => self.units.iter().any(|u| {
                self.script_unit_matches(v, u)
                    && self
                        .condition_context()
                        .visible_at(self.board.positions[u.tile])
            }),
            When::Owner(v) => self.owners[self.site(&v.tile)].as_deref() == Some(&v.faction),
            When::Own(v) if v.turns.is_none() => {
                self.owners[self.site(&v.tile)].as_deref() == Some(&v.faction)
                    && v.when.as_ref().is_none_or(|w| self.script_condition(w))
            }
            When::Own(_) | When::Held(_) => {
                let index = *hold_index;
                *hold_index += 1;
                state
                    .holds
                    .iter()
                    .find(|h| h.event == event && h.predicate == index)
                    .is_some_and(|h| {
                        h.turns
                            >= match when {
                                When::Held(v) => v.turns,
                                When::Own(v) => v.turns.unwrap(),
                                _ => unreachable!(),
                            }
                    })
            }
        }
    }
    pub(crate) fn script_condition(&self, when: &When) -> bool {
        let empty = EventState::new(&[]);
        self.when_matches(
            when,
            self.campaign
                .as_ref()
                .and_then(|c| c.scripts.as_ref())
                .unwrap_or(&empty),
            "",
            false,
            None,
            &mut 0,
        )
    }
    fn update_event_holds(&self, state: &mut EventState, events: &[ScriptEvent], tick: bool) {
        for event in events.iter().filter(|e| state.active.contains(&e.id)) {
            for (index, hold) in event.when.holds().iter().enumerate() {
                let standing = hold.units.is_none_or(|u| self.stands(u, hold.tile, false))
                    && hold
                        .faction
                        .is_none_or(|f| self.owners[self.site(hold.tile)].as_deref() == Some(f))
                    && hold.when.is_none_or(|w| self.script_condition(w));
                let slot = state
                    .holds
                    .iter()
                    .position(|h| h.event == event.id && h.predicate == index as u32);
                let index = slot.unwrap_or_else(|| {
                    state.holds.push(EventHold {
                        event: event.id.clone(),
                        predicate: index as u32,
                        turns: 0,
                        last_round: self.round,
                    });
                    state.holds.len() - 1
                });
                let counter = &mut state.holds[index];
                if !standing {
                    counter.turns = 0;
                    counter.last_round = self.round;
                } else if tick
                    && self.turn == Side::Player
                    && self.event_turn_faction.is_none()
                    && self.round > counter.last_round
                {
                    counter.turns = (counter.turns + 1).min(hold.turns);
                    counter.last_round = self.round;
                }
            }
        }
    }
    pub(crate) fn resolve_text(&self, text: &ConditionalText) -> Option<String> {
        match text {
            ConditionalText::Text(text) => Some(text.clone()),
            ConditionalText::Variants(v) => v
                .iter()
                .find(|v| v.when.as_ref().is_none_or(|w| self.script_condition(w)))
                .map(|v| v.text.clone()),
        }
    }
    pub(crate) fn resolved_story(&self) -> Option<ResolvedStory> {
        let mission = self.mission.as_ref()?;
        Some(ResolvedStory {
            briefing: self.resolve_text(&mission.briefing).unwrap_or_default(),
            debrief: mission.debrief.as_ref().and_then(|t| self.resolve_text(t)),
            hints: mission
                .hints
                .iter()
                .filter_map(|hint| match hint {
                    ConditionalHint::Text(t) => Some(t.clone()),
                    ConditionalHint::Variant(v) => v
                        .when
                        .as_ref()
                        .is_none_or(|w| self.script_condition(w))
                        .then(|| v.text.clone()),
                })
                .collect(),
        })
    }
    fn bound_tile(&self, value: &str, bindings: &Bindings) -> String {
        match value {
            "$mark.tile" => bindings
                .mark
                .as_ref()
                .expect("validated mark binding")
                .tile
                .clone(),
            "$unit.tile" => {
                self.board.ids[bindings.unit.as_ref().expect("validated unit binding").tile].clone()
            }
            "$event.deathTile" => self.board.ids[bindings
                .death
                .as_ref()
                .expect("validated death binding")
                .unit
                .tile]
                .clone(),
            _ => value.to_owned(),
        }
    }
    pub(crate) fn script_speech(&self, nodes: &[Speech], out: &mut Vec<Dialogue>) {
        for node in nodes {
            match node {
                Speech::Branch(branch) => {
                    let present = self
                        .units
                        .iter()
                        .any(|u| u.character.as_ref() == Some(&branch.with));
                    self.script_speech(
                        if present {
                            &branch.dialogue
                        } else {
                            &branch.alternative
                        },
                        out,
                    );
                }
                Speech::Line(line) => {
                    let present = line.character.as_ref().and_then(|id| {
                        self.units.iter().find(|u| u.character.as_ref() == Some(id))
                    });
                    if let Some(id) = &line.character {
                        let party =
                            self.campaign_definition().missions.iter().any(|m| {
                                m.map.units.iter().any(|u| {
                                    u.side == Side::Player && u.character.as_ref() == Some(id)
                                }) || m.arrivals.iter().any(|a| &a.character == id)
                            });
                        if party && present.is_none() {
                            continue;
                        }
                    }
                    let portrait = line
                        .portrait
                        .clone()
                        .or_else(|| present.map(|u| self.rules.units[u.kind].id.clone()))
                        .unwrap_or_else(|| "worker".into());
                    out.push(Dialogue {
                        character: line.character.clone(),
                        speaker: line.speaker.clone().unwrap_or_default(),
                        portrait,
                        text: line.text.clone(),
                        sound: line.sound.clone(),
                    });
                }
            }
        }
    }
    fn release_event_spawns(&mut self, state: &mut EventState) {
        for group in &mut state.pending {
            let mut pending = vec![];
            for member in std::mem::take(&mut group.units) {
                let kind = self.kind(&member.unit_type);
                let tile = group
                    .area
                    .iter()
                    .map(|id| self.site(id))
                    .find(|&t| self.at(t).is_none() && self.cost(kind, t) != u16::MAX);
                if let Some(tile) = tile {
                    let phase = self
                        .faction(&group.faction)
                        .expect("validated spawn faction")
                        .phase;
                    let id = self.add(kind, phase, tile);
                    let index = self.index(id).unwrap();
                    let unit = &mut self.units[index];
                    unit.faction = Some(group.faction.clone());
                    unit.ai_group = group.ai_group.clone();
                    unit.character = member.character;
                    unit.role = member.role;
                    unit.items = member
                        .items
                        .iter()
                        .map(|id| self.rules.items[id].clone())
                        .collect();
                } else {
                    pending.push(member);
                }
            }
            group.units = pending;
        }
        state.pending.retain(|p| !p.units.is_empty());
    }
    fn event_actions(
        &mut self,
        actions: &[EventAction],
        state: &mut EventState,
        bindings: &Bindings,
    ) {
        for action in actions {
            match action {
                EventAction::SetFlag(update) => {
                    self.campaign.as_mut().unwrap().set_flag(
                        update.id(),
                        update.state(),
                        self.round,
                    );
                }
                EventAction::Say(nodes) => {
                    let mut lines = vec![];
                    self.script_speech(nodes, &mut lines);
                    self.campaign.as_mut().unwrap().say(lines);
                }
                EventAction::Sound(sound) => self.campaign.as_mut().unwrap().say([Dialogue {
                    character: None,
                    speaker: String::new(),
                    portrait: "worker".into(),
                    text: String::new(),
                    sound: Some(sound.clone()),
                }]),
                EventAction::Spawn(spec) => {
                    let area = if let Some(value) = &spec.tile {
                        let tile = self.bound_tile(value, bindings);
                        let origin = self.site(&tile);
                        let mut slots: Vec<_> = (0..self.terrain.len())
                            .filter(|&t| self.board.distance(origin, t) <= spec.radius)
                            .collect();
                        slots.sort_by_key(|&t| (self.board.distance(origin, t), t));
                        slots
                            .into_iter()
                            .map(|t| self.board.ids[t].clone())
                            .collect()
                    } else {
                        spec.area
                            .iter()
                            .map(|v| self.bound_tile(v, bindings))
                            .collect()
                    };
                    state.pending.push(PendingEventSpawn {
                        area,
                        faction: spec.faction.clone(),
                        ai_group: spec.ai_group.clone(),
                        units: spec.units.iter().map(SpawnEntry::member).collect(),
                    });
                    self.release_event_spawns(state);
                }
                EventAction::Give(spec) => {
                    let tile = spec.tile.as_ref().map(|t| self.bound_tile(t, bindings));
                    let selected: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            self.script_unit_matches(&spec.units, u)
                                && tile.as_ref().is_none_or(|t| &self.board.ids[u.tile] == t)
                        })
                        .map(|u| u.id)
                        .collect();
                    for id in selected {
                        let index = self.index(id).unwrap();
                        let mut inventory = self.units[index].items.clone();
                        inventory.push(self.rules.items[&spec.item].clone());
                        if validate_equipment(
                            &inventory,
                            &self.rules.units[self.units[index].kind].attacks,
                            "event gift",
                        )
                        .is_ok()
                        {
                            self.units[index].items = inventory;
                        } else {
                            self.ground_items.push(GroundItem {
                                tile: self.board.ids[self.units[index].tile].clone(),
                                item: self.rules.items[&spec.item].clone(),
                            });
                        }
                    }
                }
                EventAction::PlaceItem(spec) => {
                    let tile = self.bound_tile(&spec.tile, bindings);
                    self.ground_items.push(GroundItem {
                        tile: tile.clone(),
                        item: self.rules.items[&spec.item].clone(),
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
                        .filter(|u| self.script_unit_matches(&spec.units, u))
                        .map(|u| u.id)
                        .collect();
                    for id in ids {
                        let index = self.index(id).unwrap();
                        let unit = &mut self.units[index];
                        unit.side = phase;
                        unit.faction = Some(spec.faction.clone());
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
                            .filter(|u| self.script_unit_matches(s, u))
                            .map(|u| u.id)
                            .collect(),
                    };
                    self.units.retain(|u| !ids.contains(&u.id));
                    self.roster_changed();
                }
                EventAction::CreateMark(spec) => {
                    let id = state.next_mark;
                    state.next_mark = state
                        .next_mark
                        .checked_add(1)
                        .expect("mark identity exhausted");
                    state.marks.push(MapMark {
                        id,
                        kind: spec.kind.clone(),
                        light_remembered: false,
                        tile: self.bound_tile(&spec.tile, bindings),
                        light: spec.light.clone(),
                    });
                }
                EventAction::RemoveMark(_) => {
                    let id = bindings.mark.as_ref().expect("validated mark").id;
                    state.marks.retain(|m| m.id != id);
                }
                EventAction::ForEachUnit(spec) => {
                    let tile = spec
                        .select
                        .tile
                        .as_ref()
                        .map(|v| self.bound_tile(v, bindings));
                    let units: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            self.script_unit_matches(&spec.select.units, u)
                                && tile.as_ref().is_none_or(|t| &self.board.ids[u.tile] == t)
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
                EventAction::SetTerrain(spec) => {
                    let tile = self.bound_tile(&spec.tile, bindings);
                    self.set_terrain(self.site(&tile), self.rules.terrains[&spec.terrain]);
                }
                EventAction::Gold(spec) => self.gold[&spec.faction] += spec.amount,
                EventAction::Recruits(spec) => {
                    state
                        .recruitment
                        .insert(spec.faction.clone(), spec.units.clone());
                }
                EventAction::FailObjective(id) => {
                    let event = self
                        .mission
                        .as_ref()
                        .unwrap()
                        .scripted_events
                        .iter()
                        .find(|e| e.objective.is_some() && e.objective_id() == id)
                        .expect("validated objective ID");
                    if state.active.contains(&event.id) && !state.failed.contains(id) {
                        state.failed.push(id.clone());
                        state.active.retain(|v| v != &event.id);
                    }
                }
                EventAction::UnlockRecruits(spec) => {
                    let base = self
                        .faction(&spec.faction)
                        .and_then(|f| f.recruits.clone())
                        .unwrap_or_default();
                    let pool = state
                        .recruitment
                        .entry(spec.faction.clone())
                        .or_insert(base);
                    for unit in &spec.units {
                        if !pool.iter().any(|u| u.unit == unit.unit) {
                            pool.push(unit.clone());
                        }
                    }
                }
                EventAction::Log(message) => self.record(message.clone()),
                EventAction::Victory(message) => self.finish_scripted_campaign(true, message),
                EventAction::Defeat(message) => self.finish_scripted_campaign(false, message),
            }
        }
    }
    fn finish_scripted_campaign(&mut self, victory: bool, message: &str) {
        if victory
            && (!self
                .units
                .iter()
                .any(|u| u.side == Side::Player && u.is_leader())
                || self
                    .mission
                    .as_ref()
                    .unwrap()
                    .required_companions
                    .iter()
                    .any(|id| {
                        !self
                            .units
                            .iter()
                            .any(|u| u.side == Side::Player && u.character.as_ref() == Some(id))
                    }))
        {
            return;
        }
        let definition = self.campaign_definition();
        let ending = definition.ending.clone();
        let last = definition.missions.len() - 1;
        let mission = self.mission.as_ref().unwrap().clone();
        self.winner = Some(if victory { Side::Player } else { Side::Enemy });
        let campaign = self.campaign.as_mut().unwrap();
        campaign.outcome = Some(message.into());
        campaign.completed = victory && campaign.mission == last;
        if victory {
            let complete = campaign.completed;
            let mut lines = vec![];
            self.script_speech(&mission.victory, &mut lines);
            if complete {
                self.script_speech(&ending, &mut lines);
            }
            self.campaign.as_mut().unwrap().say(lines);
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
        let mission = self.mission.as_ref().unwrap().clone();
        let events = &mission.scripted_events;
        let mut state = self
            .campaign
            .as_mut()
            .unwrap()
            .scripts
            .take()
            .expect("scripted chapter has event state");
        let deaths = std::mem::take(&mut self.event_deaths);
        self.release_event_spawns(&mut state);
        self.update_event_holds(&mut state, &events, tick);
        let mut fired = HashSet::new();
        loop {
            let mut changed = false;
            for id in state.active.clone() {
                if !state.active.contains(&id) {
                    continue;
                }
                let event = events
                    .iter()
                    .find(|e| e.id == id)
                    .expect("validated active event");
                let ordinary =
                    self.when_matches(&event.when, &state, &event.id, tick, None, &mut 0);
                let contexts: Vec<_> = if ordinary {
                    vec![None]
                } else if event.when.has_death() {
                    deaths
                        .iter()
                        .filter(|u| {
                            self.when_matches(&event.when, &state, &event.id, tick, Some(u), &mut 0)
                        })
                        .map(|u| Some(u.clone()))
                        .collect()
                } else {
                    vec![]
                };
                for death in contexts {
                    if !fired.insert((event.id.clone(), death.as_ref().map(|u| u.unit.id))) {
                        continue;
                    }
                    if event.remembers {
                        state.active.retain(|id| id != &event.id);
                    }
                    self.event_actions(
                        &event.actions,
                        &mut state,
                        &Bindings {
                            death,
                            ..Default::default()
                        },
                    );
                    changed = true;
                    if self.winner.is_some() || event.remembers {
                        break;
                    }
                }
                if self.winner.is_some() {
                    break;
                }
            }
            self.update_event_holds(&mut state, &events, false);
            if !changed || self.winner.is_some() {
                break;
            }
        }
        self.release_event_spawns(&mut state);
        for mark in &mut state.marks {
            if let Some(rule) = mark.light.as_ref().and_then(|l| l.when.as_ref()) {
                if rule.remember
                    && crate::data::presentation::matches(
                        &rule.condition,
                        &self.condition_context(),
                        Some(&mark.tile),
                    )
                {
                    mark.light_remembered = true;
                }
            }
        }
        self.campaign.as_mut().unwrap().scripts = Some(state);
        self.event_departure = None;
        self.event_opening = false;
    }
    pub(crate) fn script_mark_lights(&self) -> Vec<BattlefieldLight> {
        self.campaign
            .as_ref()
            .and_then(|c| c.scripts.as_ref())
            .into_iter()
            .flat_map(|s| &s.marks)
            .filter_map(|m| {
                m.light
                    .as_ref()
                    .filter(|l| {
                        m.light_remembered
                            || l.when.as_ref().is_none_or(|rule| {
                                crate::data::presentation::matches(
                                    &rule.condition,
                                    &self.condition_context(),
                                    Some(&m.tile),
                                )
                            })
                    })
                    .map(|l| {
                        let (c, r) = self.board.positions[self.site(&m.tile)];
                        l.at(c, r)
                    })
            })
            .collect()
    }
    fn approach_tiles(&self, selection: &ProximitySelection) -> Vec<String> {
        let origins: Vec<_> = if let Some(t) = &selection.tile {
            vec![self.site(t)]
        } else {
            self.units
                .iter()
                .filter(|u| self.script_unit_matches(selection.target.as_ref().unwrap(), u))
                .map(|u| u.tile)
                .collect()
        };
        (0..self.terrain.len())
            .filter(|&t| {
                origins
                    .iter()
                    .any(|&o| self.board.distance(t, o) <= selection.radius)
            })
            .map(|t| self.board.ids[t].clone())
            .collect()
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn event_goal_progress(&self) -> Vec<Progress> {
        self.mission.as_ref().map_or_else(Vec::new, |m| {
            m.scripted_events
                .iter()
                .filter(|e| e.objective.as_ref().is_some_and(|o| !o.optional))
                .map(|e| self.scripted_objective_status(e).progress)
                .collect()
        })
    }

    pub(crate) fn scripted_policy_objectives(&self, side: Side) -> Vec<Objective> {
        let Some(state) = self.campaign.as_ref().and_then(|c| c.scripts.as_ref()) else {
            return vec![];
        };
        let mut goals = vec![];
        for event in &self.mission.as_ref().unwrap().scripted_events {
            let Some(objective) = &event.objective else {
                continue;
            };
            if !state.active.contains(&event.id)
                || objective
                    .visible_when
                    .as_ref()
                    .is_some_and(|c| !self.script_condition(c))
            {
                continue;
            }
            let (kind, tiles, selector, hold) = match &objective.intent {
                EventIntent::Hold(v) => (
                    ObjectiveKind::Vigil,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    true,
                ),
                EventIntent::Escort(v) => (
                    ObjectiveKind::Escort,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    false,
                ),
                EventIntent::GoTo(v) => (
                    ObjectiveKind::Visit,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    false,
                ),
                EventIntent::Escape(v) => (
                    ObjectiveKind::Escape,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    false,
                ),
                EventIntent::Approach(v) => (
                    ObjectiveKind::Visit,
                    self.approach_tiles(v),
                    Some(&v.units),
                    false,
                ),
                EventIntent::Own(v) => (ObjectiveKind::Capture, vec![v.tile.clone()], None, true),
                EventIntent::Capture(v) => (
                    ObjectiveKind::Capture,
                    v.iter()
                        .filter(|t| self.owners[self.site(t)].as_deref() != Some("player"))
                        .cloned()
                        .collect(),
                    None,
                    true,
                ),
                EventIntent::Defeat(v) => (
                    ObjectiveKind::Defeat,
                    self.units
                        .iter()
                        .filter(|u| self.script_unit_matches(v, u))
                        .map(|u| self.board.ids[u.tile].clone())
                        .collect(),
                    None,
                    false,
                ),
            };
            if tiles.is_empty() {
                continue;
            }
            let actor = selector
                .filter(|s| {
                    s.character.is_some()
                        || s.role.is_some()
                        || s.unit_type.is_some()
                        || !s.unit_types.is_empty()
                })
                .and_then(|s| {
                    self.units
                        .iter()
                        .filter(|u| {
                            self.belongs_to_phase(u, side) && self.script_unit_matches(s, u)
                        })
                        .min_by_key(|u| {
                            tiles
                                .iter()
                                .map(|t| self.board.distance(u.tile, self.site(t)))
                                .min()
                                .unwrap_or(0)
                        })
                });
            if selector.is_some_and(|s| {
                s.character.is_some()
                    || s.role.is_some()
                    || s.unit_type.is_some()
                    || !s.unit_types.is_empty()
            }) && actor.is_none()
            {
                continue;
            }
            goals.push(Objective {
                id: if objective.optional {
                    format!("optional:{}", event.objective_id())
                } else {
                    format!("required:{}", event.objective_id())
                },
                kind,
                tiles,
                weight: if objective.optional { 4.0 } else { 24.0 },
                actor: actor.map(|u| format!("u{}", u.id)),
                family: None,
                hold: Some(hold),
            });
        }
        goals
    }
    pub(crate) fn scripted_objective_status(&self, event: &ScriptEvent) -> ObjectiveStatus {
        let objective = event.objective.as_ref().unwrap();
        let state = self.campaign.as_ref().unwrap().scripts.as_ref().unwrap();
        let failed = state.failed.iter().any(|id| id == event.objective_id());
        let done = !failed && !state.active.contains(&event.id);
        let (tiles, total) = match &objective.intent {
            EventIntent::Hold(v) => (vec![v.tile.clone()], v.turns),
            EventIntent::Escort(v) | EventIntent::GoTo(v) | EventIntent::Escape(v) => {
                (vec![v.tile.clone()], 1)
            }
            EventIntent::Approach(v) => (self.approach_tiles(v), 1),
            EventIntent::Own(v) => (vec![v.tile.clone()], v.turns.unwrap_or(1)),
            EventIntent::Capture(v) => (v.clone(), v.len() as u32),
            EventIntent::Defeat(v) => (
                self.units
                    .iter()
                    .filter(|u| self.script_unit_matches(v, u))
                    .map(|u| self.board.ids[u.tile].clone())
                    .collect(),
                1,
            ),
        };
        let current = if done {
            total
        } else if let EventIntent::Capture(tiles) = &objective.intent {
            tiles
                .iter()
                .filter(|t| self.owners[self.site(t)].as_deref() == Some("player"))
                .count() as u32
        } else if matches!(
            &objective.intent,
            EventIntent::Hold(_) | EventIntent::Own(_)
        ) {
            state
                .holds
                .iter()
                .filter(|h| h.event == event.id)
                .map(|h| h.turns)
                .max()
                .unwrap_or(0)
                .min(total)
        } else {
            0
        };
        let visible = objective
            .visible_when
            .as_ref()
            .is_none_or(|c| self.script_condition(c));
        ObjectiveStatus {
            active_tiles: if !done && !failed && visible {
                tiles.clone()
            } else {
                vec![]
            },
            visible,
            failed,
            progress: Progress {
                label: objective.label.clone(),
                current,
                total,
                done,
                tiles,
            },
        }
    }
}

impl Game<'_> {
    fn next_script_turn(&self, condition: &When) -> Option<u32> {
        match condition {
            When::Turn(t) => (t.number() >= self.round).then_some(t.number()),
            When::TurnAtLeast(t) => Some((*t).max(self.round)),
            When::All(v) => {
                let parts: Option<Vec<_>> = v.iter().map(|c| self.next_script_turn(c)).collect();
                parts.and_then(|v| v.into_iter().max())
            }
            When::Any(v) => v.iter().filter_map(|c| self.next_script_turn(c)).min(),
            When::ChapterStarted(_) => None,
            other => self.script_condition(other).then_some(self.round),
        }
    }
    #[cfg(any(test, feature = "wasm"))]
    pub(crate) fn script_notices(&self) -> Vec<String> {
        let Some(state) = self.campaign.as_ref().and_then(|c| c.scripts.as_ref()) else {
            return vec![];
        };
        let mut upcoming = vec![];
        let mut notices = vec![];
        for event in &self.mission.as_ref().unwrap().scripted_events {
            let Some(text) = &event.notice else { continue };
            if !state.active.contains(&event.id) {
                continue;
            }
            let Some(turn) = self.next_script_turn(&event.when) else {
                continue;
            };
            if event
                .actions
                .iter()
                .any(|a| matches!(a, EventAction::Defeat(_)))
            {
                notices.push(format!(
                    "Turn {}/{} — {text}",
                    self.round,
                    turn.saturating_sub(1)
                ));
            } else {
                upcoming.push((turn, text));
            }
        }
        if let Some((turn, text)) = upcoming.into_iter().min_by_key(|v| v.0) {
            notices.push(format!("Turn {turn}: {text}"));
        }
        notices
    }
}

pub(super) fn validate_event_state(
    rules: &Rules,
    battle: &State,
    mission: &Mission,
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
    let tiles: HashSet<_> = battle.tiles.iter().map(|t| t.id.as_str()).collect();
    let mut counters = HashSet::new();
    for clock in &state.holds {
        let Some(event) = mission.scripted_events.iter().find(|e| e.id == clock.event) else {
            return Err("unknown held event".into());
        };
        let predicates = event.when.holds();
        let Some(hold) = predicates.get(clock.predicate as usize) else {
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
            || !tiles.contains(mark.tile.as_str())
        {
            return Err("invalid map mark".into());
        }
        crate::data::catalog::identifier(&mark.kind, "saved mark").map_err(|e| e.message)?;
        if let Some(light) = &mark.light {
            light.validate(&battle.tiles, &[])?;
        }
    }
    for pending in &state.pending {
        if pending.area.is_empty()
            || pending.units.is_empty()
            || pending.area.iter().any(|t| !tiles.contains(t.as_str()))
            || !battle.factions.iter().any(|f| f.id == pending.faction)
        {
            return Err("invalid pending event spawn".into());
        }
        for unit in &pending.units {
            let Some(&kind) = rules.types.get(&unit.unit_type) else {
                return Err("unknown pending unit type".into());
            };
            let inventory = unit
                .items
                .iter()
                .map(|id| rules.items.get(id).cloned().ok_or("unknown pending item"))
                .collect::<Result<Vec<_>, _>>()?;
            validate_equipment(&inventory, &rules.units[kind].attacks, "pending inventory")
                .map_err(|e| e.message)?;
            if let Some(id) = &unit.character {
                if rules.campaigns[&campaign.id].character(id).is_none() {
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
            if !rules.types.contains_key(&unit.unit)
                || !types.insert(&unit.unit)
                || unit.cost.is_some_and(|c| c < 0)
            {
                return Err("invalid unlocked recruitment".into());
            }
        }
    }
    Ok(())
}
