//! Chapter initialization, generic survival and veteran carryover.
use crate::CampaignProgram;
use crate::logic::conditions::LiveContext;
use crate::sort::SharedSort;
use crate::{data::*, logic::battle::LiveBattle};
impl<'catalog> LiveBattle<'catalog> {
    pub(crate) fn evaluate_campaign(&mut self, at_turn_start: bool, _entered: Option<usize>) {
        if self.story.campaign.is_none() || self.position.runtime.winner.is_some() {
            return;
        }
        self.apply_faction_transfers();
        self.evaluate_scripted_events(at_turn_start);
    }
    pub(crate) fn start_campaign(
        rules: &'catalog Catalogue,
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
            &CampaignProgram::compile(rules, rules.campaign(id), None),
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
        rules: &'catalog Catalogue,
        definition: &CampaignProgram,
        index: usize,
        veterans: Vec<Veteran>,
        carry: i32,
        flags: FlagHistory,
        story: Vec<Veteran>,
        fallen: Vec<String>,
    ) -> Self {
        assert!(
            (0..=definition.carryover().gold_limit).contains(&carry),
            "carryover gold must satisfy authored bounds"
        );
        let id = definition.id();
        let original = &definition.chapters()[index];
        let mut game =
            LiveBattle::from_state_with_campaign(rules, original.initial_state(), Some(definition));
        game.story.campaign = Some(
            CampaignState {
                scripts: None,
                evacuated: vec![],
                id: id.into(),
                mission: index,
                recall: veterans.iter().map(|v| v.r#type.clone()).collect(),
                veterans: Some(veterans.clone()),
                story_veterans: Some(story.clone()),
                fallen_companions: Some(fallen.clone()),
                carry_gold: carry,
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
        game.story.scripts = Some(
            super::event_state::EventProgress::restore(
                game.story.mission.as_ref().unwrap().clone(),
                EventState::new(&original.scripted_events),
            )
            .into(),
        );
        let names = game.position.runtime.names.clone();
        game.position.runtime.units.retain(|u| {
            names.get(u.faction) != "player"
                || u.role == Some(Role::Leader)
                || names
                    .resolve(u.character)
                    .is_none_or(|n| !fallen.iter().any(|f| f == n))
        });
        game.roster_changed();

        game.gold = original.gold.clone();
        game.gold["player"] += carry;
        for veteran in story {
            if let Some(arrival) = original
                .arrivals
                .iter()
                .find(|arrival| Some(&arrival.character) == veteran.character.as_ref())
            {
                let tile = game.site(&arrival.tile);
                let kind = game.kind(&veteran.r#type);
                let destination = (0..game.terrain.len())
                    .filter(|&t| {
                        game.at(t).is_none()
                            && !game
                                .arrival_tiles(
                                    kind,
                                    game.runtime
                                        .names
                                        .find("player")
                                        .expect("known player faction"),
                                    &veteran.items,
                                    &[t],
                                )
                                .is_empty()
                    })
                    .min_by_key(|&t| game.board.distance(tile, t));
                if let Some(destination) = destination {
                    let id = game.add(kind, "player", destination);
                    game.set_inventory(id, veteran.items.clone());
                    let character = veteran
                        .character
                        .as_deref()
                        .map(|name| game.intern_name(name));
                    let unit = game
                        .position
                        .runtime
                        .units
                        .iter_mut()
                        .find(|u| u.id == id)
                        .unwrap();
                    unit.character = character;
                    unit.experience = Some(veteran.experience);
                    unit.role = veteran.role;
                }
                continue;
            }
            let kind = game.kind(&veteran.r#type);
            if let Some(index) = game.position.runtime.units.iter().position(|u| {
                game.unit_faction(u) == "player"
                    && if veteran.role == Some(Role::Leader) {
                        u.role == Some(Role::Leader)
                    } else {
                        veteran.character.is_some()
                            && game.name(u.character) == veteran.character.as_deref()
                    }
            }) {
                if game
                    .definitions
                    .unit(game.position.runtime.units[index].kind)
                    .family()
                    == game.definitions.unit(kind).family()
                {
                    let id = game.position.runtime.units[index].id;
                    game.set_inventory(id, veteran.items.clone());
                    let mut projected = game.position.runtime.units[index];
                    projected.kind = kind;
                    let maximum_hp = game.definitions.unit(projected.kind).max_hp;
                    let unit = &mut game.position.runtime.units[index];
                    unit.kind = kind;
                    unit.experience = Some(veteran.experience);
                    unit.hp = maximum_hp;
                }
            }
        }
        let leader = game
            .units
            .iter()
            .find(|u| game.unit_faction(u) == "player" && u.role == Some(Role::Leader))
            .unwrap()
            .tile;
        game.roster_changed();
        let mut slots: Vec<_> = (0..game.terrain.len())
            .filter(|&t| {
                game.at(t).is_none()
                    && game.board.distance(t, leader) <= definition.carryover().deployment_radius
            })
            .collect();
        slots.order_by_key(|&t| game.board.distance(t, leader));
        for veteran in veterans {
            let kind = game.kind(&veteran.r#type);
            let Some(slot) = slots.iter().position(|&t| {
                !game
                    .arrival_tiles(
                        kind,
                        game.runtime
                            .names
                            .find("player")
                            .expect("known player faction"),
                        &veteran.items,
                        &[t],
                    )
                    .is_empty()
            }) else {
                // A changed map may not have a passable recall tile for this family.
                let name = game.definitions.unit(kind).name().to_owned();
                game.record(format!("No safe recall space for {name}."));
                continue;
            };
            let tile = slots.remove(slot);
            let id = game.add(kind, "player", tile);
            game.set_inventory(id, veteran.items);
            let character = veteran
                .character
                .as_deref()
                .map(|name| game.intern_name(name));
            let unit = game.position.runtime.units.last_mut().unwrap();
            unit.experience = Some(veteran.experience);
            unit.character = character;
        }
        for unit in game.position.runtime.units.clone() {
            if game.terrain_definition(unit.tile).capturable() {
                game.position.runtime.owners[unit.tile] = Some(game.unit_faction(&unit).to_owned());
            }
        }
        game.log.clear();

        game.story.event_opening = true;
        game.evaluate_campaign(true, None);
        let mut intro = vec![];
        LiveContext::new(&game, None, None)
            .script_speech(&game.story.mission.as_ref().unwrap().intro, &mut intro);
        intro.extend(std::mem::take(
            &mut game.story.campaign.as_mut().unwrap().dialogue,
        ));
        game.story.campaign.as_mut().unwrap().dialogue = intro;
        if let Some(story) = LiveContext::new(&game, None, None).resolved_story() {
            game.record(story.briefing);
        }
        game.sync_recruitment();
        game.refresh_planning_facts();
        game.remember_conditions();
        game
    }
    pub(crate) fn returns_with_story(&self, u: &Unit) -> bool {
        let state = self.story.campaign.as_ref().unwrap();
        self.campaign_definition()
            .chapters()
            .get(state.mission + 1)
            .is_some_and(|mission| {
                mission
                    .arrivals
                    .iter()
                    .any(|arrival| self.name(u.character) == Some(arrival.character.as_str()))
                    || mission.map.units.iter().any(|incoming| {
                        incoming.faction == "player"
                            && self.definitions.unit(self.kind(&incoming.r#type)).family()
                                == self.definitions.unit(u.kind).family()
                            && if u.role == Some(Role::Leader) {
                                incoming.role == Some(Role::Leader)
                            } else {
                                u.character.is_some()
                                    && incoming.character.as_deref() == self.name(u.character)
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
        let state = self.story.campaign.as_ref().unwrap();
        let mission = self.story.mission.as_ref().unwrap();
        let mut eligible: Vec<_> = self
            .units
            .iter()
            .filter(|u| {
                self.unit_faction(u) == "player"
                    && !u.is_leader()
                    && !u.role.is_some_and(Role::is_escort)
                    && self.name(u.character).is_none_or(|n| {
                        !mission
                            .required_companions
                            .iter()
                            .any(|required| required == n)
                    })
                    && !self.returns_with_story(u)
            })
            .collect();
        let value = |unit: &Unit| {
            let definition = self.definitions.unit(unit.kind);
            let weights = &self.campaign_definition().carryover().weights;
            let owned = self.definitions.owned(unit, &self.runtime.inventories);
            let capabilities = &owned.assessment;
            {
                f64::from(definition.max_hp) * weights.hp
                    + capabilities.meta_attack_score() * weights.damage
                    + capabilities.meta_healing_score() * weights.healing
                    + f64::from(definition.level()) * weights.level
                    + f64::from(unit.experience.expect("resolved unit experience"))
                        * weights.experience
            }
        };
        eligible.order_by(|a, b| value(b).total_cmp(&value(a)));
        if let Some(selected) = selected {
            let mut recalled = Vec::new();
            if selected.len() > self.campaign_definition().carryover().veteran_limit {
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
            items: self
                .inventories
                .items(u.inventory)
                .map(str::to_owned)
                .collect(),
            r#type: self.definitions.unit(u.kind).id().to_owned(),
            experience: u.experience.expect("resolved unit experience"),
            character: self.name(u.character).map(str::to_owned),
            role: if role { u.role.clone() } else { None },
        };
        eligible.truncate(self.campaign_definition().carryover().veteran_limit);
        let overflow: Vec<_> = self
            .units
            .iter()
            .filter(|unit| {
                self.unit_faction(unit) == "player"
                    && !self.returns_with_story(unit)
                    && !eligible.iter().any(|selected| selected.id == unit.id)
            })
            .flat_map(|unit| self.inventories.items(unit.inventory).map(str::to_owned))
            .collect();
        let veterans = eligible.into_iter().map(|u| veteran(u, false)).collect();
        let mut story: Vec<Veteran> = self
            .units
            .iter()
            .filter(|u| self.unit_faction(u) == "player" && self.returns_with_story(u))
            .map(|u| veteran(u, true))
            .collect();
        if let Some(next) = self.campaign_definition().chapters().get(state.mission + 1) {
            for record in &state.evacuated {
                let unit = &record.unit;
                if unit.faction == "player"
                    && next.map.units.iter().any(|incoming| {
                        incoming.faction == "player"
                            && self.definitions.unit(self.kind(&incoming.r#type)).family()
                                == self.definitions.unit(self.kind(&unit.r#type)).family()
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
                        experience: unit.experience.expect("resolved unit experience"),
                        character: unit.character.clone(),
                        role: unit.role,
                    });
                }
            }
        }
        let flags = state.flags.clone();
        let mut fallen = state
            .fallen_companions
            .clone()
            .expect("resolved campaign fallen_companions");
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
                        u.faction == "player"
                            && u.character.is_some()
                            && u.role != Some(Role::Leader)
                            && !self.position.runtime.units.iter().any(|alive| {
                                self.unit_faction(alive) == "player"
                                    && self.name(alive.character) == u.character.as_deref()
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
            self.catalogue,
            self.campaign_definition(),
            state.mission + 1,
            veterans,
            self.position.runtime.gold["player"]
                .min(self.campaign_definition().carryover().gold_limit),
            flags,
            story,
            fallen,
        );
        if !overflow.is_empty() {
            let leader_id = next
                .units
                .iter()
                .find(|unit| self.unit_faction(unit) == "player" && unit.is_leader())
                .unwrap()
                .id;
            for item in overflow {
                next.give_item(leader_id, item);
            }
            let unit = next.units.iter().find(|unit| unit.id == leader_id).unwrap();
            let leader = Veteran {
                items: next
                    .inventories
                    .items(unit.inventory)
                    .map(str::to_owned)
                    .collect(),
                r#type: next.definitions.unit(unit.kind).id().to_owned(),
                experience: unit.experience.expect("resolved unit experience"),
                character: next.name(unit.character).map(str::to_owned),
                role: unit.role,
            };
            let story = next
                .story
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
        next.combat_rng = self.position.runtime.combat_rng;
        Ok(next)
    }
}
