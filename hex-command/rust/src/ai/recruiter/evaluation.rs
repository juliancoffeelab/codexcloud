//! Shared mechanical valuation for recruits and free, earned promotions.
use crate::{ai::context::BattleView, data::*};
use std::rc::Rc;

struct Approach {
    enemy: u32,
    weapon: usize,
    origins: Vec<usize>,
    routes: Rc<Vec<u16>>,
}
struct Support {
    healing: f64,
    thorns: f64,
    routes: Rc<Vec<u16>>,
}
pub(super) struct Assessment {
    approaches: Vec<Approach>,
    support: Vec<Support>,
    captures: Vec<(Rc<Vec<u16>>, f64)>,
    mobility: f64,
}
impl Assessment {
    pub(super) fn new(game: &BattleView<'_, '_>, unit: &Unit) -> Self {
        let rule = &game.rules().units[unit.kind];
        let enemies: Vec<_> = game
            .units()
            .iter()
            .filter(|u| u.side != unit.side)
            .collect();
        let allies: Vec<_> = game
            .units()
            .iter()
            .filter(|u| u.side == unit.side && u.id != unit.id)
            .collect();
        let mut approaches = Vec::new();
        for enemy in &enemies {
            for (weapon, attack) in rule.attacks.iter().enumerate() {
                let all = game.attack_origins(attack.category, attack.range);
                let mut origins = Vec::new();
                all[enemy.tile].for_each(|tile| {
                    if game.cost(unit.kind, tile) != u16::MAX {
                        origins.push(tile);
                    }
                });
                approaches.push(Approach {
                    enemy: enemy.id,
                    weapon,
                    routes: game.routes(unit.kind, &origins),
                    origins,
                });
            }
        }
        let healers = allies
            .iter()
            .filter(|a| game.rules().units[a.kind].healing >= rule.healing && rule.healing > 0)
            .count();
        let auras = allies
            .iter()
            .filter(|a| {
                game.rules().units[a.kind].thorn_aura >= rule.thorn_aura && rule.thorn_aura > 0
            })
            .count();
        let mut support = Vec::new();
        if rule.healing > 0 || rule.thorn_aura > 0 {
            for ally in &allies {
                let ally_rule = &game.rules().units[ally.kind];
                let mut covered = 0;
                let mut aura = 0;
                for other in &allies {
                    if other.id != ally.id && game.board().distance(other.tile, ally.tile) == 1 {
                        covered = covered.max(game.rules().units[other.kind].healing);
                        aura = aura.max(game.rules().units[other.kind].thorn_aura);
                    }
                }
                let exposed: Vec<_> = enemies
                    .iter()
                    .filter(|enemy| {
                        game.board().distance(ally.tile, enemy.tile)
                            <= game.rules().units[enemy.kind].r#move as u16 + 1
                    })
                    .collect();
                let demand = (ally_rule.hp - ally.hp).max(0) as f64
                    + if exposed.is_empty() {
                        0.0
                    } else {
                        ally_rule.hp as f64 * 0.2
                    };
                let healing =
                    marginal_healing(demand, covered, rule.healing) / (1.0 + healers as f64 * 0.5);
                // Reflection scales with incoming melee strikes. Assess each
                // enemy's best melee option, with their actual strike counts.
                let thorns = exposed
                    .iter()
                    .map(|enemy| {
                        game.rules().units[enemy.kind]
                            .attacks
                            .iter()
                            .enumerate()
                            .filter(|(_, attack)| attack.category == Category::Melee)
                            .map(|(_, attack)| {
                                f64::from((rule.thorn_aura - aura).max(0)) * f64::from(attack.count)
                            })
                            .fold(0.0, f64::max)
                    })
                    .sum::<f64>()
                    / exposed.len().max(1) as f64
                    / (1.0 + auras as f64 * 0.5);
                let goals: Vec<_> = game.board().adjacent[ally.tile]
                    .iter()
                    .copied()
                    .filter(|&t| game.cost(unit.kind, t) != u16::MAX)
                    .collect();
                support.push(Support {
                    healing,
                    thorns,
                    routes: game.routes(unit.kind, &goals),
                });
            }
        }
        let captures = game
            .owners()
            .iter()
            .enumerate()
            .filter(|(tile, owner)| {
                **owner != Some(unit.side) && game.terrain_is(*tile, Terrain::Village)
            })
            .map(|(tile, _)| {
                let covered = allies
                    .iter()
                    .map(|ally| {
                        let route = game.routes(ally.kind, &[tile])[ally.tile];
                        if route == u16::MAX {
                            0.0
                        } else {
                            1.0 / (1.0
                                + f64::from(route)
                                    / game.rules().units[ally.kind].r#move.max(1) as f64)
                        }
                    })
                    .fold(0.0, f64::max);
                (game.routes(unit.kind, &[tile]), covered)
            })
            .collect();
        let costs: Vec<_> = (0..game.terrain().len())
            .map(|tile| game.cost(unit.kind, tile))
            .filter(|&cost| cost != u16::MAX)
            .collect();
        // Both speed and actual terrain costs matter; inaccessible terrain does
        // not masquerade as a movement discount.
        let mobility = if costs.is_empty() {
            0.0
        } else {
            rule.r#move as f64 * costs.len() as f64
                / costs.iter().map(|&v| f64::from(v)).sum::<f64>().max(1.0)
                * costs.len() as f64
                / game.terrain().len().max(1) as f64
        };
        Self {
            approaches,
            support,
            captures,
            mobility,
        }
    }
    pub(super) fn score(&self, game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
        let rule = &game.rules().units[unit.kind];
        let enemies: Vec<_> = game
            .units()
            .iter()
            .filter(|u| u.side != unit.side)
            .collect();
        let mut offense = 0.0;
        let mut retaliation = 0.0;
        let mut raw = 0.0;
        let mut incoming = 0.0;
        let mut access = 0.0;
        let mut weights = 0.0;
        for enemy in &enemies {
            let enemy_rule = &game.rules().units[enemy.kind];
            let weight = 1.0
                / (1.0
                    + game.board().distance(unit.tile, enemy.tile) as f64
                        / enemy_rule.r#move.max(1) as f64);
            let mut best_offense: f64 = 0.0;
            let mut reach: f64 = 0.0;
            for approach in self.approaches.iter().filter(|a| a.enemy == enemy.id) {
                let route = approach.routes[unit.tile];
                if route == u16::MAX {
                    continue;
                }
                let turns = f64::from(route) / rule.r#move.max(1) as f64;
                let access = 1.0 / (1.0 + turns * 0.35);
                let Some(&origin) = approach
                    .origins
                    .iter()
                    .min_by_key(|&&tile| game.board().distance(unit.tile, tile))
                else {
                    continue;
                };
                let mut attacker = unit.clone();
                attacker.tile = origin;
                let (damage, response, _) = game.preview(&attacker, enemy, approach.weapon, origin);
                let attack = &rule.attacks[approach.weapon];
                let drain = damage.min(enemy.hp) as f64 * f64::from(attack.drain) / 100.0;
                let value = damage.min(enemy.hp) as f64 + drain * 0.5
                    - response.min(unit.hp) as f64 * 0.5
                    + if damage >= enemy.hp { 3.0 } else { 0.0 }
                    - if response as f64 >= unit.hp as f64 + drain {
                        unit.hp as f64
                    } else {
                        0.0
                    };
                best_offense = best_offense.max(value * access);
                reach = reach.max(access);
            }
            offense += best_offense * weight;
            access += reach * weight;
            let mut worst = None;
            for (weapon, attack) in enemy_rule.attacks.iter().enumerate() {
                let origins = game.attack_origins(attack.category, attack.range);
                let mut origin = None;
                origins[unit.tile].for_each(|tile| {
                    if game.cost(enemy.kind, tile) != u16::MAX
                        && origin.is_none_or(|old| {
                            game.board().distance(enemy.tile, tile)
                                < game.board().distance(enemy.tile, old)
                        })
                    {
                        origin = Some(tile);
                    }
                });
                let Some(origin) = origin else {
                    continue;
                };
                let mut attacker = (*enemy).clone();
                attacker.tile = origin;
                let (damage, response, _) = game.preview(&attacker, unit, weapon, origin);
                let threat = damage as f64 - response.min(enemy.hp) as f64 * 0.35;
                if worst.is_none_or(|(previous, _, _, _)| threat > previous) {
                    worst = Some((
                        threat,
                        damage,
                        response,
                        attack.damage * i32::from(attack.count),
                    ));
                }
            }
            if let Some((_, damage, response, unarmored)) = worst {
                incoming += damage as f64 * weight;
                raw += unarmored.max(1) as f64 * weight;
                retaliation += response.min(enemy.hp) as f64 * weight;
            }
            weights += weight;
        }
        if weights > 0.0 {
            offense /= weights;
            access /= weights;
            retaliation /= weights;
            raw /= weights;
            incoming /= weights;
        } else {
            offense = rule
                .attacks
                .iter()
                .map(|a| (a.damage * i32::from(a.count)) as f64)
                .fold(0.0, f64::max);
            access = 1.0;
            raw = 1.0;
            incoming = 1.0;
        }
        // Effective HP in raw incoming damage, including per-strike armor. A
        // floor keeps complete immunity to one observed weapon finite.
        let durability = unit.hp.max(1) as f64 * raw / incoming.max(raw * 0.2).max(1.0);
        let support = self
            .support
            .iter()
            .filter(|s| {
                s.routes[unit.tile] != u16::MAX && s.routes[unit.tile] as usize <= rule.r#move
            })
            .map(|s| s.healing + s.thorns)
            .sum::<f64>();
        let captures = self
            .captures
            .iter()
            .filter(|(routes, _)| routes[unit.tile] != u16::MAX)
            .map(|(routes, covered)| {
                (1.0 / (1.0 + f64::from(routes[unit.tile]) / rule.r#move.max(1) as f64) - covered)
                    .max(0.0)
            })
            .sum::<f64>();
        (offense * 1.5 + durability * access * 0.35 + retaliation * access * 0.5)
            * (1.0 + self.mobility * 0.05 + captures * 0.1)
            + support * 2.0
    }
}

fn marginal_healing(demand: f64, covered: i32, candidate: i32) -> f64 {
    (demand.min(f64::from(candidate)) - demand.min(f64::from(covered))).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strongest_only_healing_credits_only_the_increment() {
        assert_eq!(marginal_healing(12.0, 3, 3), 0.0);
        assert_eq!(marginal_healing(12.0, 3, 5), 2.0);
        assert_eq!(marginal_healing(4.0, 3, 5), 1.0);
        assert_eq!(marginal_healing(4.0, 5, 3), 0.0);
    }
}
