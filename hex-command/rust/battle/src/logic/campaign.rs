//! Chapter initialization, generic survival and veteran carryover.
use crate::sort::SharedSort;
use crate::{data::*, logic::battle::Game};
impl<'catalog> Game<'catalog> {
    pub(crate) fn evaluate_campaign(&mut self, at_turn_start: bool, _entered: Option<usize>) {
        if self.campaign.is_none() || self.winner.is_some() {
            return;
        }
        self.apply_faction_transfers();
        self.evaluate_scripted_events(at_turn_start);
        if self.winner.is_some() {
            return;
        }
        let missing = self
            .mission
            .as_ref()
            .unwrap()
            .required_companions
            .iter()
            .find(|id| {
                !self
                    .units
                    .iter()
                    .any(|u| u.character.as_ref() == Some(id) && u.side == Side::Player)
            });
        if missing.is_some()
            || !self
                .units
                .iter()
                .any(|u| u.is_leader() && u.side == Side::Player)
        {
            self.winner = Some(Side::Enemy);
            let message = "The company cannot continue without its protected companions.";
            self.campaign.as_mut().unwrap().outcome = Some(message.into());
            self.campaign.as_mut().unwrap().say([Dialogue {
                character: None,
                speaker: "The company".into(),
                portrait: "worker".into(),
                text: message.into(),
                sound: None,
            }]);
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
        Self::start_campaign_from_definition(
            rules,
            &rules.campaigns[id],
            index,
            veterans,
            carry,
            choices
                .into_iter()
                .map(|id| {
                    (
                        id,
                        vec![FlagChange {
                            state: true,
                            stage: index.saturating_sub(1),
                            turn: 0,
                        }],
                    )
                })
                .collect(),
            story,
            fallen,
        )
    }
    pub(crate) fn start_campaign_from_definition(
        rules: &'catalog Rules,
        definition: &'catalog Campaign,
        index: usize,
        veterans: Vec<Veteran>,
        carry: i32,
        flags: FlagHistory,
        story: Vec<Veteran>,
        fallen: Vec<String>,
    ) -> Self {
        let id = definition.id.as_str();
        let original = &definition.missions[index];
        let mut game = Game::new_with_campaign(rules, &original.map.id, Some(definition));
        game.campaign = Some(
            CampaignState {
                scripts: None,
                evacuated: vec![],
                id: id.into(),
                mission: index,
                recall: veterans.iter().map(|v| v.r#type.clone()).collect(),
                veterans: Some(veterans.clone()),
                story_veterans: Some(story.clone()),
                fallen_companions: Some(fallen.clone()),
                carry_gold: carry.clamp(0, 8),
                marks: vec![],
                dialogue: vec![],
                dialogue_index: 0,
                outcome: None,
                completed: false,
                refuge: None,
                vigil: None,
                holds: None,
                flags,
            }
            .into(),
        );
        game.refresh_mission();
        game.scripts = Some(
            super::event_state::EventProgress::restore(
                game.mission.as_ref().unwrap().clone(),
                EventState::new(&original.scripted_events),
            )
            .into(),
        );
        game.units.retain(|u| {
            u.side != Side::Player
                || u.role == Some(Role::Leader)
                || u.character.as_ref().is_none_or(|n| !fallen.contains(n))
        });
        game.roster_changed();

        game.gold = original.gold.clone();
        game.gold["player"] += carry.clamp(0, 8);
        for veteran in story {
            if let Some(arrival) = original
                .arrivals
                .iter()
                .find(|arrival| Some(&arrival.character) == veteran.character.as_ref())
            {
                let tile = game.site(&arrival.tile);
                let kind = game.kind(&veteran.r#type);
                let destination = (0..game.terrain.len())
                    .filter(|&t| game.at(t).is_none() && game.cost(kind, t) != u16::MAX)
                    .min_by_key(|&t| game.board.distance(tile, t));
                if let Some(destination) = destination {
                    let id = game.add(kind, Side::Player, destination);
                    let unit = game.units.iter_mut().find(|u| u.id == id).unwrap();
                    unit.items = veteran.items.clone();
                    unit.character = veteran.character.clone();
                    unit.experience = Some(veteran.experience);
                    unit.role = veteran.role;
                }
                continue;
            }
            let kind = game.kind(&veteran.r#type);
            if let Some(unit) = game.units.iter_mut().find(|u| {
                u.side == Side::Player
                    && if veteran.role == Some(Role::Leader) {
                        u.role == Some(Role::Leader)
                    } else {
                        veteran.character.is_some() && u.character == veteran.character
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
        slots.order_by_key(|&t| game.board.distance(t, leader));
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
            unit.character = veteran.character;
            unit.items = veteran.items;
        }
        for unit in &game.units {
            if game.terrain_role(unit.tile).capturable() {
                game.owners[unit.tile] = Some(game.unit_faction(&unit).to_owned());
            }
        }
        game.log.clear();

        game.event_opening = true;
        game.evaluate_campaign(true, None);
        let mut intro = vec![];
        game.script_speech(&game.mission.as_ref().unwrap().intro, &mut intro);
        intro.extend(std::mem::take(
            &mut game.campaign.as_mut().unwrap().dialogue,
        ));
        game.campaign.as_mut().unwrap().dialogue = intro;
        if let Some(story) = game.resolved_story() {
            game.record(story.briefing);
        }
        game.remember_conditions();
        game
    }
    pub(crate) fn returns_with_story(&self, u: &Unit) -> bool {
        let state = self.campaign.as_ref().unwrap();
        self.campaign_definition()
            .missions
            .get(state.mission + 1)
            .is_some_and(|mission| {
                mission
                    .arrivals
                    .iter()
                    .any(|arrival| u.character.as_ref() == Some(&arrival.character))
                    || mission.map.units.iter().any(|incoming| {
                        incoming.side == Side::Player
                            && self.rules.units[self.kind(&incoming.r#type)].family
                                == self.rules.units[u.kind].family
                            && if u.role == Some(Role::Leader) {
                                incoming.role == Some(Role::Leader)
                            } else {
                                u.character.is_some() && incoming.character == u.character
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
                    && !u.role.is_some_and(Role::is_escort)
                    && u.character
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
        eligible.order_by(|a, b| value(b).total_cmp(&value(a)));
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
            character: u.character.clone(),
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
        if let Some(next) = self.campaign_definition().missions.get(state.mission + 1) {
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
                                unit.character.is_some() && incoming.character == unit.character
                            }
                    })
                {
                    story.push(Veteran {
                        r#type: unit.r#type.clone(),
                        items: unit.items.clone(),
                        experience: unit.experience.unwrap_or(0),
                        character: unit.character.clone(),
                        role: unit.role,
                    });
                }
            }
        }
        let flags = state.flags.clone();
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
                            && u.character.is_some()
                            && u.role != Some(Role::Leader)
                            && !self.units.iter().any(|alive| {
                                alive.side == Side::Player && alive.character == u.character
                            })
                            && !state
                                .evacuated
                                .iter()
                                .any(|record| record.unit.character == u.character)
                    })
                    .map(|u| u.character.clone().unwrap()),
            )
        {
            if !fallen.contains(&name) {
                fallen.push(name);
            }
        }
        let mut next = Self::start_campaign_from_definition(
            self.rules,
            self.campaign_definition(),
            state.mission + 1,
            veterans,
            self.gold["player"],
            flags,
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
