//! Shared mechanical valuation for recruits and free, earned promotions.
use crate::{ai::context::BattleView, data::*};
use std::rc::Rc;

struct Approach {
    enemy: u32,
    weapon: usize,
    origins: Vec<usize>,
    routes: Rc<Vec<u16>>,
}
// Recruitment estimates typical coverage, rather than summing mutually
// exclusive future positions. This is a prior, not a measured runtime statistic.
const EXPECTED_SUPPORT_RECIPIENTS: f64 = 1.0;

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
    pub total: f64,
}

pub(super) struct Assessment {
    approaches: Vec<Approach>,
    support: Vec<Support>,
    captures: Vec<(Rc<Vec<u16>>, f64)>,
    friendly_count: usize,
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
        // An existing provider cannot support itself. Treat remote providers'
        // future coverage as independent; current adjacent coverage is exact.
        let overlap_probability =
            (EXPECTED_SUPPORT_RECIPIENTS / allies.len().saturating_sub(1).max(1) as f64).min(1.0);
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
                let healing = expected_marginal_support(
                    demand,
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
                    overlap_probability,
                );
                // Classify the enemy's preferred exchange against this ally,
                // including the proposed aura, rather than any melee backup.
                let thorns = extra_aura
                    * expected_thorn_triggers(
                        enemies.iter().filter_map(|enemy| {
                            dominant_melee_strikes(
                                game,
                                enemy,
                                ally,
                                Some(aura.max(rule.thorn_aura)),
                            )
                        }),
                        allies.len() + 1,
                    );
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
            friendly_count: allies.len() + 1,
            mobility,
        }
    }
    pub(super) fn score(&self, game: &BattleView<'_, '_>, unit: &Unit) -> f64 {
        self.breakdown(game, unit).total
    }
    pub(super) fn breakdown(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Score {
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
            if let Some(preferred) = preferred_attack(game, enemy, unit, None) {
                let attack = &enemy_rule.attacks[preferred.weapon];
                incoming += f64::from(preferred.damage) * weight;
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
                // Own reflection is valued separately by army pressure.
                // Preserve ordinary retaliation and existing friendly aura.
                let response = preferred.response
                    - if attack.category == Category::Melee {
                        rule.thorns * i32::from(attack.count)
                    } else {
                        0
                    };
                retaliation += f64::from(response.min(enemy.hp)) * weight;
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
        let support = expected_support_coverage(self.support.iter().filter_map(|s| {
            (s.routes[unit.tile] != u16::MAX && s.routes[unit.tile] as usize <= rule.r#move)
                .then_some(s.healing + s.thorns)
        }));
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
            f64::from(rule.thorns)
                * expected_thorn_triggers(
                    enemies
                        .iter()
                        .filter_map(|enemy| dominant_melee_strikes(game, enemy, unit, None)),
                    self.friendly_count,
                )
        } else {
            0.0
        };
        let total = (offense * 1.5 + durability * access * 0.35 + retaliation * access * 0.5)
            * (1.0 + self.mobility * 0.05 + captures * 0.1)
            + (support + thorns) * 2.0;
        Score {
            offense,
            effective_hp: durability,
            access,
            retaliation,
            support,
            thorns,
            mobility: self.mobility,
            captures,
            total,
        }
    }
}

// Each zero is an enemy whose preferred attack is ranged. Exclude it from
// melee pressure and the melee strike average. Cap attackers, not strikes.
fn expected_thorn_triggers(strikes: impl Iterator<Item = u8>, friendly_count: usize) -> f64 {
    let (enemies, strikes) = strikes
        .filter(|&count| count > 0)
        .fold((0usize, 0usize), |(enemies, strikes), count| {
            (enemies + 1, strikes + usize::from(count))
        });
    if enemies == 0 {
        0.0
    } else {
        (enemies as f64 / friendly_count.max(1) as f64).min(6.0) * strikes as f64 / enemies as f64
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
) -> Option<u8> {
    preferred_attack(game, enemy, target, aura).map(|preferred| {
        let attack = &game.rules().units[enemy.kind].attacks[preferred.weapon];
        if attack.category == Category::Melee {
            attack.count
        } else {
            0
        }
    })
}

struct PreferredAttack {
    weapon: usize,
    damage: i32,
    response: i32,
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
        let mut origin = None;
        origins[target.tile].for_each(|tile| {
            if game.cost(enemy.kind, tile) != u16::MAX
                && origin.is_none_or(|old| {
                    game.board().distance(enemy.tile, tile) < game.board().distance(enemy.tile, old)
                })
            {
                origin = Some(tile);
            }
        });
        let Some(origin) = origin else { continue };
        let mut attacker = enemy.clone();
        attacker.tile = origin;
        let (damage, mut response, _) = game.preview(&attacker, target, weapon, origin);
        if let Some(aura) = aura {
            let reflection = if attack.category == Category::Melee {
                (game.rules().units[target.kind].thorns + aura) * i32::from(attack.count)
            } else {
                0
            };
            response += reflection - game.thorn_damage(&attacker, target, weapon, origin);
        }
        let healed = game.drain_healing(&attacker, target, weapon, damage);
        let value = damage as f64 + healed as f64 + if damage >= target.hp { 18.0 } else { 0.0 }
            - response as f64 * 0.7
            - if response >= attacker.hp + healed {
                40.0
            } else {
                0.0
            };
        if best.as_ref().is_none_or(|(score, _)| value > *score) {
            best = Some((
                value,
                PreferredAttack {
                    weapon,
                    damage,
                    response,
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

fn expected_support_coverage(values: impl Iterator<Item = f64>) -> f64 {
    let (sum, count) = values.fold((0.0, 0usize), |(sum, count), value| {
        (sum + value, count + 1)
    });
    if count == 0 {
        0.0
    } else {
        sum * EXPECTED_SUPPORT_RECIPIENTS.min(count as f64) / count as f64
    }
}

fn marginal_healing(demand: f64, covered: i32, candidate: i32) -> f64 {
    (demand.min(f64::from(candidate)) - demand.min(f64::from(covered))).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thorn_pressure_caps_attackers_before_strikes_and_ignores_ranged_enemies() {
        assert_eq!(expected_thorn_triggers([3; 20].into_iter(), 1), 18.0);
        assert_eq!(expected_thorn_triggers([1].into_iter(), 10), 0.1);
        assert_eq!(expected_thorn_triggers([0; 20].into_iter(), 1), 0.0);
        assert_eq!(expected_thorn_triggers([0, 0, 1, 3].into_iter(), 4), 1.0);
        assert_eq!(expected_thorn_triggers([].into_iter(), 1), 0.0);
    }

    fn thorn_fixture() -> (Rules, usize, usize) {
        let mut rules = Rules::load().unwrap();
        let target = rules.types["swordsman"];
        let enemy = rules.types["archer"];
        for kind in [target, enemy] {
            let rule = &mut rules.units[kind];
            rule.hp = 100;
            rule.thorns = 0;
            rule.thorn_aura = 0;
            rule.healing = 0;
            rule.attacks.truncate(1);
            rule.attacks[0].category = Category::Melee;
            rule.attacks[0].range = 1;
            rule.attacks[0].damage_type = DamageType::Blade;
            rule.attacks[0].damage = 2;
            rule.attacks[0].count = 1;
            rule.attacks[0].drain = 0;
            rule.damage_modifiers = DamageModifiers {
                blade: 0,
                pierce: 0,
                impact: 0,
            };
            for environment in &mut rule.environment {
                *environment = [[0; 3]; 2];
            }
        }
        rules.units[enemy].attacks[0].damage = 8;
        rules.units[enemy].attacks[0].count = 3;
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
        rules.units[enemy].attacks.push(ranged);
        {
            let (game, target, enemy) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let target = game.unit(target).unwrap();
            let enemy = game.unit(enemy).unwrap();
            assert_eq!(dominant_melee_strikes(&view, enemy, target, None), Some(3));
            // The buff makes melee a worse exchange, so the enemy avoids it.
            assert_eq!(
                dominant_melee_strikes(&view, enemy, target, Some(2)),
                Some(0)
            );
        }
        rules.units[target].damage_modifiers.blade = -2;
        let (game, target, enemy) = thorn_battle(&rules, target, enemy);
        let view = BattleView::new(&game);
        assert_eq!(
            dominant_melee_strikes(
                &view,
                game.unit(enemy).unwrap(),
                game.unit(target).unwrap(),
                None
            ),
            Some(0)
        );
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
        rules.units[enemy].attacks.push(ranged);
        for (blade, expected_weapon, expected_hp) in [(0, 0, 100.0), (-2, 1, 120.0)] {
            rules.units[target].damage_modifiers.blade = blade;
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
        rules.units[target].hp = 40;
        rules.units[enemy].attacks[0].category = Category::Ranged;
        rules.units[enemy].attacks[0].range = 2;
        for (damage, count, armour, expected) in [
            (10, 1, 0, 40.0),
            (10, 1, -2, 50.0),
            (5, 2, -2, 40.0 * 10.0 / 6.0),
            (5, 2, -5, 200.0),
        ] {
            rules.units[enemy].attacks[0].damage = damage;
            rules.units[enemy].attacks[0].count = count;
            rules.units[target].damage_modifiers.blade = armour;
            let (game, target, _) = thorn_battle(&rules, target, enemy);
            let view = BattleView::new(&game);
            let unit = game.unit(target).unwrap();
            let score = Assessment::new(&view, unit).breakdown(&view, unit);
            assert!((score.effective_hp - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn intrinsic_thorns_use_pressure_without_also_counting_as_weapon_retaliation() {
        let (mut rules, target, enemy) = thorn_fixture();
        rules.units[target].thorns = 3;
        let (game, target, _) = thorn_battle(&rules, target, enemy);
        let unit = game.unit(target).unwrap();
        let view = BattleView::new(&game);
        let score = Assessment::new(&view, unit).breakdown(&view, unit);
        assert_eq!(score.thorns, 9.0);
        assert_eq!(score.retaliation, 2.0);
        let mut recruit = unit.clone();
        recruit.id = 0;
        // A new recruit joins the existing army; it is not omitted from F.
        let score = Assessment::new(&view, &recruit).breakdown(&view, &recruit);
        assert_eq!(score.thorns, 4.5);
    }

    #[test]
    fn expected_coverage_does_not_sum_exclusive_future_positions() {
        assert_eq!(expected_support_coverage([].into_iter()), 0.0);
        assert_eq!(expected_support_coverage([4.0].into_iter()), 4.0);
        assert_eq!(expected_support_coverage([4.0; 2].into_iter()), 4.0);
        assert_eq!(expected_support_coverage([4.0; 8].into_iter()), 4.0);
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
