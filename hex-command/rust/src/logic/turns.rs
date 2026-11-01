//! Turn boundaries, village income, healing and campaign turn-start rules.

use super::battle::Game;
use crate::data::*;
impl<'catalog> Game<'catalog> {
    pub(crate) fn income(&self, side: Side) -> i32 {
        self.owners
            .iter()
            .enumerate()
            .filter(|(_, owner)| **owner == Some(side))
            .map(|(tile, _)| self.terrain_role(tile).income())
            .sum()
    }
    pub(crate) fn begin_turn(&mut self, side: Side) {
        self.turn = side;
        self.last_combat = None;
        self.combat_present = true;
        self.gold[side.index()] += self.income(side);
        for unit in &mut self.units {
            if unit.side != side {
                continue;
            }
            unit.moved = false;
            unit.attacked = false;
            let terrain = self.rules.terrain_roles[self.terrain[unit.tile]];
            if self.owners[unit.tile] == Some(side) {
                unit.hp = (unit.hp + terrain.healing()).min(self.rules.units[unit.kind].hp);
            }
        }
        for index in 0..self.units.len() {
            if self.units[index].side != side {
                continue;
            }
            let healing = self
                .units
                .iter()
                .map(|ally| self.healing_from(ally, &self.units[index]))
                .max()
                .unwrap_or(0);
            let amount = healing
                .min((self.rules.units[self.units[index].kind].hp - self.units[index].hp).max(0));
            if amount > 0 && self.recordings.is_some() {
                let target = &self.units[index];
                let providers = self
                    .units
                    .iter()
                    .filter(|ally| self.healing_from(ally, target) == healing)
                    .map(|ally| format!("u{}", ally.id))
                    .collect();
                self.recordings
                    .as_mut()
                    .unwrap()
                    .healing
                    .push(HealingRecord {
                        target: format!("u{}", target.id),
                        providers,
                        amount,
                    });
            }
            self.units[index].hp =
                (self.units[index].hp + healing).min(self.rules.units[self.units[index].kind].hp);
        }
        if side == Side::Enemy {
            self.spawn_waves();
        }
        self.evaluate_campaign(side == Side::Player, None);
    }
    pub(crate) fn end_turn(&mut self) {
        assert!(self.winner.is_none());
        if self.arena_first.is_some() {
            if Some(self.turn) != self.arena_first {
                self.round += 1;
            }
        } else if self.turn == Side::Enemy {
            self.round += 1;
        }
        self.begin_turn(self.turn.other());
    }
}
