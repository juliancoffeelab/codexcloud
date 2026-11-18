//! Turn boundaries, village income, healing and campaign turn-start rules.

use super::battle::BattlePosition;
use crate::data::*;
impl<'catalog> BattlePosition<'catalog> {
    pub(crate) fn faction_income(&self, faction: &str) -> i32 {
        self.runtime
            .owners
            .iter()
            .enumerate()
            .filter(|(_, owner)| owner.as_deref() == Some(faction))
            .map(|(tile, _)| self.terrain_definition(tile).income())
            .sum()
    }
    pub(crate) fn begin_turn(&mut self, side: TurnPhase) {
        self.runtime.turn = side;
        self.runtime.active_faction = None;
        self.runtime.last_combat = None;
        self.runtime.combat_present = true;
        let factions: Vec<_> = self
            .gold
            .keys()
            .filter(|id| self.faction(id).is_some_and(|f| f.phase == side))
            .cloned()
            .collect();
        for faction in factions {
            let income = self.faction_income(&faction);
            self.runtime.gold[&faction] += income;
        }
        for index in 0..self.runtime.units.len() {
            let unit = self.runtime.units[index];
            let facts = &mut self.runtime.unit_perception[unit.perception.0];
            if unit.phase == side {
                facts.revealed_to.clear();
            }
        }
        for index in 0..self.runtime.units.len() {
            let unit = self.runtime.units[index];
            if unit.phase != side {
                continue;
            }
            let healing = self.with_selected_capabilities_at(
                &unit,
                unit.tile,
                &self.units,
                &[],
                |source, _, _| source.is_none(),
                |caps| caps.healing(),
            );
            self.runtime.units[index].moved = false;
            self.runtime.units[index].attack_spent = false;
            self.runtime.units[index].hp =
                (unit.hp + healing).min(self.definitions.unit(unit.kind).max_hp);
        }
        for index in 0..self.runtime.units.len() {
            if self.runtime.units[index].phase != side {
                continue;
            }
            let target = &self.runtime.units[index];
            let calculate_healing = || {
                self.with_selected_capabilities_at(
                    target,
                    target.tile,
                    &self.units,
                    &[],
                    |source, _, _| source.is_some(),
                    |caps| caps.healing(),
                )
            };
            #[cfg(all(feature = "simulation", not(target_arch = "wasm32")))]
            let diagnostic = self.runtime.recordings.as_ref().map(|_| {
                self.support_diagnostic(
                    target,
                    |source, _| source.is_some(),
                    hex_command_catalog::diagnostics::healing,
                )
            });
            #[cfg(all(feature = "simulation", not(target_arch = "wasm32")))]
            let healing = match &diagnostic {
                Some(diagnostic) => diagnostic.amount,
                None => calculate_healing(),
            };
            #[cfg(any(not(feature = "simulation"), target_arch = "wasm32"))]
            let healing = calculate_healing();
            #[cfg(all(feature = "simulation", not(target_arch = "wasm32")))]
            if let Some(diagnostic) = diagnostic {
                let amount =
                    healing.min((self.definitions.unit(target.kind).max_hp - target.hp).max(0));
                if amount > 0 {
                    self.runtime
                        .recordings
                        .as_mut()
                        .expect("recording enabled")
                        .healing
                        .push(HealingRecord {
                            target: format!("u{}", target.id),
                            providers: diagnostic.source_names(None),
                            amount,
                        });
                }
            }
            self.runtime.units[index].hp = (self.runtime.units[index].hp + healing)
                .min(self.definitions.unit(self.runtime.units[index].kind).max_hp);
        }
    }
    pub(crate) fn end_turn(&mut self) {
        assert!(self.runtime.winner.is_none());
        if self.runtime.arena_first.is_some() {
            if Some(self.runtime.turn) != self.runtime.arena_first {
                self.runtime.round += 1;
            }
        } else if self.runtime.turn == TurnPhase::Ai {
            self.runtime.round += 1;
        }
        self.begin_turn(self.runtime.turn.other());
    }
}
