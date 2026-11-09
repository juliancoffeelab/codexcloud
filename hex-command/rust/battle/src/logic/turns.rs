//! Turn boundaries, village income, healing and campaign turn-start rules.

use super::battle::Game;
use crate::data::*;
impl<'catalog> Game<'catalog> {
    pub(crate) fn income(&self, side: Side) -> i32 {
        self.faction_income(self.faction_for_phase(side))
    }
    pub(crate) fn faction_income(&self, faction: &str) -> i32 {
        self.runtime
            .owners
            .iter()
            .enumerate()
            .filter(|(_, owner)| owner.as_deref() == Some(faction))
            .map(|(tile, _)| self.terrain_role(tile).income())
            .sum()
    }
    pub(crate) fn begin_turn(&mut self, side: Side) {
        self.runtime.turn = side;
        self.runtime.last_combat = None;
        self.runtime.combat_present = true;
        let factions: Vec<_> = self
            .gold
            .keys()
            .filter(|id| {
                self.faction(id)
                    .map_or(id.as_str() == side.faction_id(), |f| f.phase == side)
            })
            .cloned()
            .collect();
        for faction in factions {
            let income = self.faction_income(&faction);
            self.runtime.gold[&faction] += income;
        }
        let names = &self.runtime.names;
        for unit in &mut self.runtime.units {
            if unit.side != side {
                continue;
            }
            unit.moved = false;
            unit.attacked = false;
            let terrain = self.rules.terrain_roles[self.runtime.terrain[unit.tile]];
            if self.runtime.owners[unit.tile].as_deref()
                == Some(
                    names
                        .resolve(unit.faction)
                        .unwrap_or(unit.side.faction_id()),
                )
            {
                unit.hp = (unit.hp + terrain.healing()).min(self.rules.units[unit.kind].hp);
            }
        }
        for index in 0..self.runtime.units.len() {
            if self.runtime.units[index].side != side {
                continue;
            }
            let healing = self
                .units
                .iter()
                .map(|ally| self.healing_from(ally, &self.runtime.units[index]))
                .max()
                .unwrap_or(0);
            #[cfg(not(target_arch = "wasm32"))]
            let amount = healing.min(
                (self.rules.units[self.runtime.units[index].kind].hp
                    - self.runtime.units[index].hp)
                    .max(0),
            );
            #[cfg(not(target_arch = "wasm32"))]
            if amount > 0 && self.runtime.recordings.is_some() {
                let target = &self.runtime.units[index];
                let providers = self
                    .units
                    .iter()
                    .filter(|ally| self.healing_from(ally, target) == healing)
                    .map(|ally| format!("u{}", ally.id))
                    .collect();
                self.runtime
                    .recordings
                    .as_mut()
                    .unwrap()
                    .healing
                    .push(HealingRecord {
                        target: format!("u{}", target.id),
                        providers,
                        amount,
                    });
            }
            self.runtime.units[index].hp = (self.runtime.units[index].hp + healing)
                .min(self.rules.units[self.runtime.units[index].kind].hp);
        }
        self.evaluate_campaign(true, None);
    }
    pub(crate) fn end_turn(&mut self) {
        assert!(self.runtime.winner.is_none());
        if self.runtime.arena_first.is_some() {
            if Some(self.runtime.turn) != self.runtime.arena_first {
                self.runtime.round += 1;
            }
        } else if self.runtime.turn == Side::Enemy {
            self.runtime.round += 1;
        }
        self.begin_turn(self.runtime.turn.other());
    }
}
