//! Turn boundaries, village income, healing and campaign turn-start rules.

use super::battle::Game;
use crate::data::*;
impl<'catalog> Game<'catalog> {
    pub(crate) fn income(&self, side: Side) -> i32 {
        self.owners
            .iter()
            .enumerate()
            .filter(|(tile, owner)| {
                **owner == Some(side) && self.terrain_is(*tile, Terrain::Village)
            })
            .count() as i32
            * 2
    }
    pub(crate) fn begin_turn(&mut self, side: Side) {
        self.turn = side;
        if let Some(combat) = self.last_combat.take() {
            self.last_combat_spare = Some(combat);
        }
        self.combat_present = true;
        self.gold[side.index()] += self.income(side);
        for unit in &mut self.units {
            if unit.side != side {
                continue;
            }
            unit.moved = false;
            unit.attacked = false;
            let terrain = self.rules.terrain_types[self.terrain[unit.tile]];
            if matches!(terrain, Terrain::Village | Terrain::Grove)
                && self.owners[unit.tile] == Some(side)
            {
                unit.hp = (unit.hp + 4).min(self.rules.units[unit.kind].hp);
            }
        }
        for index in 0..self.units.len() {
            if self.units[index].side != side {
                continue;
            }
            let healing = self
                .units
                .iter()
                .filter(|ally| {
                    ally.side == side
                        && ally.id != self.units[index].id
                        && self.board.distance(ally.tile, self.units[index].tile) == 1
                })
                .map(|ally| self.rules.units[ally.kind].healing)
                .max()
                .unwrap_or(0);
            #[cfg(not(target_arch = "wasm32"))]
            let amount = healing
                .min((self.rules.units[self.units[index].kind].hp - self.units[index].hp).max(0));
            #[cfg(not(target_arch = "wasm32"))]
            if amount > 0 && self.recordings.is_some() {
                let target = &self.units[index];
                let providers = self
                    .units
                    .iter()
                    .filter(|ally| {
                        ally.side == side
                            && ally.id != target.id
                            && self.board.distance(ally.tile, target.tile) == 1
                            && self.rules.units[ally.kind].healing == healing
                    })
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
