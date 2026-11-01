//! Atomic validation and execution of typed battle commands.

use super::{
    action::{Action, Operation},
    battle::Game,
};
use crate::data::*;
impl Game<'_> {
    /// Validate all identifiers and legality before making any state mutation.
    pub(crate) fn execute(&mut self, command: impl Into<Operation>) -> Result<(), CommandError> {
        let command = match command.into() {
            Operation::Planned(action) => return self.execute_action(action),
            Operation::Wire(command) => command,
        };
        if self.winner.is_some()
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
        match command {
            Command::Move { unit, tile } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let tile = *self
                    .board
                    .indices
                    .get(&tile.0)
                    .ok_or(CommandError::UnknownTile(tile))?;
                self.execute_action(Action::Move { unit, tile })?;
            }
            Command::Attack {
                unit,
                target,
                weapon,
            } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                self.unit(target.0)
                    .ok_or(CommandError::UnknownUnit(target))?;
                let weapon = self.rules.units[actor.kind]
                    .attacks
                    .iter()
                    .position(|a| a.id == weapon.0)
                    .ok_or(CommandError::UnknownWeapon(weapon))?;
                self.execute_action(Action::Attack {
                    unit,
                    target,
                    weapon,
                })?;
            }
            Command::Recruit { unit, kind, tile } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let kind = *self
                    .rules
                    .types
                    .get(&kind.0)
                    .ok_or(CommandError::UnknownType(kind))?;
                let tile = *self
                    .board
                    .indices
                    .get(&tile.0)
                    .ok_or(CommandError::UnknownTile(tile))?;
                self.execute_action(Action::Recruit { unit, kind, tile })?;
            }
            Command::Advance { unit, kind } => {
                self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let kind = *self
                    .rules
                    .types
                    .get(&kind.0)
                    .ok_or(CommandError::UnknownType(kind))?;
                self.execute_action(Action::Advance { unit, kind })?;
            }
            Command::EndTurn => self.end_turn(),
            Command::DismissDialogue => {
                let campaign = self.campaign.as_mut().ok_or(CommandError::NoCampaign)?;
                if campaign.dialogue_index < campaign.dialogue.len() {
                    campaign.dialogue_index += 1;
                }
            }
            Command::PreviousDialogue => {
                let campaign = self.campaign.as_mut().ok_or(CommandError::NoCampaign)?;
                if campaign.dialogue_index < campaign.dialogue.len() {
                    campaign.dialogue_index = campaign.dialogue_index.saturating_sub(1);
                }
            }
            Command::SkipDialogue => {
                let campaign = self.campaign.as_mut().ok_or(CommandError::NoCampaign)?;
                campaign.dialogue_index = campaign.dialogue.len();
            }
            Command::RetryMission => {
                let campaign = self.campaign.as_ref().ok_or(CommandError::NoCampaign)?;
                let combat_rng = self.combat_rng;
                #[cfg(not(target_arch = "wasm32"))]
                let records = self.recordings.take();
                let choices = if self.mission.as_ref().is_some_and(|m| m.refuge.is_some()) {
                    vec![]
                } else {
                    campaign.choices.clone().unwrap_or_default()
                };
                *self = Self::start_campaign_source(
                    self.rules.clone(),
                    &campaign.id,
                    campaign.mission,
                    campaign.veterans.clone().unwrap_or_default(),
                    campaign.carry_gold,
                    choices,
                    campaign.story_veterans.clone().unwrap_or_default(),
                    campaign.fallen_companions.clone().unwrap_or_default(),
                );
                self.combat_rng = combat_rng;
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.recordings = records;
                }
            }
            Command::RecallMission { units } => {
                let campaign = self.campaign.as_ref().ok_or(CommandError::NoCampaign)?;
                if self.winner != Some(Side::Player)
                    || campaign.completed
                    || campaign.mission + 1 >= self.rules.campaigns[&campaign.id].missions.len()
                {
                    return Err(CommandError::NoNextMission);
                }
                let replacement = self.continue_campaign(Some(&units))?;
                #[cfg(not(target_arch = "wasm32"))]
                let records = self.recordings.take();
                *self = replacement;
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.recordings = records;
                }
            }
            Command::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attacked,
                record_retreat,
            } => {
                self.execute_action(Action::SetUnitIntent {
                    unit,
                    retreating,
                    rebuilding,
                    attacked,
                    record_retreat,
                })?;
            }
        }
        Ok(())
    }
    fn execute_action(&mut self, action: Action) -> Result<(), CommandError> {
        if self.winner.is_some() && !matches!(action, Action::SetUnitIntent { .. }) {
            return Err(CommandError::BattleFinished);
        }
        match action {
            Action::Move { unit, tile } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if actor.tile == tile || !self.reachable(actor).iter().any(|&(t, _)| t == tile) {
                    return Err(CommandError::IllegalMove);
                }
                self.move_unit(unit.0, tile);
            }
            Action::Attack {
                unit,
                target,
                weapon,
            } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let victim = self
                    .unit(target.0)
                    .ok_or(CommandError::UnknownUnit(target))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if actor.attacked || !self.can_attack(actor, actor.tile, victim, weapon) {
                    return Err(CommandError::IllegalAttack);
                }
                self.attack(unit.0, target.0, weapon);
            }
            Action::Recruit { unit, kind, tile } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if !self.placement_tiles(actor, kind, true).contains(&tile) {
                    return Err(CommandError::IllegalRecruit);
                }
                self.recruit(unit.0, kind, tile);
            }
            Action::Advance { unit, kind } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if !self.promotion_options(actor).contains(&kind) {
                    return Err(CommandError::IllegalAdvance);
                }
                self.advance(unit.0, kind);
            }
            Action::EndTurn => self.end_turn(),
            Action::SetUnitIntent {
                unit,
                retreating,
                rebuilding,
                attacked,
                record_retreat,
            } => {
                let index = self.index(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                if self.units[index].side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if self.units[index].attacked && !attacked {
                    return Err(CommandError::IllegalIntent);
                }
                if record_retreat
                    && (retreating != Some(true) || !attacked || !self.units[index].moved)
                {
                    return Err(CommandError::IllegalIntent);
                }
                let name = (self.presentation && record_retreat).then(|| {
                    self.units[index].name.clone().unwrap_or_else(|| {
                        self.rules.units[self.units[index].kind]
                            .name
                            .as_ref()
                            .into()
                    })
                });
                self.units[index].retreating = retreating;
                self.units[index].rebuilding = rebuilding;
                self.units[index].attacked = attacked;
                if let Some(name) = name {
                    game_log!(self, format!("{name} is retreating."));
                }
            }
        }
        Ok(())
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
        game.winner = Some(Side::Player);
        let leader = game
            .units
            .iter()
            .find(|u| rules.units[u.kind].leader && u.side == Side::Player)
            .unwrap();
        let kind = rules.types["archer"];
        let site = game.board.adjacent[leader.tile]
            .iter()
            .copied()
            .find(|&t| game.at(t).is_none())
            .unwrap();
        let id = game.add(kind, Side::Player, site);
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
        let campaign = game.campaign.as_ref().unwrap();
        assert_eq!(campaign.mission, 1);
        assert_eq!(campaign.veterans.as_ref().unwrap().len(), 1);
        assert_eq!(campaign.veterans.as_ref().unwrap()[0].r#type, "archer");
    }
}
