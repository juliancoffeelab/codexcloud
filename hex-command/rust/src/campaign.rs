use crate::{engine::Game, model::*};
use std::{collections::BTreeMap, sync::Arc};
impl Game {
    pub fn goal_progress(&self, goal: &Goal) -> Progress {
        let mut current = 0;
        let mut total = 1;
        let mut label = goal.label.clone();
        let mut tiles = goal.sites();
        let state = self.campaign.as_ref().unwrap();
        match goal.kind.as_str() {
            "defeat" => {
                tiles.clear();
                current = (!self
                    .units
                    .iter()
                    .any(|u| u.side == Side::Enemy && u.role.as_deref() == Some("leader")))
                    as u32;
            }
            "escort" | "escape" => {
                current = self.units.iter().any(|u| {
                    u.side == Side::Player
                        && u.role.as_deref()
                            == Some(if goal.kind == "escort" {
                                "courier"
                            } else {
                                "leader"
                            })
                        && Some(&self.board.ids[u.tile]) == goal.tile.as_ref()
                }) as u32
            }
            "control" => {
                total = tiles.len() as u32;
                current = tiles
                    .iter()
                    .filter(|t| self.owners[self.site(t)] == Some(Side::Player))
                    .count() as u32;
            }
            "seal" => {
                total = tiles.len() as u32;
                current = tiles
                    .iter()
                    .filter(|t| state.marked(&format!("sealed:{t}")))
                    .count() as u32;
            }
            "event" => {
                current = state.marked(&format!("event:{}", goal.event.as_deref().unwrap())) as u32
            }
            "hold" => {
                total = goal.turns.unwrap();
                if goal
                    .start_event
                    .as_ref()
                    .is_some_and(|e| !state.marked(&format!("event:{e}")))
                {
                    label = if self.owners[self.site(goal.tile.as_ref().unwrap())]
                        != Some(Side::Player)
                    {
                        "Reach the wreck to start the rescue".into()
                    } else {
                        format!(
                            "Wait for high tide on turn {}",
                            goal.earliest_round.unwrap()
                        )
                    };
                } else {
                    current = state
                        .holds
                        .as_ref()
                        .and_then(|h| h.get(goal.tile.as_ref().unwrap()))
                        .map_or(0, |h| h.held_turns)
                        .min(total);
                }
            }
            "vigil" => {
                total = goal.turns.unwrap();
                current = state.vigil.as_ref().map_or(0, |p| p.held_turns).min(total);
            }
            "refuge" => {
                if state.escalated() {
                    label = "Defeat the remaining patrol".into();
                    current = (!self.units.iter().any(|u| u.side == Side::Enemy)) as u32;
                    tiles.clear();
                } else {
                    let refuge = self.mission.as_ref().unwrap().refuge.as_ref().unwrap();
                    total = if state.marked(&format!("event:{}", refuge.medicine_event)) {
                        refuge.medicine_turns
                    } else {
                        goal.turns.unwrap()
                    };
                    current = state.refuge.as_ref().map_or(0, |p| p.held_turns).min(total);
                }
            }
            _ => panic!("unknown goal"),
        }
        let done = current == total
            && (goal.kind != "hold"
                || (self.turn == Side::Player
                    && self.round >= goal.earliest_round.unwrap()
                    && self.owners[self.site(goal.tile.as_ref().unwrap())] == Some(Side::Player)));
        Progress {
            label,
            current,
            total,
            done,
            tiles,
        }
    }
    pub fn active_goal_tiles(&self, goal: &Goal) -> Vec<String> {
        if self.winner.is_some() {
            return vec![];
        }
        if goal.kind == "seal" {
            return goal
                .tiles
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
        if goal.kind == "event" && self.goal_progress(goal).done {
            return vec![];
        }
        if goal.kind == "defeat" {
            vec![]
        } else {
            goal.sites()
        }
    }
    pub fn escalate_refuge(&mut self) {
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
    pub fn spawn_waves(&mut self) {
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
                    self.add(kind, incoming.side, tile);
                    arrived += 1;
                }
            }
            if arrived > 0 {
                self.record(wave.label.clone());
            }
        }
    }
    pub fn evaluate_campaign(&mut self, at_turn_start: bool, entered: Option<usize>) {
        if self.campaign.is_none() || self.winner.is_some() {
            return;
        }
        let mission = self.mission.clone().unwrap();
        let leader = self
            .units
            .iter()
            .any(|u| u.side == Side::Player && u.role.as_deref() == Some("leader"));
        let missing = mission.required_companions.iter().find(|name| {
            !self
                .units
                .iter()
                .any(|u| u.side == Side::Player && u.name.as_ref() == Some(name))
        });
        let courier = self
            .units
            .iter()
            .any(|u| u.side == Side::Player && u.role.as_deref() == Some("courier"));
        if !leader
            || missing.is_some()
            || (mission.goals.iter().any(|g| g.kind == "escort") && !courier)
        {
            let text = if !leader {
                "Your leader fell. The company cannot continue without them.".into()
            } else if let Some(name) = missing {
                format!("{name} fell. The company cannot continue without them.")
            } else {
                "The courier fell before reaching safety.".into()
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
        if let Some(goal) = mission.goals.iter().find(|g| g.kind == "vigil") {
            let tile = self.site(goal.tile.as_ref().unwrap());
            let present = self
                .units
                .iter()
                .any(|u| u.side == Side::Player && u.name == goal.companion && u.tile == tile);
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
                counter.held_turns = (counter.held_turns + 1).min(goal.turns.unwrap());
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
                .any(|u| u.side == Side::Enemy && u.role.as_deref() == Some("leader"))
            {
                self.escalate_refuge();
            }
            if let Some(goal) = mission.goals.iter().find(|g| g.kind == "refuge") {
                let tile = self.site(goal.tile.as_ref().unwrap());
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
        for goal in &mission.goals {
            if goal.kind == "hold" {
                let tile = self.site(goal.tile.as_ref().unwrap());
                let state = self.campaign.as_mut().unwrap();
                let started = goal
                    .start_event
                    .as_ref()
                    .is_none_or(|e| state.marked(&format!("event:{e}")));
                let counter = state
                    .holds
                    .get_or_insert_with(BTreeMap::new)
                    .entry(goal.tile.clone().unwrap())
                    .or_insert(Counter {
                        held_turns: 0,
                        last_counted_round: self.round,
                    });
                if !started || self.owners[tile] != Some(Side::Player) {
                    counter.held_turns = 0;
                    counter.last_counted_round = self.round;
                } else if at_turn_start && self.round > counter.last_counted_round {
                    counter.held_turns = (counter.held_turns + 1).min(goal.turns.unwrap());
                    counter.last_counted_round = self.round;
                }
            }
            if goal.kind != "seal" {
                continue;
            }
            for id in &goal.tiles {
                let mark = format!("sealed:{id}");
                if self.campaign.as_ref().unwrap().marked(&mark) {
                    continue;
                }
                let eligible = self.at(self.site(id)).is_some_and(|u| {
                    u.side == Side::Player
                        && goal.unit_type.as_ref().is_none_or(|k| {
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
                let triggered = match event.trigger.kind.as_str() {
                    "visit" => entered
                        .is_some_and(|t| Some(&self.board.ids[t]) == event.trigger.tile.as_ref()),
                    "round" => self.round >= event.trigger.round.unwrap(),
                    _ => {
                        self.owners[self.site(event.trigger.tile.as_ref().unwrap())]
                            == Some(Side::Player)
                            && self.round >= event.trigger.round.unwrap_or(1)
                    }
                };
                if !triggered
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
                let state = self.campaign.as_mut().unwrap();
                state.marks.push(marker);
                self.gold[0] += event.gold.unwrap_or(0);
                let lines = if state.escalated() {
                    event.war_dialogue.as_ref().unwrap_or(&event.dialogue)
                } else {
                    &event.dialogue
                };
                state.say(lines.clone());
                self.record(
                    event
                        .label
                        .clone()
                        .unwrap_or("The company found a new story.".into()),
                );
            }
        }
        if mission.goals.iter().all(|g| self.goal_progress(g).done) {
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
    pub fn start_campaign(
        rules: Arc<Rules>,
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
            .goals
            .iter()
            .filter(|g| g.kind == "hold")
            .map(|g| {
                (
                    g.tile.clone().unwrap(),
                    Counter {
                        held_turns: 0,
                        last_counted_round: 1,
                    },
                )
            })
            .collect();
        game.campaign = Some(CampaignState {
            id: id.into(),
            mission: index,
            recall: veterans.iter().map(|v| v.r#type.clone()).collect(),
            veterans: Some(veterans.clone()),
            story_veterans: Some(story.clone()),
            fallen_companions: Some(fallen.clone()),
            carry_gold: carry.clamp(0, 8),
            marks: vec![],
            dialogue: Arc::new(original.intro.clone()),
            dialogue_index: 0,
            outcome: None,
            completed: false,
            refuge: original.refuge.as_ref().map(|_| RefugeCounter {
                held_turns: 0,
                last_counted_round: 1,
                escalated: false,
            }),
            vigil: original
                .goals
                .iter()
                .any(|g| g.kind == "vigil")
                .then_some(Counter {
                    held_turns: 0,
                    last_counted_round: 1,
                }),
            holds: Some(holds),
            choices: (!choices.is_empty()).then_some(choices.clone()),
        });
        game.refresh_mission();
        game.units.retain(|u| {
            u.side != Side::Player || u.name.as_ref().is_none_or(|n| !fallen.contains(n))
        });
        game.campaign.as_mut().unwrap().dialogue =
            Arc::new(game.mission.as_ref().unwrap().intro.clone());
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
                added.name = incoming.name.clone();
                added.role = incoming.role.clone();
            }
        }
        game.gold = [original.gold + carry.clamp(0, 8), original.enemy_gold];
        for veteran in story {
            let kind = game.kind(&veteran.r#type);
            if let Some(unit) = game.units.iter_mut().find(|u| {
                u.side == Side::Player
                    && if veteran.role.as_deref() == Some("leader") {
                        u.role.as_deref() == Some("leader")
                    } else {
                        veteran.name.is_some() && u.name == veteran.name
                    }
            }) {
                if game.rules.units[unit.kind].family == game.rules.units[kind].family {
                    unit.kind = kind;
                    unit.experience = Some(veteran.experience);
                    unit.hp = game.rules.units[kind].hp;
                }
            }
        }
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player && u.role.as_deref() == Some("leader"))
            .unwrap()
            .tile;
        let mut slots: Vec<_> = (0..game.terrain.len())
            .filter(|&t| game.at(t).is_none() && game.board.distance(t, leader) <= 3)
            .collect();
        slots.sort_by_key(|&t| game.board.distance(t, leader));
        for veteran in veterans {
            let kind = game.kind(&veteran.r#type);
            let slot = slots
                .iter()
                .position(|&t| game.cost(kind, t) != u16::MAX)
                .expect("recall space");
            let tile = slots.remove(slot);
            game.add(kind, Side::Player, tile);
            let unit = game.units.last_mut().unwrap();
            unit.experience = Some(veteran.experience);
            unit.name = veteran.name;
        }
        for unit in &game.units {
            if ["village", "grove", "ruins", "fort"]
                .iter()
                .any(|t| game.terrain_is(unit.tile, t))
            {
                game.owners[unit.tile] = Some(unit.side);
            }
        }
        game.log.clear();
        game.record(game.mission.as_ref().unwrap().briefing.clone());
        game
    }
    pub fn returns_with_story(&self, u: &Unit) -> bool {
        let state = self.campaign.as_ref().unwrap();
        u.name.as_ref().is_some_and(|name| {
            self.rules.campaigns[&state.id]
                .missions
                .get(state.mission + 1)
                .is_some_and(|m| {
                    m.map.units.iter().any(|incoming| {
                        incoming.side == Side::Player && incoming.name.as_ref() == Some(name)
                    })
                })
        })
    }
    pub fn next_campaign(&self) -> Self {
        let state = self.campaign.as_ref().unwrap();
        let mission = self.mission.as_ref().unwrap();
        let mut eligible: Vec<_> = self
            .units
            .iter()
            .filter(|u| {
                u.side == Side::Player
                    && !self.rules.units[u.kind].leader
                    && u.role.as_deref() != Some("courier")
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
        let veteran = |u: &Unit, role: bool| Veteran {
            r#type: self.rules.units[u.kind].id.clone(),
            experience: u.experience.unwrap_or(0),
            name: u.name.clone(),
            role: if role { u.role.clone() } else { None },
        };
        let veterans = eligible
            .into_iter()
            .take(4)
            .map(|u| veteran(u, false))
            .collect();
        let story = self
            .units
            .iter()
            .filter(|u| {
                u.side == Side::Player
                    && (u.role.as_deref() == Some("leader") || self.returns_with_story(u))
            })
            .map(|u| veteran(u, true))
            .collect();
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
                            && u.role.as_deref() != Some("leader")
                            && !self
                                .units
                                .iter()
                                .any(|alive| alive.side == Side::Player && alive.name == u.name)
                    })
                    .map(|u| u.name.clone().unwrap()),
            )
        {
            if !fallen.contains(&name) {
                fallen.push(name);
            }
        }
        Self::start_campaign(
            self.rules.clone(),
            &state.id,
            state.mission + 1,
            veterans,
            self.gold[0],
            choices,
            story,
            fallen,
        )
    }
}
