//! Ordered, general event execution. Conditions inspect facts; actions create durable facts.
use super::{Game, battle::Shared, event_queries::EventFacts, event_state::EventProgress};
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
    /// Unconditional timed defeats are explicit mission limits. Conditional
    /// events remain unknown rather than guessing whether their guards will hold.
    pub(crate) fn planning_deadline_round(&self) -> Option<u32> {
        if self.story.campaign.is_none() {
            return None;
        }
        self.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .filter(|event| {
                event
                    .actions
                    .iter()
                    .any(|a| matches!(a, EventAction::Defeat(_)))
            })
            .filter_map(|event| match &event.when {
                When::TurnAtLeast(n) => Some(*n),
                When::Turn(TurnSelection::Number(n)) => Some(*n),
                _ => None,
            })
            .min()
    }

    #[inline]
    pub(crate) fn script_unit_matches(&self, select: &UnitMatch, unit: &Unit) -> bool {
        super::event_queries::unit_matches(
            self.position.rules,
            &self.position.runtime.factions,
            &self.position.runtime.names,
            select,
            unit,
        )
    }
    pub(crate) fn objective_unit_matches(
        &self,
        faction: &str,
        select: &UnitMatch,
        unit: &Unit,
    ) -> bool {
        super::event_queries::unit_matches_for_faction(
            self.rules,
            &self.factions,
            &self.names,
            select,
            unit,
            faction,
        )
    }
    pub(crate) fn script_condition(&self, when: &When) -> bool {
        let state = self.story.scripts.as_deref();
        assert!(
            state.is_some() || self.story.campaign.is_none(),
            "event evaluation requires explicit current progress"
        );
        EventFacts::new(self).when_matches(when, state, None, false, None)
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
        let mission = self.story.mission.as_ref()?;
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
    pub(crate) fn reference_site(&self, target: &TileRef) -> usize {
        self.runtime
            .board
            .resolve(target)
            .expect("validated authored target")
    }
    fn bound_tile(&self, value: &TileRef, bindings: &Bindings) -> Hex {
        match value {
            TileRef::Binding(TileBinding::Mark) => {
                bindings.mark.as_ref().expect("validated mark binding").tile
            }
            TileRef::Binding(TileBinding::Unit) => {
                self.position.runtime.board.ids
                    [bindings.unit.as_ref().expect("validated unit binding").tile]
            }
            TileRef::Binding(TileBinding::Death) => {
                self.position.runtime.board.ids[bindings
                    .death
                    .as_ref()
                    .expect("validated death binding")
                    .unit
                    .tile]
            }
            _ => self.position.runtime.board.ids[self.reference_site(value)],
        }
    }
    pub(crate) fn script_speech(&self, nodes: &[Speech], out: &mut Vec<Dialogue>) {
        for node in nodes {
            match node {
                Speech::Branch(branch) => {
                    let present = self
                        .units
                        .iter()
                        .any(|u| self.name(u.character) == Some(branch.with.as_str()));
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
                        self.runtime
                            .units
                            .iter()
                            .find(|u| self.name(u.character) == Some(id.as_str()))
                    });
                    if let Some(id) = &line.character {
                        let party =
                            self.campaign_definition().missions.iter().any(|m| {
                                m.map.units.iter().any(|u| {
                                    u.faction == "player" && u.character.as_ref() == Some(id)
                                }) || m.arrivals.iter().any(|a| &a.character == id)
                            });
                        if party && present.is_none() {
                            continue;
                        }
                    }
                    let portrait = line
                        .portrait
                        .clone()
                        .or_else(|| present.map(|u| self.position.rules.units[u.kind].id.clone()))
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
                    self.at(tile).is_none() && self.cost(kind, tile) != u16::MAX
                })
            })
        }) {
            return;
        }
        for group in state.pending_mut() {
            let mut pending = vec![];
            for member in std::mem::take(&mut group.units) {
                let kind = self.kind(&member.unit_type);
                let tile = group
                    .area
                    .iter()
                    .map(|id| self.site(id))
                    .find(|&t| self.at(t).is_none() && self.cost(kind, t) != u16::MAX);
                if let Some(tile) = tile {
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
                    let items = member
                        .items
                        .iter()
                        .map(|id| self.position.rules.items[id].clone())
                        .collect();
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
                        update.id(),
                        update.state(),
                        self.position.runtime.round,
                    );
                }
                EventAction::Say(nodes) => {
                    let mut lines = vec![];
                    self.script_speech(nodes, &mut lines);
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
                        let tile = self.bound_tile(value, bindings);
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
                            .map(|v| self.bound_tile(v, bindings))
                            .collect()
                    };
                    state.pending_mut().push(PendingEventSpawn {
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
                                && tile
                                    .as_ref()
                                    .is_none_or(|t| &self.position.runtime.board.ids[u.tile] == t)
                        })
                        .map(|u| u.id)
                        .collect();
                    for id in selected {
                        let index = self.index(id).unwrap();
                        let item = &self.position.rules.items[&spec.item];
                        if validate_equipment(
                            self.inventory(&self.position.runtime.units[index])
                                .chain(std::iter::once(item)),
                            &self.position.rules.units[self.position.runtime.units[index].kind]
                                .attacks,
                            "event gift",
                        )
                        .is_ok()
                        {
                            self.give_item(id, item.clone());
                        } else {
                            let item =
                                std::sync::Arc::make_mut(&mut self.position.runtime.inventories)
                                    .add_item(item.clone(), self.position.mechanics);
                            self.position.runtime.ground_items.push(GroundItemState {
                                tile: self.position.runtime.units[index].tile,
                                item,
                            });
                        }
                    }
                }
                EventAction::PlaceItem(spec) => {
                    let tile = self.bound_tile(&spec.tile, bindings);
                    let item = std::sync::Arc::make_mut(&mut self.position.runtime.inventories)
                        .add_item(
                            self.position.rules.items[&spec.item].clone(),
                            self.position.mechanics,
                        );
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
                        .filter(|u| self.script_unit_matches(&spec.units, u))
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
                            .filter(|u| self.script_unit_matches(s, u))
                            .map(|u| u.id)
                            .collect(),
                    };
                    self.position.runtime.units.retain(|u| !ids.contains(&u.id));
                    self.roster_changed();
                }
                EventAction::CreateMark(spec) => {
                    let id = state.allocate_mark();
                    state.marks_mut().push(MapMark {
                        sight: spec.sight.clone(),
                        id,
                        kind: spec.kind.clone(),
                        light_remembered: false,
                        tile: self.bound_tile(&spec.tile, bindings),
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
                        .map(|v| self.bound_tile(v, bindings));
                    let units: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| {
                            self.script_unit_matches(&spec.select.units, u)
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
                EventAction::AddConcealment(spec) => self.add_concealment(&spec.area, &spec.rule),
                EventAction::RemoveConcealment(spec) => {
                    self.remove_concealment(&spec.area, spec.kind)
                }
                EventAction::GrantPerception(spec) => {
                    let selected: Vec<_> = self
                        .units
                        .iter()
                        .filter(|u| self.script_unit_matches(&spec.units, u))
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
                        .filter(|u| self.script_unit_matches(&spec.units, u))
                        .map(|u| u.id)
                        .collect();
                    for id in selected {
                        let slot = self.unit(id).unwrap().perception.0;
                        self.position.runtime.unit_perception[slot]
                            .effects
                            .retain(|e| e.id != spec.effect);
                    }
                }
                EventAction::RevealTerrain(spec) => self.reveal_terrain(&spec.group, &spec.area),
                EventAction::SetVisionGroup(spec) => {
                    self.assign_vision_group(&spec.faction, &spec.group)
                }
                EventAction::SetTerrain(spec) => {
                    let tile = self.bound_tile(&spec.tile, bindings);
                    let site = self.site(&tile);
                    let terrain = self.position.rules.terrains[&spec.terrain];
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
        if victory
            && (!self
                .units
                .iter()
                .any(|u| self.unit_faction(u) == "player" && u.is_leader())
                || self
                    .story
                    .mission
                    .as_ref()
                    .unwrap()
                    .required_companions
                    .iter()
                    .any(|id| {
                        !self.units.iter().any(|u| {
                            self.unit_faction(u) == "player"
                                && self.name(u.character) == Some(id.as_str())
                        })
                    }))
        {
            return;
        }
        let definition = self.campaign_definition();
        let ending = definition.ending.clone();
        let last = definition.missions.len() - 1;
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
            self.script_speech(&mission.victory, &mut lines);
            if complete {
                self.script_speech(&ending, &mut lines);
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
                                && crate::data::presentation::matches(
                                    &rule.condition,
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
    pub(crate) fn approach_tiles<'unit>(
        &self,
        selection: &ProximitySelection,
        faction: &str,
        units: impl Iterator<Item = &'unit Unit>,
    ) -> Vec<Hex> {
        let origins: Vec<_> = if let Some(t) = &selection.tile {
            vec![self.reference_site(t)]
        } else {
            units
                .filter(|u| {
                    self.objective_unit_matches(faction, selection.target.as_ref().unwrap(), u)
                })
                .map(|u| u.tile)
                .collect()
        };
        (0..self.position.runtime.terrain.len())
            .filter(|&t| {
                origins
                    .iter()
                    .any(|&o| self.position.runtime.board.distance(t, o) <= selection.radius)
            })
            .map(|t| self.position.runtime.board.ids[t].clone())
            .collect()
    }
    fn objective_available(&self, event: &ScriptEvent) -> bool {
        let state = self.story.scripts.as_deref().unwrap();
        let objective = event.objective.as_ref().unwrap();
        !state.failed.iter().any(|id| id == event.objective_id())
            && (objective.satisfied_when.is_some()
                || !state.completed.iter().any(|id| id == event.objective_id()))
            && objective.available_when.as_ref().is_none_or(|when| {
                EventFacts::for_faction(self, &objective.faction).when_matches(
                    when,
                    Some(state),
                    None,
                    false,
                    None,
                )
            })
    }
    pub(crate) fn planning_objectives(
        &self,
        faction: &str,
    ) -> Vec<crate::view::CampaignObjective<'_>> {
        use crate::view::{CampaignIntent as Kind, CampaignObjective};
        let Some(state) = self.story.scripts.as_deref() else {
            return vec![];
        };
        let facts = EventFacts::new(self);
        self.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .filter_map(|event| {
                let objective = event.objective.as_ref()?;
                if objective.faction != faction {
                    return None;
                }
                let id = event.objective_id();
                if !state.revealed.iter().any(|known| known == id)
                    || !self.objective_available(event)
                {
                    return None;
                }
                let satisfied = facts.objective_satisfied(state, id);
                let (intent, selector, tiles) = match &objective.intent {
                    EventIntent::Hold(v) => (
                        Kind::Hold,
                        Some(&v.units),
                        vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                    ),
                    EventIntent::Escort(v) => (
                        Kind::Escort,
                        Some(&v.units),
                        vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                    ),
                    EventIntent::GoTo(v) => (
                        Kind::GoTo,
                        Some(&v.units),
                        vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                    ),
                    EventIntent::Escape(v) => (
                        Kind::Escape,
                        Some(&v.units),
                        vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                    ),
                    EventIntent::Approach(v) => {
                        let tiles = self.approach_tiles(
                            v,
                            faction,
                            self.units
                                .iter()
                                .filter(|unit| self.unit_visible_to(faction, unit, None)),
                        );
                        if tiles.is_empty() {
                            return None;
                        }
                        (Kind::Approach, Some(&v.units), tiles)
                    }
                    EventIntent::Own(v) => (
                        Kind::Own,
                        None,
                        vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                    ),
                    EventIntent::Capture(v) => (
                        Kind::Capture,
                        None,
                        v.iter()
                            .map(|t| self.position.runtime.board.ids[self.reference_site(t)])
                            .collect(),
                    ),
                    EventIntent::Defeat(v) => (
                        Kind::Defeat,
                        Some(v),
                        self.runtime
                            .units
                            .iter()
                            .filter(|unit| self.script_unit_matches(v, unit))
                            .map(|unit| self.position.runtime.board.ids[unit.tile].clone())
                            .collect(),
                    ),
                };
                let progress = self.scripted_objective_status(event).progress;
                let hold_turns_remaining = match &objective.intent {
                    EventIntent::Hold(v) => Some(v.turns.saturating_sub(progress.current)),
                    EventIntent::Own(v) => v.turns.map(|n| n.saturating_sub(progress.current)),
                    _ => None,
                };
                Some(CampaignObjective {
                    id,
                    optional: objective.optional,
                    satisfied,
                    intent,
                    selector,
                    tiles,
                    progress_current: progress.current,
                    progress_total: progress.total,
                    hold_turns_remaining,
                })
            })
            .collect()
    }
    /// Aggregate readiness blocks terminal shortcuts without revealing hidden requirements.
    pub(crate) fn requirements_disclosed(&self, faction: &str) -> bool {
        let Some(state) = self.story.scripts.as_deref() else {
            return true;
        };
        let facts = EventFacts::new(self);
        self.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .all(|event| {
                event.objective.as_ref().is_none_or(|objective| {
                    objective.faction != faction
                        || objective.optional
                        || facts.objective_satisfied(state, event.objective_id())
                        || (state.revealed.iter().any(|id| id == event.objective_id())
                            && objective.available_when.as_ref().is_none_or(|when| {
                                EventFacts::for_faction(self, faction).when_matches(
                                    when,
                                    Some(state),
                                    None,
                                    false,
                                    None,
                                )
                            }))
                })
            })
    }
    pub(crate) fn script_pending_arrivals(&self) -> bool {
        if self.position.runtime.turn != TurnPhase::Player {
            return false;
        }
        let Some(state) = self.story.scripts.as_deref() else {
            return false;
        };
        state.pending.iter().any(|pending| {
            self.faction(&pending.faction)
                .is_some_and(|f| f.phase == TurnPhase::Ai)
        }) || self
            .story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .any(|event| {
                let due = match &event.when {
                    When::Turn(TurnSelection::Faction { number, faction }) => {
                        *number <= self.position.runtime.round
                            && self
                                .faction(faction)
                                .is_some_and(|f| f.phase == TurnPhase::Ai)
                    }
                    _ => false,
                };
                due && state.active.iter().any(|id| id == &event.id)
                    && event
                        .actions_recursive()
                        .iter()
                        .any(|a| matches!(a, EventAction::Spawn(_)))
            })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn event_goal_progress(&self) -> Vec<Progress> {
        self.story.mission.as_ref().map_or_else(Vec::new, |m| {
            m.scripted_events
                .iter()
                .filter(|e| {
                    e.objective
                        .as_ref()
                        .is_some_and(|o| o.faction == "player" && !o.optional)
                })
                .map(|e| self.scripted_objective_status(e).progress)
                .collect()
        })
    }

    pub(crate) fn scripted_objective_status(&self, event: &ScriptEvent) -> ObjectiveStatus {
        let objective = event.objective.as_ref().unwrap();
        let state = self.story.scripts.as_deref().unwrap();
        let failed = state.failed.iter().any(|id| id == event.objective_id());
        let done = EventFacts::for_faction(self, &objective.faction)
            .objective_satisfied(state, event.objective_id());
        let available = self.objective_available(event);
        let (tiles, total) = match &objective.intent {
            EventIntent::Hold(v) => (
                vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                v.turns,
            ),
            EventIntent::Escort(v) | EventIntent::GoTo(v) | EventIntent::Escape(v) => (
                vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                1,
            ),
            EventIntent::Approach(v) => (
                self.approach_tiles(v, &objective.faction, self.units.iter()),
                1,
            ),
            EventIntent::Own(v) => (
                vec![self.position.runtime.board.ids[self.reference_site(&v.tile)]],
                v.turns.unwrap_or(1),
            ),
            EventIntent::Capture(v) => (
                v.iter()
                    .map(|t| self.position.runtime.board.ids[self.reference_site(t)])
                    .collect(),
                v.len() as u32,
            ),
            EventIntent::Defeat(v) => (
                self.runtime
                    .units
                    .iter()
                    .filter(|u| self.script_unit_matches(v, u))
                    .map(|u| self.position.runtime.board.ids[u.tile].clone())
                    .collect(),
                1,
            ),
        };
        let current = if done {
            total
        } else if let EventIntent::Capture(tiles) = &objective.intent {
            tiles
                .iter()
                .filter(|t| {
                    self.position.runtime.owners[self.reference_site(t)].as_deref()
                        == Some(objective.faction.as_str())
                })
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
        let visible = state.revealed.iter().any(|id| id == event.objective_id());
        // Markers are authored UI data, never inferred from AI movement or completion regions.
        let markers = objective
            .player_marker
            .as_ref()
            .map_or_else(Vec::new, |marker| {
                let destinations: Vec<Hex> = if let Some(select) = &marker.units {
                    self.runtime
                        .units
                        .iter()
                        .filter(|unit| self.script_unit_matches(select, unit))
                        .filter(|unit| {
                            marker.visible_in_fog
                                || self.unit_visible_to(&objective.faction, unit, None)
                        })
                        .map(|unit| self.position.runtime.board.ids[unit.tile].clone())
                        .collect()
                } else {
                    marker
                        .tiles
                        .iter()
                        .map(|name| {
                            self.position.runtime.board.ids[self.reference_site(name)].clone()
                        })
                        .collect()
                };
                destinations
                    .into_iter()
                    .filter(|tile| {
                        marker.visible_in_fog
                            || super::conditions::LiveContext::for_faction(
                                self,
                                &objective.faction,
                                None,
                            )
                            .visible_at(self.position.runtime.board.positions[self.site(tile)])
                    })
                    .collect()
            });
        ObjectiveStatus {
            active_tiles: if !done
                && !failed
                && visible
                && available
                && self.position.runtime.winner.is_none()
            {
                markers
            } else {
                vec![]
            },
            visible,
            available,
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
        crate::data::catalog::identifier(&mark.kind, "saved mark").map_err(|e| e.message)?;
        if let Some(sight) = &mark.sight {
            if !battle
                .vision_groups
                .iter()
                .any(|group| group.id == sight.group)
            {
                return Err("unknown saved mark sight group".into());
            }
            validate_unit_perception(&[], &sight.profile.penetrates)?;
        }
        if let Some(light) = &mark.light {
            light.validate(&battle.tiles, &[])?;
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
