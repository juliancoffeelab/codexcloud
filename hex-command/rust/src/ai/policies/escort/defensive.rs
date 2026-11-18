//! Defensive escort policy: screening and threat removal without travel-position rewards.
use crate::ai::{orders::PlanningReport, policies::threat::Threats};
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*, logic::geometry::TileSet};
use hex_command_battle::PhaseForecasts;
use rustc_hash::FxHashMap;

#[derive(Default)]
pub(in crate::ai) struct EscortDefenseWorkspace {
    signature: Vec<(u32, usize, usize, i32, bool, bool)>,
    terrain: Vec<usize>,
    owners: Vec<Option<String>>,
    fields: FxHashMap<u32, EscortDefenseField>,
    assignments: Vec<(UnitId, Vec<UnitId>)>,
}
struct EscortDefenseField {
    positions: Vec<f64>,
    threats: FxHashMap<u32, i32>,
}
fn pressure(board: &BattleView<'_, '_>, actor: &Unit, enemy: &Unit, blockers: TileSet) -> i32 {
    let reach = board.threat_reach_blocked(enemy, blockers);
    let coverage = Threats::coverage(board, enemy, &reach);
    board
        .attack_reaches(enemy)
        .enumerate()
        .filter(|(weapon, _)| {
            coverage[*weapon].contains(actor.tile)
                && board.can_threaten_from(enemy, actor, *weapon, &reach)
        })
        .map(|(weapon, _)| board.damage(enemy, actor, weapon))
        .max()
        .unwrap_or(0)
}
impl EscortDefenseWorkspace {
    pub(in crate::ai) fn prepare(
        &mut self,
        phases: PhaseForecasts<'_, '_>,
        report: &PlanningReport,
    ) {
        let game = phases.view();
        let assignments: Vec<_> = report
            .duties
            .iter()
            .filter(|d| d.id.ends_with(":escort"))
            .filter_map(|d| d.actor.map(|actor| (actor, d.guards.clone())))
            .collect();
        if assignments.is_empty() {
            self.assignments.clear();
            self.fields.clear();
            return;
        }
        let same = !game.perception_sensitive()
            && self.assignments == assignments
            && self.signature.len() == game.units().len()
            && self.terrain == *game.terrain()
            && self.owners == *game.owners()
            && self.signature.iter().zip(game.units()).all(|(key, unit)| {
                *key == (
                    unit.id,
                    unit.kind,
                    unit.tile,
                    unit.hp,
                    unit.moved,
                    unit.attack_spent,
                )
            });
        if same {
            return;
        }
        self.assignments = assignments.clone();
        self.signature = game
            .units()
            .iter()
            .map(|u| (u.id, u.kind, u.tile, u.hp, u.moved, u.attack_spent))
            .collect();
        self.terrain.clone_from(&game.terrain());
        self.owners.clone_from(&game.owners());
        self.fields.clear();
        let forecast = phases.planning_next_phase();
        if forecast.winner().is_some() {
            return;
        }
        let mut evaluations = 0;
        for (actor, guards) in assignments {
            let Some(actor) = forecast.unit(actor.0) else {
                continue;
            };
            let enemies: Vec<_> = forecast
                .units()
                .iter()
                .filter(|u| forecast.hostile(actor, u))
                .collect();
            let mut blocked = TileSet::new(game.terrain().len());
            for ally in forecast
                .units()
                .iter()
                .filter(|u| forecast.allied(actor, u) && u.id != actor.id)
            {
                blocked.insert(ally.tile);
            }
            let mut threats = FxHashMap::default();
            let mut baseline = 0;
            for enemy in &enemies {
                let mut own = TileSet::new(game.terrain().len());
                own.insert(enemy.tile);
                let exposed = pressure(&forecast, actor, enemy, own);
                if exposed > 0 {
                    threats.insert(enemy.id, exposed);
                }
                let mut blockers = blocked.clone();
                blockers.insert(enemy.tile);
                baseline += pressure(&forecast, actor, enemy, blockers);
            }
            for guard in guards {
                let Some(unit) = game.unit(guard.0) else {
                    continue;
                };
                let mut positions: Vec<_> = std::iter::once(unit.tile)
                    .chain(game.reachable(unit).iter().map(|&(tile, _)| tile))
                    .collect();
                positions.order_by_key(|&tile| {
                    let slack = enemies
                        .iter()
                        .map(|enemy| {
                            game.board()
                                .distance(enemy.tile, tile)
                                .saturating_add(game.board().distance(tile, actor.tile))
                                .saturating_sub(game.board().distance(enemy.tile, actor.tile))
                        })
                        .min()
                        .unwrap_or(0);
                    (
                        tile != unit.tile,
                        slack,
                        game.board().distance(tile, actor.tile),
                        tile,
                    )
                });
                positions.truncate(32);
                let mut scores = vec![0.0; game.terrain().len()];
                for tile in positions {
                    if evaluations >= 128 {
                        break;
                    }
                    evaluations += 1;
                    let mut occupancy = blocked.clone();
                    occupancy.remove(unit.tile);
                    occupancy.insert(tile);
                    let mut after = 0;
                    for enemy in &enemies {
                        let mut blockers = occupancy.clone();
                        blockers.insert(enemy.tile);
                        after += pressure(&forecast, actor, enemy, blockers);
                    }
                    scores[tile] += ((baseline - after) as f64 * 2.0).clamp(-64.0, 32.0);
                }
                self.fields.insert(
                    unit.id,
                    EscortDefenseField {
                        positions: scores,
                        threats: threats.clone(),
                    },
                );
            }
        }
    }
    pub(in crate::ai) fn position(&self, guard: u32, tile: usize) -> f64 {
        self.fields
            .get(&guard)
            .map_or(0.0, |field| field.positions[tile])
    }
    pub(in crate::ai) fn attack(
        &self,
        guard: u32,
        target: &Unit,
        damage: i32,
        kill_chance: f64,
    ) -> f64 {
        let risk = self
            .fields
            .get(&guard)
            .and_then(|field| field.threats.get(&target.id))
            .copied()
            .unwrap_or(0);
        if risk == 0 {
            return 0.0;
        }
        kill_chance * (risk as f64 * 2.0).min(32.0)
            + (1.0 - kill_chance) * 8.0 * damage.max(0).min(target.hp) as f64
                / target.hp.max(1) as f64
    }
    pub(in crate::ai) fn bound(&self, guard: u32) -> f64 {
        if self.fields.contains_key(&guard) {
            64.0
        } else {
            0.0
        }
    }
}
