//! Escort scoring rewards actual threat removal and reduced enemy access.
use crate::ai::{
    orders::{PlanningReport, TacticalObjective, TacticalOrders},
    threats::Threats,
};
use crate::{ai::context::BattleView, data::*, logic::geometry::TileSet};
use rustc_hash::FxHashMap;

#[derive(Default)]
pub(in crate::ai) struct Workspace {
    signature: Vec<(u32, usize, usize, i32, bool, bool)>,
    terrain: Vec<usize>,
    owners: Vec<Option<Side>>,
    fields: FxHashMap<u32, Field>,
    assignments: Vec<(UnitId, Vec<UnitId>, Option<(usize, usize)>)>,
}
struct Field {
    positions: Vec<f64>,
    threats: FxHashMap<u32, i32>,
}
fn pressure(board: &BattleView<'_, '_>, actor: &Unit, enemy: &Unit, blockers: TileSet) -> i32 {
    let reach = board.threat_reach_blocked(enemy, blockers);
    let coverage = Threats::coverage(board, enemy, &reach);
    board
        .weapons(enemy)
        .enumerate()
        .filter(|(weapon, _)| coverage[*weapon].contains(actor.tile))
        .map(|(weapon, _)| board.damage(enemy, actor, weapon))
        .max()
        .unwrap_or(0)
}
impl Workspace {
    pub(in crate::ai) fn prepare(
        &mut self,
        game: &BattleView<'_, '_>,
        report: &PlanningReport,
        orders: &TacticalOrders,
    ) {
        let assignments: Vec<_> = report
            .duties
            .iter()
            .filter(|d| d.id.ends_with(":escort"))
            .filter_map(|d| {
                d.actor.map(|actor| {
                    let next = game.unit(actor.0).and_then(|unit| {
                        let destination =
                            orders
                                .objectives
                                .iter()
                                .find_map(|order| match &order.objective {
                                    TacticalObjective::GoTo {
                                        actor: traveller,
                                        destination,
                                    } if *traveller == actor => Some(game.site(&destination.0)),
                                    TacticalObjective::GoTo { .. }
                                    | TacticalObjective::Occupy { .. }
                                    | TacticalObjective::Escort { .. }
                                    | TacticalObjective::ClearPassage { .. } => None,
                                })?;
                        let route = game.routes(unit.kind, &[destination]);
                        let home = game.routes(unit.kind, &[unit.tile]);
                        let budget = game.rules().units[unit.kind].r#move as u16
                            + game.cost(unit.kind, unit.tile);
                        // Reverse entry costs differ only at the endpoints.
                        // Allow allies to yield when predicting the next step.
                        (0..game.terrain().len())
                            .filter(|&tile| {
                                home[tile] != u16::MAX
                                    && home[tile].saturating_add(game.cost(unit.kind, tile))
                                        <= budget
                                    && game.at(tile).is_none_or(|other| game.allied(unit, other))
                            })
                            .min_by_key(|&tile| (route[tile], home[tile], tile))
                            .map(|tile| (tile, destination))
                    });
                    (actor, d.guards.clone(), next)
                })
            })
            .collect();
        if assignments.is_empty() {
            self.assignments.clear();
            self.fields.clear();
            return;
        }
        let same = self.assignments == assignments
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
                    unit.attacked,
                )
            });
        if same {
            return;
        }
        self.assignments = assignments.clone();
        self.signature = game
            .units()
            .iter()
            .map(|u| (u.id, u.kind, u.tile, u.hp, u.moved, u.attacked))
            .collect();
        self.terrain.clone_from(&game.terrain());
        self.owners.clone_from(&game.owners());
        self.fields.clear();
        let mut forecast = game.fork();
        forecast.end_turn();
        if forecast.winner().is_some() {
            return;
        }
        let mut evaluations = 0;
        for (actor, guards, next) in assignments {
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
                let following = next.and_then(|(site, destination)| {
                    let tiles: Vec<_> = (0..game.terrain().len())
                        .filter(|&tile| {
                            game.board().distance(site, tile) <= 2
                                && tile != site
                                && tile != actor.tile
                                && game.cost(unit.kind, tile) != u16::MAX
                        })
                        .collect();
                    // Supporting positions should leave the traveller's shortest
                    // approaches open. Narrow passages retain the same fallback.
                    let home = game.routes(actor.kind, &[actor.tile]);
                    let onward = game.routes(actor.kind, &[destination]);
                    let direct =
                        onward[actor.tile].saturating_add(game.cost(actor.kind, actor.tile));
                    let aside: Vec<_> = tiles
                        .iter()
                        .copied()
                        .filter(|&tile| {
                            home[tile]
                                .saturating_add(onward[tile])
                                .saturating_add(game.cost(actor.kind, tile))
                                > direct
                        })
                        .collect();
                    let targets = if aside.is_empty() { &tiles } else { &aside };
                    (!targets.is_empty()).then(|| game.routes(unit.kind, targets))
                });
                let mut positions: Vec<_> = std::iter::once(unit.tile)
                    .chain(game.reachable(unit).iter().map(|&(tile, _)| tile))
                    .collect();
                positions.sort_by_key(|&tile| {
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
                        following.as_ref().map_or(slack, |route| route[tile]),
                        game.board().distance(tile, actor.tile),
                        tile,
                    )
                });
                positions.truncate(32);
                // Following is a cheap route-field lookup for every guard. Only
                // the more expensive screening forecast shares the bounded budget.
                let mut scores: Vec<_> = (0..game.terrain().len())
                    .map(|tile| {
                        following.as_ref().map_or(0.0, |route| {
                            if route[unit.tile] == u16::MAX || route[tile] == u16::MAX {
                                0.0
                            } else {
                                ((route[unit.tile] as f64 - route[tile] as f64) * 6.0)
                                    .clamp(-32.0, 32.0)
                            }
                        })
                    })
                    .collect();
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
                    Field {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::orders::{DutyReport, PlanningStatus};
    use crate::logic::Game;

    #[test]
    fn escorts_reward_threat_removal_and_real_screens_without_adjacency_goals() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        game.units.retain(|u| u.is_leader());
        let actor = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .id;
        let actor_tile = game.site("1,3");
        game.units.iter_mut().find(|u| u.id == actor).unwrap().tile = actor_tile;
        let mut guard = game.unit(actor).unwrap().clone();
        guard.id = game.next_id;
        game.next_id += 1;
        guard.kind = game.kind("archer");
        guard.tile = game.site("3,3");
        guard.hp = rules.units[guard.kind].hp;
        guard.name = None;
        guard.role = None;
        let guard_id = guard.id;
        let mut enemy = guard.clone();
        enemy.id = game.next_id;
        game.next_id += 1;
        enemy.kind = game.kind("swordsman");
        enemy.hp = rules.units[enemy.kind].hp;
        enemy.side = Side::Enemy;
        enemy.tile = game.site("5,3");
        let enemy_id = enemy.id;
        game.units.extend([guard, enemy]);
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, rules.terrains["water"]);
        }
        for col in 0..game.cols {
            let tile = game.site(&format!("{col},3"));
            game.set_terrain(tile, rules.terrains["plains"]);
        }
        let branch = game.site("3,2");
        game.set_terrain(branch, rules.terrains["plains"]);
        game.roster_changed();
        let report = PlanningReport {
            duties: vec![DutyReport {
                id: "protect:escort".into(),
                actor: Some(UnitId(actor)),
                status: PlanningStatus::Fulfilled,
                reason: None,
                guards: vec![UnitId(guard_id)],
                incoming: None,
            }],
            ..PlanningReport::default()
        };
        let mut escort = Workspace::default();
        escort.prepare(
            &crate::ai::context::BattleView::new(&game),
            &report,
            &TacticalOrders::default(),
        );
        let screen = game.site("4,3");
        assert!(game.board.distance(screen, actor_tile) > 1);
        assert!(escort.position(guard_id, screen) > escort.position(guard_id, branch));
        let enemy = game.unit(enemy_id).unwrap();
        assert!(escort.attack(guard_id, enemy, enemy.hp, 1.0) > 0.0);
        let distant = game
            .units
            .iter()
            .find(|u| u.side == Side::Enemy && u.is_leader())
            .unwrap();
        assert_eq!(escort.attack(guard_id, distant, distant.hp, 1.0), 0.0);
        assert!(
            Threats::unshielded(
                &crate::ai::context::BattleView::new(&game),
                game.unit(actor).unwrap()
            ) > 0
        );
        assert_eq!(
            Threats::new(&crate::ai::context::BattleView::new(&game), Side::Player).at(
                game.unit(actor).unwrap(),
                actor_tile,
                None
            ),
            0
        );
    }
}
