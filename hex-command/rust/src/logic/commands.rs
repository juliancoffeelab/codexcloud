//! Atomic validation and execution of typed battle commands.

use super::battle::Game;
use crate::data::*;
impl Game<'_> {
    /// Validate all identifiers and legality before making any state mutation.
    pub(crate) fn execute(&mut self, command: Command) -> Result<Effects, CommandError> {
        if self.winner.is_some()
            && !matches!(
                command,
                Command::DismissDialogue | Command::NextMission | Command::SetUnitIntent { .. }
            )
        {
            return Err(CommandError::BattleFinished);
        }
        let mut effects = Effects::default();
        match command {
            Command::Move { unit, tile } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let site = *self
                    .board
                    .indices
                    .get(&tile.0)
                    .ok_or_else(|| CommandError::UnknownTile(tile.clone()))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if actor.tile == site || !self.reachable(actor).iter().any(|&(t, _)| t == site) {
                    return Err(CommandError::IllegalMove);
                }
                self.move_unit(unit.0, site);
            }
            Command::Attack {
                unit,
                target,
                weapon,
            } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let victim = self
                    .unit(target.0)
                    .ok_or(CommandError::UnknownUnit(target))?;
                let attack = self.rules.units[actor.kind]
                    .attacks
                    .iter()
                    .position(|a| a.id == weapon.0)
                    .ok_or_else(|| CommandError::UnknownWeapon(weapon.clone()))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if actor.attacked || !self.can_attack(actor, actor.tile, victim, attack) {
                    return Err(CommandError::IllegalAttack);
                }
                let (damage, retaliation) = self.attack(unit.0, target.0, attack);
                effects.damage = Some(damage);
                effects.retaliation = Some(retaliation);
            }
            Command::Recruit { unit, kind, tile } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let recruit = *self
                    .rules
                    .types
                    .get(&kind.0)
                    .ok_or_else(|| CommandError::UnknownType(kind.clone()))?;
                let site = *self
                    .board
                    .indices
                    .get(&tile.0)
                    .ok_or_else(|| CommandError::UnknownTile(tile.clone()))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if !self.placement_tiles(actor, recruit, true).contains(&site) {
                    return Err(CommandError::IllegalRecruit);
                }
                effects.added_unit = Some(UnitId(self.recruit(unit.0, recruit, site)));
            }
            Command::Advance { unit, kind } => {
                let actor = self.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
                let advance = *self
                    .rules
                    .types
                    .get(&kind.0)
                    .ok_or_else(|| CommandError::UnknownType(kind.clone()))?;
                if actor.side != self.turn {
                    return Err(CommandError::WrongTurn);
                }
                if !self.promotion_options(actor).contains(&advance) {
                    return Err(CommandError::IllegalAdvance);
                }
                self.advance(unit.0, advance);
            }
            Command::EndTurn => self.end_turn(),
            Command::DismissDialogue => {
                let campaign = self.campaign.as_mut().ok_or(CommandError::NoCampaign)?;
                if campaign.dialogue_index < campaign.dialogue.len() {
                    campaign.dialogue_index += 1;
                }
            }
            Command::NextMission => {
                let campaign = self.campaign.as_ref().ok_or(CommandError::NoCampaign)?;
                if self.winner != Some(Side::Player)
                    || campaign.completed
                    || campaign.mission + 1 >= self.rules.campaigns[&campaign.id].missions.len()
                {
                    return Err(CommandError::NoNextMission);
                }
                let records = self.recordings.take();
                *self = self.next_campaign();
                self.recordings = records;
            }
            Command::SetUnitIntent {
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
                let name = record_retreat.then(|| {
                    self.units[index]
                        .name
                        .clone()
                        .unwrap_or_else(|| self.rules.units[self.units[index].kind].name.clone())
                });
                self.units[index].retreating = retreating;
                self.units[index].rebuilding = rebuilding;
                self.units[index].attacked = attacked;
                if let Some(name) = name {
                    self.record(format!("{name} is retreating."));
                }
            }
        }
        effects.winner = self.winner;
        Ok(effects)
    }
}
