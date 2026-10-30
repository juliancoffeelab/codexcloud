//! Shared mechanical valuation for recruits and free, earned promotions.
use crate::{ai::context::BattleView, data::*};
use std::rc::Rc;

struct Approach {
    enemy: usize,
    weapon: usize,
    origins: Vec<usize>,
    routes: Rc<Vec<u16>>,
}
// Before four troops are visible, blend the known legal enemy roster into the
// matchup estimate. This prior prevents opening purchases from countering only
// a leader. Observed troops progressively replace it, including after losses.

struct Support {
    healing: f64,
    thorns: f64,
    routes: Rc<Vec<u16>>,
}
#[derive(serde::Serialize)]
pub(super) struct Score {
    pub offense: f64,
    pub effective_hp: f64,
    pub access: f64,
    pub retaliation: f64,
    pub support: f64,
    pub thorns: f64,
    pub mobility: f64,
    pub captures: f64,
    pub leader_pressure: f64,
    pub movement_bonus: f64,
    pub healing: f64,
    pub aura_thorns: f64,
    pub total: f64,
}

pub(super) struct Assessment {
    weights: RecruitmentWeights,
    forecasts: Vec<Unit>,
    forecast_weight: f64,
    approaches: Vec<Approach>,
    support: Vec<Support>,
    captures: Vec<(Rc<Vec<u16>>, f64)>,
    friendly_count: usize,
}
impl Assessment {
    #[cfg(test)]
    pub(super) fn new(game: &BattleView<'_, '_>, unit: &Unit) -> Self {
        Self::with_forecast(game, unit, false)
    }
    pub(super) fn with_forecast(game: &BattleView<'_, '_>, unit: &Unit, predict: bool) -> Self {
        Self::with_weights(game, unit, predict, game.rules().recruitment.standard)
    }
    pub(super) fn with_weights(
        game: &BattleView<'_, '_>,
        unit: &Unit,
        predict: bool,
        weights: RecruitmentWeights,
    ) -> Self {
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
        let mut forecasts = Vec::new();
        let mut forecast_weight = 0.0;
        if predict {
            let actual = enemies
                .iter()
                .filter(|e| !game.rules().units[e.kind].leader)
                .count();
            if let Some(leader) = enemies.iter().find(|e| game.rules().units[e.kind].leader) {
                let options = game.recruitment_options(leader.side);
                forecast_weight =
                    (weights.future_enemies - actual as f64).max(0.0) / options.len().max(1) as f64;
                if forecast_weight > 0.0 {
                    for kind in options {
                        let mut forecast = (*leader).clone();
                        forecast.kind = kind;
                        forecast.hp = game.rules().units[kind].hp;
                        forecast.id = 0;
                        forecasts.push(forecast);
                    }
                }
            }
        }
        let mut approaches = Vec::new();
        for (index, enemy) in enemies.iter().copied().chain(forecasts.iter()).enumerate() {
            for (weapon, attack) in rule.attacks.iter().enumerate() {
                let all = game.attack_origins(attack.category, attack.range);
                let mut origins = Vec::new();
                all[enemy.tile].for_each(|tile| {
                    if game.cost(unit.kind, tile) != u16::MAX {
                        origins.push(tile);
                    }
                });
                approaches.push(Approach {
                    enemy: index,
                    weapon,
                    routes: game.routes(unit.kind, &origins),
                    origins,
                });
            }
        }
        // An existing provider cannot support itself. Treat remote providers'
        // future coverage as independent; current adjacent coverage is exact.
        let overlap_probability = (weights.support_recipients
            / allies.len().saturating_sub(1).max(1) as f64
            * weights.healing_overlap)
            .min(1.0);
        let aura_overlap_probability = (weights.support_recipients
            / allies.len().saturating_sub(1).max(1) as f64
            * weights.aura_overlap)
            .min(1.0);
        let mut support = Vec::new();
        if rule.healing > 0 || rule.thorn_aura > 0 {
            for ally in &allies {
                let ally_rule = &game.rules().units[ally.kind];
                let mut covered = 0;
                let mut aura = 0;
                let mut remote_healing = Vec::new();
                let mut remote_auras = Vec::new();
                for other in &allies {
                    if other.id == ally.id {
                        continue;
                    }
                    let other_rule = &game.rules().units[other.kind];
                    if game.board().distance(other.tile, ally.tile) == 1 {
                        covered = covered.max(other_rule.healing);
                        aura = aura.max(other_rule.thorn_aura);
                    } else {
                        if other_rule.healing > 0 {
                            remote_healing.push(other_rule.healing);
                        }
                        if other_rule.thorn_aura > 0 {
                            remote_auras.push(other_rule.thorn_aura);
                        }
                    }
                }
                // Anticipate support use without a nearby-enemy gate.
                let healing = expected_marginal_support(
                    (ally_rule.hp - ally.hp).max(0) as f64
                        + ally_rule.hp as f64 * weights.future_wounds,
                    covered,
                    rule.healing,
                    &mut remote_healing,
                    overlap_probability,
                );
                let extra_aura = expected_marginal_support(
                    f64::from(rule.thorn_aura),
                    aura,
                    rule.thorn_aura,
                    &mut remote_auras,
                    aura_overlap_probability,
                );
                // Classify the enemy's preferred exchange against this ally,
                // including the proposed aura, rather than any melee backup.
                let thorns = if extra_aura > 0.0 {
                    extra_aura
                        * expected_weighted_thorn_triggers(
                            enemies
                                .iter()
                                .copied()
                                .chain(forecasts.iter())
                                .enumerate()
                                .filter_map(|(index, enemy)| {
                                    dominant_melee_strikes(
                                        game,
                                        enemy,
                                        ally,
                                        Some(aura.max(rule.thorn_aura)),
                                    )
                                    .map(|count| {
                                        (
                                            count,
                                            if index < enemies.len() {
                                                1.0
                                            } else {
                                                forecast_weight
                                            },
                                        )
                                    })
                                }),
                            allies.len() + 1,
                        )
                } else {
                    0.0
                };
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
        Self {
            weights,
            forecasts,
            forecast_weight,
            approaches,
            support,
            captures,
            friendly_count: allies.len() + 1,
        }
    }
    pub(super) fn score(&self, game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
        self.breakdown(game, unit).total
    }
    pub(super) fn recruitment_breakdown(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Score {
        self.valuation(game, unit, true)
    }
    pub(super) fn breakdown(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Score {
        self.valuation(game, unit, false)
    }
    fn valuation(&self, game: &BattleView<'_, '_>, unit: &Unit, recruitment: bool) -> Score {
        // Recruitment can reward village capture and configured mobility; earned
        // promotions retain village value. Enemy relevance never depends on distance.
        let rule = &game.rules().units[unit.kind];
        let enemies: Vec<_> = game
            .units()
            .iter()
            .filter(|u| u.side != unit.side)
            .collect();
        let coefficients = self.weights;
        let mut leader_pressure: f64 = 0.0;
        let mut offense = 0.0;
        let mut retaliation = 0.0;
        let mut raw = 0.0;
        let mut incoming = 0.0;
        let mut access = 0.0;
        let mut weights = 0.0;
        for (index, enemy) in enemies
            .iter()
            .copied()
            .chain(self.forecasts.iter())
            .enumerate()
        {
            let enemy_rule = &game.rules().units[enemy.kind];
            let weight = if index >= enemies.len() {
                self.forecast_weight
            } else {
                1.0
            };
            let mut best_offense: f64 = 0.0;
            let mut reach: f64 = 0.0;
            for approach in self.approaches.iter().filter(|a| a.enemy == index) {
                let route = approach.routes[unit.tile];
                if route == u16::MAX {
                    continue;
                }
                let access = 1.0;
                let origins = representative_origins(game, unit.kind, unit.tile, &approach.origins);
                if origins.is_empty() {
                    continue;
                }
                let mut value = 0.0;
                let mut leader_damage = 0.0;
                for &origin in &origins {
                    let mut attacker = unit.clone();
                    attacker.tile = origin;
                    let forecast = game.exchange(&attacker, enemy, approach.weapon, origin);
                    let damage = forecast.damage[0].round() as i32;
                    let response = forecast.damage[1].round() as i32;
                    let attack = &rule.attacks[approach.weapon];
                    let drain = damage.min(enemy.hp) as f64 * f64::from(attack.drain) / 100.0;
                    let exchange = forecast.effective[0] + drain * coefficients.drain
                        - response.min(unit.hp) as f64 * coefficients.exchange_retaliation
                        + forecast.death[1] * coefficients.kill
                        - if response as f64 >= unit.hp as f64 + drain {
                            unit.hp as f64
                        } else {
                            0.0
                        };
                    value += exchange;
                    leader_damage += damage.min(enemy.hp).max(0) as f64;
                }
                value /= origins.len() as f64;
                if recruitment && coefficients.leader_pressure > 0.0 && enemy_rule.leader {
                    let turns = f64::from(route) / rule.r#move.max(1) as f64;
                    let fraction = leader_damage / origins.len() as f64 / enemy.hp.max(1) as f64;
                    leader_pressure = leader_pressure.max(fraction.min(1.0) / (1.0 + turns));
                }
                best_offense = best_offense.max(value * access);
                reach = reach.max(access);
            }
            offense += best_offense * weight;
            access += reach * weight;
            // A common unarmoured, terrain-neutral target takes the enemy's
            // strongest raw attack. This reference cannot change when armour,
            // retaliation or thorns makes that enemy switch weapons.
            let reference = enemy_rule
                .attacks
                .iter()
                .map(|a| a.damage * i32::from(a.count))
                .max()
                .unwrap_or(0);
            raw += f64::from(reference.max(1)) * weight;
            if let Some(preferred) = preferred_attack(game, enemy, unit, None) {
                let attack = &enemy_rule.attacks[preferred.weapon];
                incoming += preferred.damage * weight;
                // Own reflection is valued separately by army pressure.
                // Preserve ordinary retaliation and existing friendly aura.
                let response = preferred.response
                    - if attack.category == Category::Melee {
                        f64::from(rule.thorns) * preferred.strikes
                    } else {
                        0.0
                    };
                retaliation += response.max(0.0).min(enemy.hp as f64) * weight;
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
        let support = support_coverage(
            self.support
                .iter()
                .filter_map(|s| (s.routes[unit.tile] != u16::MAX).then_some(s.healing + s.thorns)),
            coefficients.support_recipients,
        );
        let aura_thorns = support_coverage(
            self.support
                .iter()
                .filter_map(|s| (s.routes[unit.tile] != u16::MAX).then_some(s.thorns)),
            coefficients.support_recipients,
        );
        let healing = (support - aura_thorns).max(0.0);
        let captures = self
            .captures
            .iter()
            .filter(|(routes, _)| routes[unit.tile] != u16::MAX)
            .map(|(routes, covered)| {
                (1.0 / (1.0 + f64::from(routes[unit.tile]) / rule.r#move.max(1) as f64) - covered)
                    .max(0.0)
            })
            .sum::<f64>();
        let thorns = if rule.thorns > 0 {
            let pressure = expected_weighted_thorn_triggers(
                enemies
                    .iter()
                    .copied()
                    .chain(self.forecasts.iter())
                    .enumerate()
                    .filter_map(|(index, enemy)| {
                        dominant_melee_strikes(game, enemy, unit, None).map(|count| {
                            (
                                count,
                                if index < enemies.len() {
                                    1.0
                                } else {
                                    self.forecast_weight
                                },
                            )
                        })
                    }),
                self.friendly_count,
            );
            f64::from(rule.thorns) * pressure
        } else {
            0.0
        };
        // Recruitment buys a unit's future contributions: endurance helps an
        // effective attacker deliver them repeatedly. Adding HP independently
        // can instead buy durable units with little useful damage. Support
        // remains separate from sustained weapon contribution.
        // Earned promotions retain their existing valuation.
        let combat = if recruitment {
            // Linear endurance values sustained contribution. Normalize to
            // the former square-root score at 20 effective HP so retaliation
            // and support keep the same relative scale at that reference.
            let endurance = durability / coefficients.endurance_reference_hp.sqrt();
            offense * coefficients.offense * endurance
                + retaliation * access * coefficients.retaliation
        } else {
            offense * 1.5 + durability * access * 0.35 + retaliation * access * 0.5
        };
        let mobility = if recruitment && coefficients.speed > 0.0 {
            terrain_speed(game, unit)
        } else {
            0.0
        };
        let movement_bonus = if recruitment {
            movement_bonus(captures, mobility, leader_pressure, coefficients)
        } else {
            captures * 0.1
        };
        // Keep the old arithmetic when the support weights agree. The aura
        // adjustment exposes a separate coefficient without changing defaults.
        let total = combat * (1.0 + movement_bonus)
            + (support + thorns) * coefficients.healing
            + aura_thorns * (coefficients.aura_thorns - coefficients.healing)
            + thorns * (coefficients.intrinsic_thorns - coefficients.healing);
        Score {
            offense,
            effective_hp: durability,
            access,
            retaliation,
            support,
            thorns,
            mobility,
            captures,
            leader_pressure,
            movement_bonus,
            healing,
            aura_thorns,
            total,
        }
    }
}

// Intrinsic speed uses typical traversable terrain cost, not distance to troops.
fn terrain_speed(game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
    let costs: Vec<_> = (0..game.board().ids.len())
        .map(|t| game.cost(unit.kind, t))
        .filter(|&c| c != u16::MAX)
        .collect();
    let mean = costs.iter().map(|&x| f64::from(x)).sum::<f64>() / costs.len().max(1) as f64;
    ((game.rules().units[unit.kind].r#move as f64 / mean.max(1.0) - 3.0) / 3.0).clamp(0.0, 1.0)
}

// Speed and leader delivery describe alternative uses of the same mobility.
// Take their maximum, then add distinct marginal village value, with one cap.
fn movement_bonus(captures: f64, speed: f64, leader: f64, weights: RecruitmentWeights) -> f64 {
    let bonus = captures * weights.village_capture
        + (speed * weights.speed).max(leader * weights.leader_pressure);
    weights.movement_cap.map_or(bonus, |cap| bonus.min(cap))
}

// Forecast types carry fractional troop weights. Ranged preferences contribute
// no melee triggers; the six-attacker cap remains the same as observed pressure.
fn expected_weighted_thorn_triggers(
    strikes: impl Iterator<Item = (f64, f64)>,
    friendly_count: usize,
) -> f64 {
    let (enemies, strikes) = strikes
        .filter(|&(count, weight)| count > 0.0 && weight > 0.0)
        .fold((0.0, 0.0), |(enemies, strikes), (count, weight)| {
            (enemies + weight, strikes + count * weight)
        });
    if enemies == 0.0 {
        0.0
    } else {
        (enemies / friendly_count.max(1) as f64).min(6.0) * strikes / enemies
    }
}

// Representative legal origins let melee and ranged weapons compete even when
// the enemy is currently distant. As with engine best_attack, choose by damage,
// drain, retaliation, kills and survival, not raw damage or strike count alone.
// An aura override replaces actual adjacent aura in previews, so promotions do
// not count their old aura as well as their proposed new one.
fn dominant_melee_strikes(
    game: &BattleView<'_, '_>,
    enemy: &Unit,
    target: &Unit,
    aura: Option<i32>,
) -> Option<f64> {
    preferred_attack(game, enemy, target, aura).map(|preferred| {
        let attack = &game.rules().units[enemy.kind].attacks[preferred.weapon];
        if attack.category == Category::Melee {
            preferred.strikes
        } else {
            0.0
        }
    })
}

// Equally sample terrain-reachable legal exchange positions. Travel distance
// never chooses a representative or changes the matchup's weight.
fn representative_origins(
    game: &BattleView<'_, '_>,
    kind: usize,
    from: usize,
    origins: &[usize],
) -> Vec<usize> {
    origins
        .iter()
        .copied()
        .filter(|&tile| game.routes(kind, &[tile])[from] != u16::MAX)
        .collect()
}

struct PreferredAttack {
    weapon: usize,
    damage: f64,
    response: f64,
    strikes: f64,
}

fn preferred_attack(
    game: &BattleView<'_, '_>,
    enemy: &Unit,
    target: &Unit,
    aura: Option<i32>,
) -> Option<PreferredAttack> {
    let rule = &game.rules().units[enemy.kind];
    let mut best = None;
    for (weapon, attack) in rule.attacks.iter().enumerate() {
        let origins = game.attack_origins(attack.category, attack.range);
        let mut legal = Vec::new();
        origins[target.tile].for_each(|tile| {
            if game.cost(enemy.kind, tile) != u16::MAX {
                legal.push(tile);
            }
        });
        let origins = representative_origins(game, enemy.kind, enemy.tile, &legal);
        if origins.is_empty() {
            continue;
        }
        let mut value = 0.0;
        let mut damage_sum = 0.0;
        let mut response_sum = 0.0;
        let mut strikes_sum = 0.0;
        for &origin in &origins {
            let mut attacker = enemy.clone();
            attacker.tile = origin;
            let forecast = if let Some(aura) = aura {
                let reflection = if attack.category == Category::Melee {
                    game.rules().units[target.kind].thorns + aura
                } else {
                    0
                };
                game.exchange_with_thorns(&attacker, target, weapon, origin, reflection)
            } else {
                game.exchange(&attacker, target, weapon, origin)
            };
            let damage = forecast.damage[0];
            let response = forecast.damage[1];
            let exchange = forecast.effective[0] + forecast.healing[0] + forecast.death[1] * 18.0
                - response * 0.7
                - forecast.death[0] * 40.0;
            value += exchange;
            damage_sum += damage;
            response_sum += response;
            strikes_sum += forecast.strikes[0];
        }
        value /= origins.len() as f64;
        let damage = damage_sum / origins.len() as f64;
        let response = response_sum / origins.len() as f64;
        let strikes = strikes_sum / origins.len() as f64;
        if best.as_ref().is_none_or(|(score, _)| value > *score) {
            best = Some((
                value,
                PreferredAttack {
                    weapon,
                    damage,
                    response,
                    strikes,
                },
            ));
        }
    }
    best.map(|(_, attack)| attack)
}

// With equal providers this is marginal_value * (1 - overlap_probability)^count.
// Descending strengths also handle upgrades correctly: stronger existing buffs
// erase the gain; weaker ones leave only the candidate's incremental benefit.
fn expected_marginal_support(
    demand: f64,
    covered: i32,
    candidate: i32,
    others: &mut [i32],
    overlap_probability: f64,
) -> f64 {
    others.sort_unstable_by(|a, b| b.cmp(a));
    let mut uncovered_probability = 1.0;
    let mut value = 0.0;
    for &strength in others.iter() {
        value += uncovered_probability
            * overlap_probability
            * marginal_healing(demand, covered.max(strength), candidate);
        uncovered_probability *= 1.0 - overlap_probability;
    }
    value + uncovered_probability * marginal_healing(demand, covered, candidate)
}

// Recruitment estimates typical coverage, rather than summing mutually
// exclusive future positions. This is a prior, not a measured runtime statistic.
fn support_coverage(values: impl Iterator<Item = f64>, recipients: f64) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), value| {
        (sum + value, count + 1)
    });
    if count == 0 {
        0.0
    } else {
        sum * recipients.min(count as f64) / count as f64
    }
}

fn marginal_healing(demand: f64, covered: i32, candidate: i32) -> f64 {
    (demand.min(f64::from(candidate)) - demand.min(f64::from(covered))).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn movement_controls_are_optional_separate_and_share_a_cap() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units.to_mut()[enemy].leader = true;
        rules.units.to_mut()[target].r#move = 6;
        let (mut game, target_id, enemy_id) = thorn_battle(&rules, target, enemy);
        let a = game.site("1,4");
        let b = game.site("8,4");
        game.units
            .iter_mut()
            .find(|u| u.id == target_id)
            .unwrap()
            .tile = a;
        game.units
            .iter_mut()
            .find(|u| u.id == enemy_id)
            .unwrap()
            .tile = b;
        game.owners.fill(Some(Side::Player));
        let view = BattleView::new(&game);
        let unit = game.unit(target_id).unwrap();
        let score = |weights| {
            Assessment::with_weights(&view, unit, false, weights).recruitment_breakdown(&view, unit)
        };
        let baseline = score(RecruitmentWeights::default());
        let speed = score(RecruitmentWeights {
            speed: 0.5,
            ..RecruitmentWeights::default()
        });
        let leader = score(RecruitmentWeights {
            leader_pressure: 2.0,
            ..RecruitmentWeights::default()
        });
        assert_eq!(baseline.movement_bonus, 0.0);
        assert!(speed.movement_bonus > 0.0 && leader.movement_bonus > 0.0);
        assert_eq!(speed.leader_pressure, 0.0);
        assert_eq!(leader.mobility, 0.0);
        assert_eq!(baseline.offense, leader.offense);
        assert_eq!(baseline.effective_hp, speed.effective_hp);
        let capped = score(RecruitmentWeights {
            speed: 100.0,
            leader_pressure: 100.0,
            movement_cap: Some(0.25),
            ..RecruitmentWeights::default()
        });
        assert_eq!(capped.movement_bonus, 0.25);
        assert_eq!(
            movement_bonus(
                0.0,
                1.0,
                1.0,
                RecruitmentWeights {
                    speed: 0.4,
                    leader_pressure: 0.4,
                    ..RecruitmentWeights::default()
                }
            ),
            0.4
        );
    }

    #[test]
    fn thorn_pressure_caps_attackers_before_strikes_and_ignores_ranged_enemies() {
        assert_eq!(
            expected_weighted_thorn_triggers([(3.0, 1.0); 20].into_iter(), 1),
            18.0
        );
        assert_eq!(
            expected_weighted_thorn_triggers([(1.0, 1.0)].into_iter(), 10),
            0.1
        );
        assert_eq!(
            expected_weighted_thorn_triggers([(0.0, 1.0); 20].into_iter(), 1),
            0.0
        );
        assert_eq!(
            expected_weighted_thorn_triggers(
                [(0.0, 1.0), (0.0, 1.0), (1.0, 1.0), (3.0, 1.0)].into_iter(),
                4
            ),
            1.0
        );
        assert_eq!(expected_weighted_thorn_triggers([].into_iter(), 1), 0.0);
    }

    fn thorn_fixture() -> (Rules, usize, usize) {
        let mut rules = Rules::load().unwrap();
        let target = rules.types["swordsman"];
        let enemy = rules.types["archer"];
        for kind in [target, enemy] {
            let rule = &mut rules.units.to_mut()[kind];
            rule.hp = 100;
            rule.thorns = 0;
            rule.thorn_aura = 0;
            rule.healing = 0;
            rule.attacks.to_mut().truncate(1);
            rule.attacks.to_mut()[0].category = Category::Melee;
            rule.attacks.to_mut()[0].range = 1;
            rule.attacks.to_mut()[0].damage_type = DamageType::Blade;
            rule.attacks.to_mut()[0].damage = 2;
            rule.attacks.to_mut()[0].count = 1;
            rule.attacks.to_mut()[0].drain = 0;
            rule.damage_modifiers = DamageModifiers {
                blade: 0,
                pierce: 0,
                impact: 0,
            };
            for environment in rule.environment.to_mut().iter_mut() {
                *environment = [[0; 3]; 2];
            }
        }
        rules.units.to_mut()[enemy].attacks.to_mut()[0].damage = 8;
        rules.units.to_mut()[enemy].attacks.to_mut()[0].count = 3;
        (rules, target, enemy)
    }

    fn thorn_battle(
        rules: &Rules,
        target: usize,
        enemy: usize,
    ) -> (crate::logic::Game<'_>, u32, u32) {
        let mut game = crate::logic::Game::new(rules, "mirror");
        game.units.clear();
        game.roster_changed();
        let tile = game.site("4,4");
        let target = game.add(target, Side::Player, tile);
        let tile = game.site("5,4");
        let enemy = game.add(enemy, Side::Enemy, tile);
        (game, target, enemy)
    }

    #[test]
    fn dominant_attack_accounts_for_ranged_alternatives_armor_and_proposed_aura() {
        let (mut rules, target, enemy) = thorn_fixture();
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        ranged.damage_type = DamageType::Pierce;
        ranged.damage = 20;
        ranged.count = 1;
        rules.units.to_mut()[enemy].attacks.to_mut().push(ranged);
        {
            let (game, target, enemy) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let target = game.unit(target).unwrap();
            let enemy = game.unit(enemy).unwrap();
            assert!(
                (dominant_melee_strikes(&view, enemy, target, None).unwrap() - 3.0).abs() < 1e-9
            );
            // The buff makes melee a worse exchange, so the enemy avoids it.
            assert_eq!(
                dominant_melee_strikes(&view, enemy, target, Some(2)),
                Some(0.0)
            );
        }
        rules.units.to_mut()[target].damage_modifiers.blade = -2;
        let (game, target, enemy) = thorn_battle(&rules, target, enemy);
        let view = BattleView::new(&game);
        assert_eq!(
            dominant_melee_strikes(
                &view,
                game.unit(enemy).unwrap(),
                game.unit(target).unwrap(),
                None
            ),
            Some(0.0)
        );
    }

    #[test]
    fn recruitment_anticipates_legal_enemy_rosters_until_observed_troops_replace_them() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units.to_mut()[enemy].leader = true;
        rules.units.to_mut()[target].attacks.to_mut()[0].category = Category::Ranged;
        rules.units.to_mut()[target].attacks.to_mut()[0].range = 2;
        rules.units.to_mut()[target].attacks.to_mut()[0].damage = 12;
        rules.units.to_mut()[target].attacks.to_mut()[0].spread = 0;
        let resistant = rules.types["scout"];
        let observed = rules.types["spearman"];
        for kind in [resistant, observed] {
            let mut rule = rules.units[enemy].clone();
            rule.id = rules.units[kind].id.clone();
            rule.leader = false;
            rules.units.to_mut()[kind] = rule;
            rules.route_classes.to_mut()[kind] = kind;
        }
        rules.units.to_mut()[resistant].damage_modifiers.blade = -8;
        let (mut game, target, _) = thorn_battle(&rules, target, enemy);
        game.owners.fill(Some(Side::Player));
        let assess = |game: &crate::logic::Game<'_>| {
            let view = BattleView::new(game);
            let unit = game.unit(target).unwrap();
            let assessment = Assessment::with_forecast(&view, unit, true);
            (
                assessment.recruitment_breakdown(&view, unit).offense,
                Assessment::new(&view, unit).breakdown(&view, unit).total,
                Assessment::new(&view, unit)
                    .recruitment_breakdown(&view, unit)
                    .total,
            )
        };
        game.pools = Some([vec![], vec![]]);
        let neutral = assess(&game);
        game.pools = Some([vec![], vec![resistant]]);
        let anticipated = assess(&game);
        assert!(anticipated.0 < neutral.0);
        // Free promotions continue to consider the actual battlefield only.
        assert_eq!(anticipated.1, neutral.1);
        // Situational never reads the hidden future recruit pool.
        assert_eq!(anticipated.2, neutral.2);
        for site in ["5,3", "6,3", "6,4", "6,5"] {
            let tile = game.site(site);
            game.add(observed, Side::Enemy, tile);
        }
        let observed_with_prior = assess(&game);
        game.pools = Some([vec![], vec![]]);
        assert_eq!(observed_with_prior, assess(&game));
    }

    #[test]
    fn only_predictor_credits_thorns_against_future_melee_troops() {
        for aura in [false, true] {
            let (mut rules, target, enemy) = thorn_fixture();
            let melee = rules.types["scout"];
            rules.units.to_mut()[melee] = rules.units.to_mut()[enemy].clone();
            rules.units.to_mut()[enemy].leader = true;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].category = Category::Ranged;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].range = 2;
            if aura {
                rules.units.to_mut()[target].thorn_aura = 4;
            } else {
                rules.units.to_mut()[target].thorns = 4;
            }
            let (mut game, target_id, _) = thorn_battle(&rules, target, enemy);
            if aura {
                let tile = game.site("3,4");
                game.add(target, Side::Player, tile);
            }
            game.pools = Some([vec![], vec![melee]]);
            let view = BattleView::new(&game);
            let unit = game.unit(target_id).unwrap();
            let situational = Assessment::new(&view, unit).recruitment_breakdown(&view, unit);
            let predictor =
                Assessment::with_forecast(&view, unit, true).recruitment_breakdown(&view, unit);
            if aura {
                assert_eq!(situational.support, 0.0);
                assert!(predictor.support > 0.0);
            } else {
                assert_eq!(situational.thorns, 0.0);
                assert!(predictor.thorns > 0.0);
            }
        }
    }

    #[test]
    fn effective_hp_and_thorn_weapon_preferences_do_not_choose_the_nearest_terrain() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units.to_mut()[target].attacks.to_mut()[0].spread = 0;
        rules.units.to_mut()[enemy].attacks.to_mut()[0].spread = 0;
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        ranged.damage = 20;
        ranged.count = 1;
        rules.units.to_mut()[enemy].attacks.to_mut().push(ranged);
        let forest = rules.terrains["forest"];
        rules.units.to_mut()[enemy].environment.to_mut()[forest][Category::Melee.index()]
            [DamageType::Blade.index()] = 10;
        let (mut game, target, enemy) = thorn_battle(&rules, target, enemy);
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, rules.terrains["plains"]);
        }
        let tile = game.site("3,4");
        game.set_terrain(tile, forest);
        let mut health = Vec::new();
        for site in ["1,4", "8,4"] {
            let tile = game.site(site);
            let index = game.index(enemy).unwrap();
            game.units[index].tile = tile;
            game.roster_changed();
            let view = BattleView::new(&game);
            let defender = game.unit(target).unwrap();
            let attacker = game.unit(enemy).unwrap();
            assert_eq!(
                preferred_attack(&view, attacker, defender, None)
                    .unwrap()
                    .weapon,
                0
            );
            assert_eq!(
                preferred_attack(&view, attacker, defender, Some(1))
                    .unwrap()
                    .weapon,
                1
            );
            health.push(
                Assessment::new(&view, defender)
                    .recruitment_breakdown(&view, defender)
                    .effective_hp,
            );
        }
        assert!((health[0] - health[1]).abs() < 1e-9);
    }

    #[test]
    fn effective_hp_keeps_a_common_reference_when_armour_changes_preferred_weapon() {
        let (mut rules, target, enemy) = thorn_fixture();
        let mut ranged = rules.units[enemy].attacks[0].clone();
        ranged.category = Category::Ranged;
        ranged.range = 2;
        ranged.damage_type = DamageType::Pierce;
        ranged.damage = 20;
        ranged.count = 1;
        rules.units.to_mut()[enemy].attacks.to_mut().push(ranged);
        for (blade, expected_weapon, expected_hp) in [(0, 0, 100.0), (-2, 1, 120.0)] {
            rules.units.to_mut()[target].damage_modifiers.blade = blade;
            let (game, target, enemy) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            assert_eq!(
                preferred_attack(&view, game.unit(enemy).unwrap(), unit, None)
                    .unwrap()
                    .weapon,
                expected_weapon
            );
            let score = Assessment::new(&view, unit).breakdown(&view, unit);
            assert!((score.effective_hp - expected_hp).abs() < 1e-9);
        }
    }

    #[test]
    fn effective_hp_values_repeated_ranged_attacks_and_per_strike_armour() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units.to_mut()[target].hp = 40;
        rules.units.to_mut()[enemy].attacks.to_mut()[0].category = Category::Ranged;
        rules.units.to_mut()[enemy].attacks.to_mut()[0].range = 2;
        for (damage, count, armour, expected) in [
            (10, 1, 0, 40.0),
            (10, 1, -2, 50.0),
            (5, 2, -2, 40.0 * 10.0 / 6.0),
            (5, 2, -5, 200.0),
        ] {
            rules.units.to_mut()[enemy].attacks.to_mut()[0].damage = damage;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].count = count;
            rules.units.to_mut()[target].damage_modifiers.blade = armour;
            let (game, target, _) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            let score = Assessment::new(&view, unit).breakdown(&view, unit);
            assert!((score.effective_hp - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn recruitment_values_endurance_together_with_damage_without_changing_promotions() {
        let mut scores = Vec::new();
        // The old additive score prefers the much tougher, weak attacker.
        // Recruiting should instead value the useful damage it can sustain.
        for (hp, damage) in [(100, 2), (30, 8)] {
            let (mut rules, target, enemy) = thorn_fixture();
            rules.units.to_mut()[target].hp = hp;
            let attack = &mut rules.units.to_mut()[target].attacks.to_mut()[0];
            attack.category = Category::Ranged;
            attack.range = 2;
            attack.damage = damage;
            attack.spread = 0;
            let (game, target, _) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            let assessment = Assessment::new(&view, unit);
            scores.push((
                assessment.score(&view, unit),
                assessment.recruitment_breakdown(&view, unit).total,
            ));
        }
        assert!(scores[0].0 > scores[1].0);
        assert!(scores[1].1 > scores[0].1);
    }

    #[test]
    fn recruitment_values_sustained_damage_linearly_with_endurance() {
        let mut scores = Vec::new();
        for hp in [40, 80] {
            let (mut rules, target, enemy) = thorn_fixture();
            rules.units.to_mut()[target].hp = 80;
            let attack = &mut rules.units.to_mut()[target].attacks.to_mut()[0];
            attack.category = Category::Ranged;
            attack.range = 2;
            attack.damage = 8;
            attack.spread = 0;
            let (mut game, target_id, _) = thorn_battle(&rules, target, enemy);
            game.units
                .iter_mut()
                .find(|u| u.id == target_id)
                .unwrap()
                .hp = hp;
            game.owners.fill(Some(Side::Player));
            let view = BattleView::new(&game);
            let unit = game.unit(target_id).unwrap();
            let score = Assessment::new(&view, unit).recruitment_breakdown(&view, unit);
            assert_eq!(score.retaliation, 0.0);
            assert_eq!(score.captures, 0.0);
            scores.push(score.total);
        }
        assert!(scores[0] > 0.0);
        assert!((scores[1] - 2.0 * scores[0]).abs() < 1e-9);
    }

    #[test]
    fn recruitment_combat_and_future_healing_ignore_enemy_proximity() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units.to_mut()[target].attacks.to_mut()[0].category = Category::Ranged;
        rules.units.to_mut()[target].attacks.to_mut()[0].range = 2;
        rules.units.to_mut()[target].attacks.to_mut()[0].damage = 12;
        rules.units.to_mut()[target].healing = 4;
        rules.units.to_mut()[target].damage_modifiers.blade = -2;
        let other = rules.types["scout"];
        rules.units.to_mut()[other] = rules.units.to_mut()[enemy].clone();
        rules.units.to_mut()[other].damage_modifiers.blade = -8;
        rules.units.to_mut()[other].attacks.to_mut()[0].damage = 20;
        rules.units.to_mut()[other].attacks.to_mut()[0].count = 1;
        rules.units.to_mut()[other].attacks.to_mut()[0].damage_type = DamageType::Pierce;
        let (mut game, target, enemy) = thorn_battle(&rules, target, enemy);
        let plains = rules.terrains["plains"];
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, plains);
        }
        game.owners.fill(Some(Side::Player));
        let ally_tile = game.site("3,4");
        game.add(game.unit(target).unwrap().kind, Side::Player, ally_tile);
        let other_tile = game.site("8,5");
        let other = game.add(other, Side::Enemy, other_tile);
        let mut samples = Vec::new();
        for (first, second) in [("5,4", "8,5"), ("8,4", "5,5"), ("8,3", "8,5")] {
            for (id, site) in [(enemy, first), (other, second)] {
                let tile = game.site(site);
                let index = game.index(id).unwrap();
                game.units[index].tile = tile;
            }
            game.roster_changed();
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            samples.push(Assessment::new(&view, unit).recruitment_breakdown(&view, unit));
        }
        assert!(samples[0].support > 0.0);
        assert_eq!(samples[0].captures, 0.0);
        for sample in &samples[1..] {
            assert!((samples[0].offense - sample.offense).abs() < 1e-9);
            assert!((samples[0].effective_hp - sample.effective_hp).abs() < 1e-9);
            assert!((samples[0].support - sample.support).abs() < 1e-9);
            assert!((samples[0].total - sample.total).abs() < 1e-9);
        }
    }

    #[test]
    fn recruitment_rewards_speed_only_for_village_reach() {
        for healing in [0, 4] {
            let mut samples = Vec::new();
            for movement in [1, 6] {
                let (mut rules, target, enemy) = thorn_fixture();
                rules.units.to_mut()[target].r#move = movement;
                rules.units.to_mut()[target].healing = healing;
                let (mut game, target_id, enemy_id) = thorn_battle(&rules, target, enemy);
                let start = game.site("1,4");
                let end = game.site("8,4");
                for unit in &mut game.units {
                    unit.tile = if unit.id == target_id { start } else { end };
                }
                assert_eq!(game.unit(enemy_id).unwrap().tile, end);
                if healing > 0 {
                    let tile = game.site("6,4");
                    let ally = game.add(target, Side::Player, tile);
                    game.units.iter_mut().find(|u| u.id == ally).unwrap().hp = 50;
                }
                game.roster_changed();
                game.owners.fill(Some(Side::Player));
                let (recruit, promotion) = {
                    let view = BattleView::new(&game);
                    let unit = game.unit(target_id).unwrap();
                    let assessment = Assessment::new(&view, unit);
                    (
                        assessment.recruitment_breakdown(&view, unit),
                        assessment.score(&view, unit),
                    )
                };
                assert_eq!(recruit.mobility, 0.0);
                assert_eq!(recruit.access, 1.0);
                assert_eq!(recruit.captures, 0.0);
                let village = (0..game.terrain.len())
                    .find(|&tile| {
                        game.terrain_is(tile, Terrain::Village)
                            && game.board.distance(start, tile) > 2
                    })
                    .unwrap();
                game.owners[village] = None;
                let view = BattleView::new(&game);
                let unit = game.unit(target_id).unwrap();
                let capture = Assessment::new(&view, unit).recruitment_breakdown(&view, unit);
                samples.push((recruit, promotion, capture));
            }
            assert!((samples[0].0.total - samples[1].0.total).abs() < 1e-9);
            assert!((samples[1].1 - samples[0].1).abs() < 1e-9);
            if healing == 0 {
                assert!(samples[1].2.captures > samples[0].2.captures);
                assert!(samples[1].2.total > samples[0].2.total);
            } else {
                assert!(samples[0].0.support > 0.0);
                assert_eq!(samples[0].0.support, samples[1].0.support);
            }
        }
    }

    #[test]
    fn intrinsic_thorns_use_pressure_without_also_counting_as_weapon_retaliation() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units.to_mut()[target].thorns = 3;
        let (game, target, _) = thorn_battle(&rules, target, enemy);
        let unit = game.unit(target).unwrap();
        let view = BattleView::new(&game);
        let score = Assessment::new(&view, unit).breakdown(&view, unit);
        assert!((score.thorns - 9.0).abs() < 1e-9);
        assert!((score.retaliation - 2.0).abs() < 1e-9);
        let mut recruit = unit.clone();
        recruit.id = 0;
        // A new recruit joins the existing army; it is not omitted from F.
        let score = Assessment::new(&view, &recruit).breakdown(&view, &recruit);
        assert!((score.thorns - 4.5).abs() < 1e-9);
    }

    #[test]
    fn expected_coverage_does_not_sum_exclusive_future_positions() {
        assert_eq!(support_coverage([].into_iter(), 1.0), 0.0);
        assert_eq!(support_coverage([4.0].into_iter(), 1.0), 4.0);
        assert_eq!(support_coverage([4.0; 2].into_iter(), 1.0), 4.0);
        assert_eq!(support_coverage([4.0; 8].into_iter(), 1.0), 4.0);
    }

    #[test]
    fn overlapping_providers_have_diminishing_value_scaled_by_army_size() {
        // Six possible recipients: each provider covers a given ally with p=.25.
        let first = expected_marginal_support(12.0, 0, 4, &mut [], 0.25);
        let second = expected_marginal_support(12.0, 0, 4, &mut [4], 0.25);
        let third = expected_marginal_support(12.0, 0, 4, &mut [4, 4], 0.25);
        assert_eq!((first, second, third), (4.0, 3.0, 2.25));
        // A larger company gives a second provider more uncovered recipients.
        assert!(expected_marginal_support(12.0, 0, 4, &mut [4], 0.1) > second);
    }

    #[test]
    fn overlap_credits_strength_upgrades_and_respects_actual_coverage() {
        assert_eq!(expected_marginal_support(12.0, 3, 3, &mut [], 0.25), 0.0);
        assert_eq!(expected_marginal_support(12.0, 3, 5, &mut [], 0.25), 2.0);
        assert_eq!(expected_marginal_support(12.0, 0, 5, &mut [3], 0.5), 3.5);
        assert_eq!(expected_marginal_support(12.0, 0, 3, &mut [5], 1.0), 0.0);
        assert_eq!(expected_marginal_support(4.0, 0, 5, &mut [3], 1.0), 1.0);
        // Multiple strengths: highest covering provider determines the increment.
        assert_eq!(
            expected_marginal_support(12.0, 0, 5, &mut [3, 5], 0.5),
            1.75
        );
    }

    #[test]
    fn strongest_only_healing_credits_only_the_increment() {
        assert_eq!(marginal_healing(12.0, 3, 3), 0.0);
        assert_eq!(marginal_healing(12.0, 3, 5), 2.0);
        assert_eq!(marginal_healing(4.0, 3, 5), 1.0);
        assert_eq!(marginal_healing(4.0, 5, 3), 0.0);
    }
}
