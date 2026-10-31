//! Campaign objectives, scripted events, reinforcement waves and chapter transitions.

use crate::{data::*, logic::battle::Game};
use std::collections::BTreeMap;
impl<'catalog> Game<'catalog> {
    fn hold_actor_present(&self, goal: &Goal) -> bool {
        goal.companion().is_none_or(|name| {
            self.units.iter().any(|u| {
                u.side == Side::Player
                    && u.name.as_ref() == Some(name)
                    && self.board.ids[u.tile] == *goal.tile().unwrap()
            })
        })
    }
    pub(crate) fn goal_visible(&self, goal: &Goal) -> bool {
        goal.visible_when()
            .is_none_or(|condition| self.event_condition_met(condition))
    }
    /// Internal completion queries do not need labels or serialized tile lists.
    pub(crate) fn goal_done(&self, goal: &Goal) -> bool {
        let state = self.campaign.as_ref().unwrap();
        match goal {
            Goal::Defeat { .. } => !self
                .units
                .iter()
                .any(|u| self.hostile_to_phase(u, Side::Player) && u.role == Some(Role::Leader)),
            Goal::Escort { tile, .. } | Goal::Escape { tile, .. } => {
                self.units.iter().any(|u| {
                    u.side == Side::Player
                        && u.role
                            == Some(if matches!(goal, Goal::Escort { .. }) {
                                Role::Courier
                            } else {
                                Role::Leader
                            })
                        && &self.board.ids[u.tile] == tile
                }) || state.evacuated.iter().any(|record| {
                    record.unit.side == Side::Player
                        && record.unit.role
                            == Some(if matches!(goal, Goal::Escort { .. }) {
                                Role::Courier
                            } else {
                                Role::Leader
                            })
                        && &record.unit.tile == tile
                })
            }
            Goal::Control { .. } => goal
                .tiles()
                .iter()
                .all(|tile| self.owners[self.site(tile)] == Some(Side::Player)),
            Goal::Seal { .. } => goal
                .tiles()
                .iter()
                .all(|tile| state.marked_prefixed("sealed:", tile)),
            Goal::Event { event, .. } => state.marked_prefixed("event:", event),
            Goal::Hold {
                tile,
                turns,
                earliest_round,
                start_event,
                ..
            } => {
                let started = start_event
                    .as_ref()
                    .is_none_or(|event| state.marked_prefixed("event:", event));
                let held = if started {
                    state
                        .holds
                        .as_ref()
                        .and_then(|holds| holds.get(tile))
                        .map_or(0, |counter| counter.held_turns)
                } else {
                    0
                };
                held >= *turns
                    && self.turn == Side::Player
                    && self.round >= *earliest_round
                    && self.owners[self.site(tile)] == Some(Side::Player)
                    && self.hold_actor_present(goal)
            }
            Goal::Vigil { turns, .. } => {
                state.vigil.as_ref().map_or(0, |counter| counter.held_turns) >= *turns
            }
            Goal::Refuge { turns, .. } => {
                if state.escalated() {
                    return !self
                        .units
                        .iter()
                        .any(|u| self.hostile_to_phase(u, Side::Player));
                }
                let refuge = self.mission.as_ref().unwrap().refuge.as_ref().unwrap();
                let total = if state.marked_prefixed("event:", &refuge.medicine_event) {
                    refuge.medicine_turns
                } else {
                    *turns
                };
                state
                    .refuge
                    .as_ref()
                    .map_or(0, |counter| counter.held_turns)
                    >= total
            }
        }
    }
    pub(crate) fn goal_progress(&self, goal: &Goal) -> Progress {
        let mut current = 0;
        let mut total = 1;
        let mut label = goal.label().to_owned();
        let mut tiles = goal.sites();
        let state = self.campaign.as_ref().unwrap();
        match goal {
            Goal::Defeat { .. } => {
                tiles.clear();
                current = (!self.units.iter().any(|u| {
                    self.hostile_to_phase(u, Side::Player) && u.role == Some(Role::Leader)
                })) as u32;
            }
            Goal::Escort { .. } | Goal::Escape { .. } => {
                current = self.goal_done(goal) as u32;
            }
            Goal::Control { .. } => {
                total = tiles.len() as u32;
                current = tiles
                    .iter()
                    .filter(|t| self.owners[self.site(t)] == Some(Side::Player))
                    .count() as u32;
            }
            Goal::Seal { .. } => {
                total = tiles.len() as u32;
                current = tiles
                    .iter()
                    .filter(|t| state.marked(&format!("sealed:{t}")))
                    .count() as u32;
            }
            Goal::Event { event, .. } => current = state.marked(&format!("event:{}", event)) as u32,
            Goal::Hold {
                tile,
                turns,
                earliest_round,
                start_event,
                ..
            } => {
                total = *turns;
                if start_event
                    .as_ref()
                    .is_some_and(|e| !state.marked(&format!("event:{e}")))
                {
                    label = if self.owners[self.site(tile)] != Some(Side::Player) {
                        "Reach the wreck to start the rescue".into()
                    } else {
                        format!("Wait for high tide on turn {}", *earliest_round)
                    };
                } else {
                    current = state
                        .holds
                        .as_ref()
                        .and_then(|h| h.get(tile))
                        .map_or(0, |h| h.held_turns)
                        .min(total);
                }
            }
            Goal::Vigil { turns, .. } => {
                total = *turns;
                current = state.vigil.as_ref().map_or(0, |p| p.held_turns).min(total);
            }
            Goal::Refuge { turns, .. } => {
                if state.escalated() {
                    label = "Defeat the remaining patrol".into();
                    current = (!self
                        .units
                        .iter()
                        .any(|u| self.hostile_to_phase(u, Side::Player)))
                        as u32;
                    tiles.clear();
                } else {
                    let refuge = self.mission.as_ref().unwrap().refuge.as_ref().unwrap();
                    total = if state.marked(&format!("event:{}", refuge.medicine_event)) {
                        refuge.medicine_turns
                    } else {
                        *turns
                    };
                    current = state.refuge.as_ref().map_or(0, |p| p.held_turns).min(total);
                }
            }
        }
        let done = current == total
            && match goal {
                Goal::Hold {
                    tile,
                    earliest_round,
                    ..
                } => {
                    self.turn == Side::Player
                        && self.round >= *earliest_round
                        && self.owners[self.site(tile)] == Some(Side::Player)
                        && self.hold_actor_present(goal)
                }
                _ => true,
            };
        Progress {
            label,
            current,
            total,
            done,
            tiles,
        }
    }
    pub(crate) fn active_goal_tiles(&self, goal: &Goal) -> Vec<String> {
        if self.winner.is_some() || !self.goal_visible(goal) {
            return vec![];
        }
        if goal.kind() == GoalKind::Seal {
            return goal
                .tiles()
                .iter()
                .filter(|t| {
                    !self
                        .campaign
                        .as_ref()
                        .unwrap()
                        .marked(&format!("sealed:{t}"))
                })
                .cloned()
                .collect();
        }
        if goal.kind() == GoalKind::Event && self.goal_done(goal) {
            return vec![];
        }
        if goal.kind() == GoalKind::Defeat {
            vec![]
        } else {
            goal.sites()
        }
    }
    pub(crate) fn escalate_refuge(&mut self) {
        let Some(refuge) = self.mission.as_ref().and_then(|m| m.refuge.clone()) else {
            return;
        };
        let Some(state) = &mut self.campaign else {
            return;
        };
        let counter = state.refuge.get_or_insert(RefugeCounter {
            held_turns: 0,
            last_counted_round: self.round,
            escalated: false,
        });
        if counter.escalated {
            return;
        }
        counter.escalated = true;
        state.say(refuge.escalation);
        self.record("The chance of mediation is lost. Defeat the remaining patrol.".into());
        self.refresh_mission();
    }
    pub(crate) fn spawn_waves(&mut self) {
        if self.campaign.is_none() || self.winner.is_some() {
            return;
        }
        let mission = self.mission.clone().unwrap();
        for wave in &mission.waves {
            let marker = format!("wave:{}", wave.id);
            if self.round < wave.round || self.campaign.as_ref().unwrap().marked(&marker) {
                continue;
            }
            self.campaign.as_mut().unwrap().marks.push(marker);
            if wave.requires_unsealed.as_ref().is_some_and(|t| {
                self.campaign
                    .as_ref()
                    .unwrap()
                    .marked(&format!("sealed:{t}"))
            }) {
                continue;
            }
            let mut arrived = 0;
            for incoming in &wave.units {
                let origin = self.site(&format!("{},{}", incoming.col, incoming.row));
                let kind = self.kind(&incoming.r#type);
                let tile = (0..self.terrain.len())
                    .filter(|&t| {
                        self.cost(kind, t) != u16::MAX
                            && self.at(t).is_none()
                            && self.board.distance(t, origin) <= 2
                    })
                    .min_by_key(|&t| self.board.distance(t, origin));
                if let Some(tile) = tile {
                    let id = self.add(kind, incoming.side, tile);
                    let unit = self.units.iter_mut().find(|unit| unit.id == id).unwrap();
                    unit.ai_group = incoming.ai_group.clone();
                    unit.items = incoming.items.clone();
                    unit.faction = incoming.faction.clone();
                    arrived += 1;
                }
            }
            if arrived > 0 {
                self.record(wave.label.clone());
            }
        }
    }
    /// Activation is remembered independently from placement and each member is released once.
    pub(crate) fn spawn_events(&mut self) {
        let Some(mission) = self.mission.clone() else {
            return;
        };
        for event in &mission.events {
            let Some(group) = &event.spawn else {
                continue;
            };
            if !self
                .campaign
                .as_ref()
                .is_some_and(|state| state.marked(&format!("event:{}", event.id)))
            {
                continue;
            }
            for incoming in &group.units {
                let marker = format!("spawn:{}:{}", event.id, incoming.id);
                if self.campaign.as_ref().unwrap().marked(&marker) {
                    continue;
                }
                let kind = self.kind(&incoming.r#type);
                let tile = group
                    .area
                    .iter()
                    .map(|id| self.site(id))
                    .find(|&tile| self.at(tile).is_none() && self.cost(kind, tile) != u16::MAX);
                if let Some(tile) = tile {
                    let id = self.add(kind, incoming.side, tile);
                    let unit = self.units.iter_mut().find(|unit| unit.id == id).unwrap();
                    unit.ai_group = group.ai_group.clone();
                    unit.faction = incoming.faction.clone().or_else(|| group.faction.clone());
                    unit.name = incoming.name.clone();
                    unit.role = incoming.role.clone();
                    self.campaign.as_mut().unwrap().marks.push(marker);
                }
            }
        }
    }
    pub(crate) fn evaluate_campaign(&mut self, at_turn_start: bool, entered: Option<usize>) {
        if self.campaign.is_none() || self.winner.is_some() {
            return;
        }
        self.apply_faction_transfers();
        let mission = self.mission.clone().unwrap();
        let leader = self
            .units
            .iter()
            .any(|u| u.side == Side::Player && u.role == Some(Role::Leader))
            || self
                .campaign
                .as_ref()
                .unwrap()
                .evacuated
                .iter()
                .any(|record| {
                    record.unit.side == Side::Player && record.unit.role == Some(Role::Leader)
                });
        let missing = mission.required_companions.iter().find(|name| {
            !self
                .units
                .iter()
                .any(|u| u.side == Side::Player && u.name.as_ref() == Some(name))
                && !self
                    .campaign
                    .as_ref()
                    .unwrap()
                    .evacuated
                    .iter()
                    .any(|record| record.unit.name.as_ref() == Some(name))
        });
        let courier = self
            .units
            .iter()
            .any(|u| !self.hostile_to_phase(u, Side::Player) && u.role == Some(Role::Courier))
            || mission
                .goals
                .iter()
                .filter(|goal| matches!(goal, Goal::Escort { .. }))
                .all(|goal| self.goal_done(goal));
        if !leader
            || missing.is_some()
            || (mission.goals.iter().any(|g| g.kind() == GoalKind::Escort) && !courier)
        {
            let text = if !leader {
                "Your leader fell. The company cannot continue without them.".into()
            } else if let Some(name) = missing {
                format!("{name} fell. The company cannot continue without them.")
            } else {
                "The people you were escorting were killed before reaching safety.".into()
            };
            self.winner = Some(Side::Enemy);
            let state = self.campaign.as_mut().unwrap();
            state.outcome = Some(text.clone());
            state.say([Dialogue {
                speaker: "The company".into(),
                portrait: self.rules.campaigns[&state.id].leader.clone(),
                text: text.clone(),
            }]);
            self.record(text);
            return;
        }
        if let Some(goal) = mission.all_goals().find(|g| g.kind() == GoalKind::Vigil) {
            let tile = self.site(goal.tile().unwrap());
            let present = self.units.iter().any(|u| {
                u.side == Side::Player && u.name.as_ref() == goal.companion() && u.tile == tile
            });
            let counter = self
                .campaign
                .as_mut()
                .unwrap()
                .vigil
                .get_or_insert(Counter {
                    held_turns: 0,
                    last_counted_round: self.round,
                });
            if !present || self.owners[tile] != Some(Side::Player) {
                counter.held_turns = 0;
                counter.last_counted_round = self.round;
            } else if at_turn_start && self.round > counter.last_counted_round {
                counter.held_turns = (counter.held_turns + 1).min(goal.turns().unwrap());
                counter.last_counted_round = self.round;
            }
        }
        if mission.refuge.is_some() {
            self.campaign
                .as_mut()
                .unwrap()
                .refuge
                .get_or_insert(RefugeCounter {
                    held_turns: 0,
                    last_counted_round: self.round,
                    escalated: false,
                });
            if !self
                .units
                .iter()
                .any(|u| self.hostile_to_phase(u, Side::Player) && u.role == Some(Role::Leader))
            {
                self.escalate_refuge();
            }
            if let Some(goal) = mission.all_goals().find(|g| g.kind() == GoalKind::Refuge) {
                let tile = self.site(goal.tile().unwrap());
                let counter = self.campaign.as_mut().unwrap().refuge.as_mut().unwrap();
                if !counter.escalated {
                    if self.owners[tile] != Some(Side::Player) {
                        counter.held_turns = 0;
                    } else if at_turn_start && self.round > counter.last_counted_round {
                        counter.held_turns += 1;
                    }
                    if at_turn_start {
                        counter.last_counted_round = self.round;
                    }
                }
            }
        }
        for goal in mission.all_goals() {
            if goal.kind() == GoalKind::Hold {
                let tile = self.site(goal.tile().unwrap());
                let actor_present = self.hold_actor_present(goal);
                let state = self.campaign.as_mut().unwrap();
                let started = goal
                    .start_event()
                    .as_ref()
                    .is_none_or(|e| state.marked(&format!("event:{e}")));
                let counter = state
                    .holds
                    .get_or_insert_with(BTreeMap::new)
                    .entry(goal.tile().unwrap().clone())
                    .or_insert(Counter {
                        held_turns: 0,
                        last_counted_round: self.round,
                    });
                if !started || !actor_present || self.owners[tile] != Some(Side::Player) {
                    counter.held_turns = 0;
                    counter.last_counted_round = self.round;
                } else if at_turn_start && self.round > counter.last_counted_round {
                    counter.held_turns = (counter.held_turns + 1).min(goal.turns().unwrap());
                    counter.last_counted_round = self.round;
                }
            }
            if goal.kind() != GoalKind::Seal {
                continue;
            }
            for id in goal.tiles() {
                let mark = format!("sealed:{id}");
                if self.campaign.as_ref().unwrap().marked(&mark) {
                    continue;
                }
                let eligible = self.at(self.site(id)).is_some_and(|u| {
                    u.side == Side::Player
                        && goal.unit_type().is_none_or(|k| {
                            self.rules.units[u.kind].id == *k
                                || self.rules.units[u.kind].family
                                    == self.rules.units[self.kind(k)].family
                        })
                });
                if eligible {
                    self.campaign.as_mut().unwrap().marks.push(mark);
                    self.record(
                        "A binding stone was sealed. It will no longer raise the dead.".into(),
                    );
                }
            }
        }
        if self.turn == Side::Player {
            for event in &mission.events {
                let marker = format!("event:{}", event.id);
                let state = self.campaign.as_ref().unwrap();
                let triggered = match &event.trigger {
                    Trigger::Condition { condition } => self.event_condition_met(condition),
                    Trigger::Visit { tile } => entered.is_some_and(|t| &self.board.ids[t] == tile),
                    Trigger::Round { round } => self.round >= *round,
                    Trigger::Control { tile, round } => {
                        self.owners[self.site(tile)] == Some(Side::Player)
                            && self.round >= round.unwrap_or(1)
                    }
                };
                if !triggered
                    || event.evacuate.as_ref().is_some_and(|effect| {
                        !self.units.iter().any(|unit| {
                            effect.units.accepts(unit) && self.board.ids[unit.tile] == effect.tile
                        })
                    })
                    || event
                        .items
                        .iter()
                        .any(|reward| !self.units.iter().any(|unit| reward.recipient.accepts(unit)))
                    || state.marked(&marker)
                    || event
                        .requires_event
                        .as_ref()
                        .is_some_and(|e| !state.marked(&format!("event:{e}")))
                    || event
                        .unless_event
                        .as_ref()
                        .is_some_and(|e| state.marked(&format!("event:{e}")))
                    || event.before_round.is_some_and(|r| self.round >= r)
                {
                    continue;
                }
                let lines = if state.escalated() {
                    event.war_dialogue.as_ref().unwrap_or(&event.dialogue)
                } else {
                    &event.dialogue
                };
                // Named party members speak only while present, including after promotion.
                // Unbound speakers such as local witnesses and narration remain available.
                let lines: Vec<_> = lines
                    .iter()
                    .filter(|line| !state.marked(&format!("fallen:{}", line.speaker)))
                    .filter(|line| {
                        !mission.map.units.iter().any(|unit| {
                            unit.side == Side::Player
                                && unit.name.as_deref() == Some(line.speaker.as_str())
                        }) || self.units.iter().any(|unit| {
                            unit.side == Side::Player
                                && unit.name.as_deref() == Some(line.speaker.as_str())
                        })
                    })
                    .cloned()
                    .collect();
                let evacuated: Vec<_> = event
                    .evacuate
                    .as_ref()
                    .map(|effect| {
                        self.units
                            .iter()
                            .filter(|unit| {
                                effect.units.accepts(unit)
                                    && self.board.ids[unit.tile] == effect.tile
                            })
                            .map(|unit| EvacuatedUnit {
                                event: event.id.clone(),
                                unit: self.public_unit(unit),
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if !evacuated.is_empty() {
                    self.units.retain(|unit| {
                        !evacuated
                            .iter()
                            .any(|record| record.unit.id == format!("u{}", unit.id))
                    });
                    self.roster_changed();
                }
                let state = self.campaign.as_mut().unwrap();
                state.evacuated.extend(evacuated);
                state.marks.push(marker);
                if let Some(choice) = &event.choice {
                    let choices = state.choices.get_or_insert_with(Vec::new);
                    if !choices.contains(choice) {
                        choices.push(choice.clone());
                    }
                }
                self.gold[0] += event.gold.unwrap_or(0);
                state.say(lines);
                for reward in &event.items {
                    for unit in self
                        .units
                        .iter_mut()
                        .filter(|unit| reward.recipient.accepts(unit))
                    {
                        if !unit.items.iter().any(|item| item.id == reward.item.id) {
                            unit.items.push(reward.item.clone());
                        }
                    }
                }
                self.record(
                    event
                        .label
                        .clone()
                        .unwrap_or("The company found a new story.".into()),
                );
            }
        }
        self.spawn_events();
        self.apply_faction_transfers();
        if mission.goals.iter().all(|g| self.goal_done(g)) {
            self.winner = Some(Side::Player);
            let dain_survived = self
                .units
                .iter()
                .any(|u| u.side == Side::Player && u.name.as_deref() == Some("Captain Dain"));
            let state = self.campaign.as_mut().unwrap();
            state.outcome = Some("Mission complete. The company can rest and regroup.".into());
            let war = state.escalated();
            if mission.refuge.is_some() {
                state
                    .choices
                    .get_or_insert_with(Vec::new)
                    .push(if war { "refuge-war" } else { "refuge-peace" }.into());
            }
            state.say(if mission.refuge.is_some() && war {
                mission.refuge.as_ref().unwrap().war_victory.clone()
            } else {
                mission.victory.clone()
            });
            let definition = &self.rules.campaigns[&state.id];
            state.completed = state.mission == definition.missions.len() - 1;
            if state.completed {
                let survived = dain_survived
                    && !state.chosen("refuge-war")
                    && !state.chosen("green-dain-dead")
                    && !state.marked("dain-fell");
                state.say(
                    definition
                        .ending
                        .iter()
                        .filter(|line| {
                            survived || (line.speaker != "Dain" && line.speaker != "Captain Dain")
                        })
                        .cloned(),
                );
            }
            let text = if state.completed {
                format!("{} is complete.", definition.name)
            } else {
                "Mission complete.".into()
            };
            self.record(text);
        }
    }
    pub(crate) fn start_campaign(
        rules: &'catalog Rules,
        id: &str,
        index: usize,
        veterans: Vec<Veteran>,
        carry: i32,
        choices: Vec<String>,
        story: Vec<Veteran>,
        fallen: Vec<String>,
    ) -> Self {
        let original = rules.campaigns[id].missions[index].clone();
        let mut game = Game::new(rules, &original.map.id);
        let holds = original
            .all_goals()
            .filter(|g| g.kind() == GoalKind::Hold)
            .map(|g| {
                (
                    g.tile().unwrap().clone(),
                    Counter {
                        held_turns: 0,
                        last_counted_round: 1,
                    },
                )
            })
            .collect();
        game.campaign = Some(CampaignState {
            evacuated: vec![],
            id: id.into(),
            mission: index,
            recall: veterans.iter().map(|v| v.r#type.clone()).collect(),
            veterans: Some(veterans.clone()),
            story_veterans: Some(story.clone()),
            fallen_companions: Some(fallen.clone()),
            carry_gold: carry.clamp(0, 8),
            marks: vec![],
            dialogue: original.intro.clone(),
            dialogue_index: 0,
            outcome: None,
            completed: false,
            refuge: original.refuge.as_ref().map(|_| RefugeCounter {
                held_turns: 0,
                last_counted_round: 1,
                escalated: false,
            }),
            vigil: original
                .all_goals()
                .any(|g| g.kind() == GoalKind::Vigil)
                .then_some(Counter {
                    held_turns: 0,
                    last_counted_round: 1,
                }),
            holds: Some(holds),
            choices: (!choices.is_empty()).then_some(choices.clone()),
        });
        game.refresh_mission();
        game.units.retain(|u| {
            u.side != Side::Player
                || u.role == Some(Role::Leader)
                || u.name.as_ref().is_none_or(|n| !fallen.contains(n))
        });
        game.roster_changed();
        game.campaign.as_mut().unwrap().dialogue = game.mission.as_ref().unwrap().intro.clone();
        if (choices
            .iter()
            .any(|c| c == "refuge-war" || c == "green-dain-dead"))
            && original.without_dain.is_some()
        {
            game.units
                .retain(|u| u.name.as_deref() != Some("Captain Dain"));
            for incoming in original
                .without_dain
                .as_ref()
                .unwrap()
                .reinforcements
                .as_ref()
                .unwrap()
            {
                let kind = game.kind(&incoming.r#type);
                let tile = game.site(&format!("{},{}", incoming.col, incoming.row));
                game.add(kind, incoming.side, tile);
                let added = game.units.last_mut().unwrap();
                added.ai_group = incoming.ai_group.clone();
                added.items = incoming.items.clone();
                added.faction = incoming.faction.clone();
                added.name = incoming.name.clone();
                added.role = incoming.role.clone();
            }
        }
        game.gold = [original.gold + carry.clamp(0, 8), original.enemy_gold];
        for veteran in story {
            let kind = game.kind(&veteran.r#type);
            if let Some(unit) = game.units.iter_mut().find(|u| {
                u.side == Side::Player
                    && if veteran.role == Some(Role::Leader) {
                        u.role == Some(Role::Leader)
                    } else {
                        veteran.name.is_some() && u.name == veteran.name
                    }
            }) {
                if game.rules.units[unit.kind].family == game.rules.units[kind].family {
                    unit.items = veteran.items.clone();
                    unit.kind = kind;
                    unit.experience = Some(veteran.experience);
                    unit.hp = game.rules.units[kind].hp;
                }
            }
        }
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player && u.role == Some(Role::Leader))
            .unwrap()
            .tile;
        game.roster_changed();
        let mut slots: Vec<_> = (0..game.terrain.len())
            .filter(|&t| game.at(t).is_none() && game.board.distance(t, leader) <= 3)
            .collect();
        slots.sort_by_key(|&t| game.board.distance(t, leader));
        for veteran in veterans {
            let kind = game.kind(&veteran.r#type);
            let Some(slot) = slots.iter().position(|&t| game.cost(kind, t) != u16::MAX) else {
                // A changed map may not have a passable recall tile for this family.
                game.record(format!(
                    "No safe recall space for {}.",
                    game.rules.units[kind].name
                ));
                continue;
            };
            let tile = slots.remove(slot);
            game.add(kind, Side::Player, tile);
            let unit = game.units.last_mut().unwrap();
            unit.experience = Some(veteran.experience);
            unit.name = veteran.name;
            unit.items = veteran.items;
        }
        for unit in &game.units {
            if [
                Terrain::Village,
                Terrain::Grove,
                Terrain::Ruins,
                Terrain::Fort,
            ]
            .iter()
            .any(|t| game.terrain_is(unit.tile, *t))
            {
                game.owners[unit.tile] = Some(unit.side);
            }
        }
        game.log.clear();
        game.record(game.mission.as_ref().unwrap().briefing.clone());
        game.evaluate_campaign(false, None);
        game.remember_conditions();
        game
    }
    pub(crate) fn returns_with_story(&self, u: &Unit) -> bool {
        let state = self.campaign.as_ref().unwrap();
        self.rules.campaigns[&state.id]
            .missions
            .get(state.mission + 1)
            .is_some_and(|mission| {
                mission.map.units.iter().any(|incoming| {
                    incoming.side == Side::Player
                        && self.rules.units[self.kind(&incoming.r#type)].family
                            == self.rules.units[u.kind].family
                        && if u.role == Some(Role::Leader) {
                            incoming.role == Some(Role::Leader)
                        } else {
                            u.name.is_some() && incoming.name == u.name
                        }
                })
            })
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn next_campaign(&self) -> Self {
        self.continue_campaign(None)
            .expect("automatic recall is valid")
    }

    pub(crate) fn continue_campaign(
        &self,
        selected: Option<&[UnitId]>,
    ) -> Result<Self, CommandError> {
        let state = self.campaign.as_ref().unwrap();
        let mission = self.mission.as_ref().unwrap();
        let mut eligible: Vec<_> = self
            .units
            .iter()
            .filter(|u| {
                u.side == Side::Player
                    && !u.is_leader()
                    && u.role != Some(Role::Courier)
                    && u.name
                        .as_ref()
                        .is_none_or(|n| !mission.required_companions.contains(n))
                    && !self.returns_with_story(u)
            })
            .collect();
        let value = |u: &Unit| {
            let r = &self.rules.units[u.kind];
            r.hp as f64
                + r.damage as f64 * 2.0
                + r.healing as f64 * 5.0
                + r.level as f64 * 3.0
                + u.experience.unwrap_or(0) as f64 / 3.0
        };
        eligible.sort_by(|a, b| value(b).total_cmp(&value(a)));
        if let Some(selected) = selected {
            let mut recalled = Vec::new();
            if selected.len() > 4 {
                return Err(CommandError::NoNextMission);
            }
            for id in selected {
                if recalled.iter().any(|u: &&Unit| u.id == id.0) {
                    return Err(CommandError::NoNextMission);
                }
                recalled.push(
                    *eligible
                        .iter()
                        .find(|u| u.id == id.0)
                        .ok_or(CommandError::NoNextMission)?,
                );
            }
            eligible = recalled;
        }
        let veteran = |u: &Unit, role: bool| Veteran {
            items: u.items.clone(),
            r#type: self.rules.units[u.kind].id.clone(),
            experience: u.experience.unwrap_or(0),
            name: u.name.clone(),
            role: if role { u.role.clone() } else { None },
        };
        eligible.truncate(4);
        let overflow: Vec<_> = self
            .units
            .iter()
            .filter(|unit| {
                unit.side == Side::Player
                    && !self.returns_with_story(unit)
                    && !eligible.iter().any(|selected| selected.id == unit.id)
            })
            .flat_map(|unit| unit.items.iter().cloned())
            .collect();
        let veterans = eligible.into_iter().map(|u| veteran(u, false)).collect();
        let mut story: Vec<Veteran> = self
            .units
            .iter()
            .filter(|u| u.side == Side::Player && self.returns_with_story(u))
            .map(|u| veteran(u, true))
            .collect();
        if let Some(next) = self.rules.campaigns[&state.id]
            .missions
            .get(state.mission + 1)
        {
            for record in &state.evacuated {
                let unit = &record.unit;
                if unit.side == Side::Player
                    && next.map.units.iter().any(|incoming| {
                        incoming.side == Side::Player
                            && self.rules.units[self.kind(&incoming.r#type)].family
                                == self.rules.units[self.kind(&unit.r#type)].family
                            && if unit.role == Some(Role::Leader) {
                                incoming.role == Some(Role::Leader)
                            } else {
                                unit.name.is_some() && incoming.name == unit.name
                            }
                    })
                {
                    story.push(Veteran {
                        r#type: unit.r#type.clone(),
                        items: unit.items.clone(),
                        experience: unit.experience.unwrap_or(0),
                        name: unit.name.clone(),
                        role: unit.role,
                    });
                }
            }
        }
        let mut choices = state.choices.clone().unwrap_or_default();
        if state.marked("dain-fell") && !choices.iter().any(|c| c == "green-dain-dead") {
            choices.push("green-dain-dead".into());
        }
        let mut fallen = state.fallen_companions.clone().unwrap_or_default();
        for name in state
            .marks
            .iter()
            .filter_map(|m| m.strip_prefix("fallen:").map(str::to_owned))
            .chain(
                mission
                    .map
                    .units
                    .iter()
                    .filter(|u| {
                        u.side == Side::Player
                            && u.name.is_some()
                            && u.role != Some(Role::Leader)
                            && !self
                                .units
                                .iter()
                                .any(|alive| alive.side == Side::Player && alive.name == u.name)
                            && !state
                                .evacuated
                                .iter()
                                .any(|record| record.unit.name == u.name)
                    })
                    .map(|u| u.name.clone().unwrap()),
            )
        {
            if !fallen.contains(&name) {
                fallen.push(name);
            }
        }
        let mut next = Self::start_campaign(
            self.rules,
            &state.id,
            state.mission + 1,
            veterans,
            self.gold[0],
            choices,
            story,
            fallen,
        );
        if !overflow.is_empty() {
            let leader = next
                .units
                .iter_mut()
                .find(|unit| unit.side == Side::Player && unit.is_leader())
                .unwrap();
            leader.items.extend(overflow);
            // Retry must restore the same equipment as the chapter's first entry.
            let leader = veteran(leader, true);
            let story = next
                .campaign
                .as_mut()
                .unwrap()
                .story_veterans
                .get_or_insert_default();
            if let Some(saved) = story
                .iter_mut()
                .find(|unit| unit.role == Some(Role::Leader))
            {
                *saved = leader;
            } else {
                story.push(leader);
            }
        }
        next.combat_rng = self.combat_rng;
        Ok(next)
    }
}
