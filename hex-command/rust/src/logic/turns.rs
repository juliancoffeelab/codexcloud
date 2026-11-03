//! Turn boundaries, village income, healing and campaign turn-start rules.

use super::battle::Game;
use crate::data::*;
impl<'catalog> Game<'catalog> {
    pub(crate) fn income(&self, side: Side) -> i32 {
        self.faction_income(self.faction_for_phase(side))
    }
    pub(crate) fn faction_income(&self, faction: &str) -> i32 {
        self.owners
            .iter()
            .enumerate()
            .filter(|(_, owner)| owner.as_deref() == Some(faction))
            .map(|(tile, _)| self.terrain_role(tile).income())
            .sum()
    }
    pub(crate) fn begin_turn(&mut self, side: Side) {
        self.turn = side;
        self.last_combat = None;
        self.combat_present = true;
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
            self.gold[&faction] += income;
        }
        for unit in &mut self.units {
            if unit.side != side {
                continue;
            }
            unit.moved = false;
            unit.attacked = false;
            let terrain = self.rules.terrain_roles[self.terrain[unit.tile]];
            if self.owners[unit.tile].as_deref()
                == Some(unit.faction.as_deref().unwrap_or(unit.side.faction_id()))
            {
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
            #[cfg(not(target_arch = "wasm32"))]
            let amount = healing
                .min((self.rules.units[self.units[index].kind].hp - self.units[index].hp).max(0));
            #[cfg(not(target_arch = "wasm32"))]
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
        self.evaluate_campaign(true, None);
        let factions: Vec<_> = self
            .factions
            .iter()
            .filter(|f| f.phase == side)
            .map(|f| f.id.clone())
            .collect();
        for faction in factions {
            self.event_turn_faction = Some(faction);
            self.evaluate_campaign(true, None);
        }
        self.event_turn_faction = None;
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
