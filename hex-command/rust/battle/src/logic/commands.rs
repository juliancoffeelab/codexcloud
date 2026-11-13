//! Atomic validation and execution of typed battle commands.

use super::battle::Game;
use crate::{RuntimeCommand, data::*};
impl Game<'_> {
    /// Resolve the wire representation once; runtime AI bypasses this boundary.
    pub(crate) fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        if self.runtime.winner.is_some()
            && !matches!(
                command,
                Command::DismissDialogue
                    | Command::PreviousDialogue
                    | Command::SkipDialogue
                    | Command::RecallMission { .. }
                    | Command::RetryMission
                    | Command::SetUnitIntent { .. }
            )
        {
            return Err(CommandError::BattleFinished);
        }
        let command = self.resolve_command(command)?;
        self.execute_runtime(command)
    }
}

#[cfg(test)]
mod browser_controls_tests {
    use super::*;

    #[test]
    fn explicit_recall_preserves_selection_and_rejects_duplicates_atomically() {
        let rules = Rules::load().unwrap();
        let mut game =
            Game::start_campaign(&rules, "lantern-road", 0, vec![], 0, vec![], vec![], vec![]);
        game.winner = Some(BattleResult::PlayerVictory);
        let leader = game
            .units
            .iter()
            .find(|u| u.role == Some(Role::Leader) && u.phase == TurnPhase::Player)
            .unwrap();
        let kind = rules.types["archer"];
        let site = game.board.adjacent[leader.tile]
            .iter()
            .copied()
            .find(|&t| game.at(t).is_none())
            .unwrap();
        let id = game.add(kind, "player", site);
        let before = serde_json::to_value(game.state()).unwrap();
        assert!(
            game.execute(Command::RecallMission {
                units: vec![UnitId(id), UnitId(id)]
            })
            .is_err()
        );
        assert_eq!(serde_json::to_value(game.state()).unwrap(), before);
        game.execute(Command::RecallMission {
            units: vec![UnitId(id)],
        })
        .unwrap();
        let campaign = game.story.campaign.as_ref().unwrap();
        assert_eq!(campaign.mission, 1);
        assert_eq!(campaign.veterans.as_ref().unwrap().len(), 1);
        assert_eq!(campaign.veterans.as_ref().unwrap()[0].r#type, "archer");
    }
}

impl super::BattlePosition<'_> {
    pub(super) fn resolve_command(&self, command: Command) -> Result<RuntimeCommand, CommandError> {
        Ok(match command {
            Command::Move { unit, tile } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let tile = TileIndex(
                    *self
                        .board
                        .indices
                        .get(&tile.0)
                        .ok_or_else(|| CommandError::UnknownTile(tile.clone()))?,
                );
                RuntimeCommand::Move { unit, tile }
            }
            Command::Attack {
                unit,
                target,
                weapon,
            } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                self.unit(target.0)
                    .ok_or(CommandError::UnknownUnit(target))?;
                let actor = self.unit(unit.0).unwrap();
                let weapon = self
                    .weapons(actor)
                    .find(|w| self.source_weapon(w).id == weapon.0)
                    .ok_or_else(|| CommandError::UnknownWeapon(weapon.clone()))?
                    .key();
                RuntimeCommand::Attack {
                    unit,
                    target,
                    weapon,
                }
            }
            Command::Recruit { unit, kind, tile } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let kind = UnitKind(
                    *self
                        .rules
                        .types
                        .get(&kind.0)
                        .ok_or_else(|| CommandError::UnknownType(kind.clone()))?,
                );
                let tile = TileIndex(
                    *self
                        .board
                        .indices
                        .get(&tile.0)
                        .ok_or_else(|| CommandError::UnknownTile(tile.clone()))?,
                );
                RuntimeCommand::Recruit { unit, kind, tile }
            }
            Command::Advance { unit, kind } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let kind = UnitKind(
                    *self
                        .rules
                        .types
                        .get(&kind.0)
                        .ok_or_else(|| CommandError::UnknownType(kind.clone()))?,
                );
                RuntimeCommand::Advance { unit, kind }
            }
            Command::DropItem { unit, item } => RuntimeCommand::DropItem { unit, item },
            Command::RecallMission { units } => RuntimeCommand::RecallMission { units },
            Command::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attack_spent,
                record_retreat,
            } => RuntimeCommand::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attack_spent,
                record_retreat,
            },
            Command::EndTurn => RuntimeCommand::EndTurn,
            Command::DismissDialogue => RuntimeCommand::DismissDialogue,
            Command::RetryMission => RuntimeCommand::RetryMission,
            Command::PreviousDialogue => RuntimeCommand::PreviousDialogue,
            Command::SkipDialogue => RuntimeCommand::SkipDialogue,
        })
    }
    pub(crate) fn command_snapshot(
        &self,
        command: &RuntimeCommand,
    ) -> Result<Command, CommandError> {
        self.check_command_indices(command)?;
        Ok(match command {
            RuntimeCommand::Move { unit, tile } => Command::Move {
                unit: unit.clone(),
                tile: TileId(self.runtime.board.ids[tile.0].clone()),
            },
            RuntimeCommand::Attack {
                unit,
                target,
                weapon,
            } => Command::Attack {
                unit: unit.clone(),
                target: target.clone(),
                weapon: WeaponId(self.weapon_id(*weapon).to_owned()),
            },
            RuntimeCommand::Recruit { unit, kind, tile } => Command::Recruit {
                unit: unit.clone(),
                kind: UnitTypeId(self.rules.units[kind.0].id.clone()),
                tile: TileId(self.runtime.board.ids[tile.0].clone()),
            },
            RuntimeCommand::Advance { unit, kind } => Command::Advance {
                unit: unit.clone(),
                kind: UnitTypeId(self.rules.units[kind.0].id.clone()),
            },
            RuntimeCommand::DropItem { unit, item } => Command::DropItem {
                unit: unit.clone(),
                item: item.clone(),
            },
            RuntimeCommand::RecallMission { units } => Command::RecallMission {
                units: units.clone(),
            },
            RuntimeCommand::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attack_spent,
                record_retreat,
            } => Command::SetUnitIntent {
                unit: unit.clone(),
                retreating: retreating.clone(),
                rebuilding: rebuilding.clone(),
                attack_spent: attack_spent.clone(),
                record_retreat: record_retreat.clone(),
            },
            RuntimeCommand::EndTurn => Command::EndTurn,
            RuntimeCommand::DismissDialogue => Command::DismissDialogue,
            RuntimeCommand::RetryMission => Command::RetryMission,
            RuntimeCommand::PreviousDialogue => Command::PreviousDialogue,
            RuntimeCommand::SkipDialogue => Command::SkipDialogue,
        })
    }
    pub(crate) fn check_command_indices(
        &self,
        command: &RuntimeCommand,
    ) -> Result<(), CommandError> {
        match command {
            RuntimeCommand::Move { tile, .. } if tile.0 >= self.runtime.terrain.len() => {
                Err(CommandError::IllegalMove)
            }
            RuntimeCommand::Recruit { kind, tile, .. }
                if kind.0 >= self.rules.units.len() || tile.0 >= self.runtime.terrain.len() =>
            {
                Err(CommandError::IllegalRecruit)
            }
            RuntimeCommand::Advance { kind, .. } if kind.0 >= self.rules.units.len() => {
                Err(CommandError::IllegalAdvance)
            }
            _ => Ok(()),
        }
    }
    pub(crate) fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        let command = self.resolve_command(command)?;
        self.execute_runtime(command)
    }
    pub(crate) fn execute_runtime(&mut self, command: RuntimeCommand) -> Result<(), CommandError> {
        if self.winner.is_some() && !matches!(command, RuntimeCommand::SetUnitIntent { .. }) {
            return Err(CommandError::BattleFinished);
        }
        self.check_command_indices(&command)?;
        self.runtime.traversal = None;
        self.runtime.mechanical_deaths.clear();
        match command {
            RuntimeCommand::Move { unit, tile } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let site = tile.0;
                if !self.may_act(actor) {
                    return Err(CommandError::WrongTurn);
                }
                if actor.tile == site || !self.reachable(actor).iter().any(|&(t, _)| t == site) {
                    return Err(CommandError::IllegalMove);
                }
                self.move_unit(unit.0, site);
            }

            RuntimeCommand::Attack {
                unit,
                target,
                weapon,
            } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let victim = self
                    .unit(target.0)
                    .ok_or(CommandError::UnknownUnit(target))?;
                let attack = self
                    .weapons(actor)
                    .position(|a| a.matches_key(&weapon))
                    .ok_or_else(|| {
                        CommandError::UnknownWeapon(WeaponId(self.weapon_id(weapon).to_owned()))
                    })?;
                if !self.may_act(actor) {
                    return Err(CommandError::WrongTurn);
                }
                if actor.attack_spent || !self.can_attack(actor, actor.tile, victim, attack) {
                    return Err(CommandError::IllegalAttack);
                }
                self.attack(unit.0, target.0, attack);
            }

            RuntimeCommand::Recruit { unit, kind, tile } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let recruit = kind.0;
                let site = tile.0;
                if !self.may_act(actor) {
                    return Err(CommandError::WrongTurn);
                }
                if !self.placement_tiles(actor, recruit, true).contains(&site) {
                    return Err(CommandError::IllegalRecruit);
                }
                self.recruit(unit.0, recruit, site);
            }

            RuntimeCommand::Advance { unit, kind } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let advance = kind.0;
                if !self.may_act(actor) {
                    return Err(CommandError::WrongTurn);
                }
                if !self.promotion_options(actor).contains(&advance) {
                    return Err(CommandError::IllegalAdvance);
                }
                self.advance(unit.0, advance);
            }

            RuntimeCommand::DropItem { unit, item } => {
                let index = self.index(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                if !self.may_act(&self.runtime.units[index]) {
                    return Err(CommandError::WrongTurn);
                }
                let slot = self
                    .inventory(&self.runtime.units[index])
                    .position(|candidate| candidate.id == item)
                    .ok_or(CommandError::IllegalItem)?;
                let item = self.drop_item(unit.0, slot);
                self.record(format!(
                    "{} dropped {}.",
                    self.unit_name(&self.runtime.units[index]),
                    self.runtime.inventories.item(item).name
                ));
                self.runtime.ground_items.push(GroundItemState {
                    tile: self.runtime.units[index].tile,
                    item,
                });
                #[cfg(not(target_arch = "wasm32"))]
                self.action(ActionRecord::Drop {
                    unit: format!("u{}", unit.0),
                    item: self.runtime.inventories.item(item).id.clone(),
                });
            }

            RuntimeCommand::EndTurn => self.end_turn(),

            RuntimeCommand::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attack_spent,
                record_retreat,
            } => {
                let index = self.index(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                if !self.may_act(&self.runtime.units[index]) {
                    return Err(CommandError::WrongTurn);
                }
                if self.runtime.units[index].attack_spent && !attack_spent {
                    return Err(CommandError::IllegalIntent);
                }
                if record_retreat
                    && (retreating != Some(true)
                        || !attack_spent
                        || !self.runtime.units[index].moved)
                {
                    return Err(CommandError::IllegalIntent);
                }
                let name =
                    record_retreat.then(|| self.unit_name(&self.runtime.units[index]).to_owned());
                self.runtime.units[index].retreating = retreating;
                self.runtime.units[index].rebuilding = rebuilding;
                self.runtime.units[index].attack_spent = attack_spent;
                if let Some(name) = name {
                    self.record(format!("{name} is retreating."));
                }
            }
            RuntimeCommand::DismissDialogue
            | RuntimeCommand::PreviousDialogue
            | RuntimeCommand::SkipDialogue
            | RuntimeCommand::RecallMission { .. }
            | RuntimeCommand::RetryMission => return Err(CommandError::NoCampaign),
        }
        self.remember_faction_sight();
        Ok(())
    }
}
impl Game<'_> {
    pub(crate) fn execute_runtime(&mut self, command: RuntimeCommand) -> Result<(), CommandError> {
        match command {
            RuntimeCommand::DismissDialogue => {
                let campaign = self
                    .story
                    .campaign
                    .as_mut()
                    .ok_or(CommandError::NoCampaign)?;
                if campaign.dialogue_index < campaign.dialogue.len() {
                    campaign.dialogue_index += 1;
                }
            }
            RuntimeCommand::PreviousDialogue => {
                let campaign = self
                    .story
                    .campaign
                    .as_mut()
                    .ok_or(CommandError::NoCampaign)?;
                if campaign.dialogue_index < campaign.dialogue.len() {
                    campaign.dialogue_index = campaign.dialogue_index.saturating_sub(1);
                }
            }
            RuntimeCommand::SkipDialogue => {
                let campaign = self
                    .story
                    .campaign
                    .as_mut()
                    .ok_or(CommandError::NoCampaign)?;
                campaign.dialogue_index = campaign.dialogue.len();
            }
            RuntimeCommand::RetryMission => {
                let campaign = self
                    .story
                    .campaign
                    .as_ref()
                    .ok_or(CommandError::NoCampaign)?
                    .clone();
                let combat_rng = self.runtime.combat_rng;
                #[cfg(not(target_arch = "wasm32"))]
                let records = self.runtime.recordings.take();
                let flags = campaign.retry_flags();
                *self = Self::start_campaign_from_definition(
                    self.rules,
                    self.campaign_definition(),
                    campaign.mission,
                    campaign.veterans.clone().unwrap_or_default(),
                    campaign.carry_gold,
                    flags,
                    campaign.story_veterans.clone().unwrap_or_default(),
                    campaign.fallen_companions.clone().unwrap_or_default(),
                );
                self.runtime.combat_rng = combat_rng;
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.runtime.recordings = records;
                }
            }
            RuntimeCommand::RecallMission { units } => {
                let campaign = self
                    .story
                    .campaign
                    .as_ref()
                    .ok_or(CommandError::NoCampaign)?
                    .clone();
                if self.runtime.winner != Some(BattleResult::PlayerVictory)
                    || campaign.completed
                    || campaign.mission + 1 >= self.campaign_definition().missions.len()
                {
                    return Err(CommandError::NoNextMission);
                }
                let replacement = self.continue_campaign(Some(&units))?;
                #[cfg(not(target_arch = "wasm32"))]
                let records = self.runtime.recordings.take();
                *self = replacement;
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.runtime.recordings = records;
                }
            }
            command => {
                let moving = match &command {
                    RuntimeCommand::Move { unit, .. } => self.unit(unit.0).copied(),
                    _ => None,
                };
                let combat = match &command {
                    RuntimeCommand::Attack { unit, target, .. } => self
                        .unit(unit.0)
                        .zip(self.unit(target.0))
                        .map(|(a, b)| (*a, *b, self.mediation_at_risk(b))),
                    _ => None,
                };
                let tick = matches!(command, RuntimeCommand::EndTurn);
                if let RuntimeCommand::Move { unit, tile } = command {
                    self.position
                        .check_command_indices(&RuntimeCommand::Move { unit, tile })?;
                    if self.winner.is_some() {
                        return Err(CommandError::BattleFinished);
                    }
                    let movement = self.position.validate_observed_move(unit.0, tile.0)?;
                    self.position.runtime.traversal = None;
                    self.position.runtime.mechanical_deaths.clear();
                    if self.position.execute_observed_move(movement) {
                        self.story.planning_revision = self.story.planning_revision.wrapping_add(1);
                    }
                    self.position.remember_faction_sight();
                } else {
                    self.position.execute_runtime(command)?;
                }
                self.story.event_departure = moving;
                if let Some((attacker, target, mediated)) = combat {
                    self.accept_combat_deaths(attacker, target, mediated);
                }
                self.apply_faction_transfers();
                self.refresh_perception_sources();
                self.evaluate_campaign(tick, None);
            }
        }
        self.refresh_planning_facts();
        self.remember_conditions();
        Ok(())
    }
}
