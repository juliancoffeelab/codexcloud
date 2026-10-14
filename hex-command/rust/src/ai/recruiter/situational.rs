//! Adaptive recruitment using engine matchups and terrain routes, without role quotas.
use super::{Recruiter, RecruiterKind};
use crate::ai::{context::BattleView, orders::RecruitmentOrders};
use crate::data::*;

pub(super) struct Situational;
impl Recruiter for Situational {
    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, CommandError> {
        super::drive(RecruiterKind::Situational, context, orders)
    }
}

pub(super) fn choose(
    game: &BattleView<'_, '_>,
    seed: u32,
    orders: &RecruitmentOrders,
) -> Option<Command> {
    let side = game.turn();
    let leader = game
        .units()
        .iter()
        .find(|u| u.side == side && game.rules().units[u.kind].leader)?;
    let required = orders.requirements.iter().find(|r| r.deficit(game) > 0);
    let reserved: rustc_hash::FxHashSet<_> =
        orders.reserved.iter().map(|t| game.site(&t.0)).collect();
    let enemies: Vec<_> = game.units().iter().filter(|u| u.side != side).collect();
    let allies: Vec<_> = game.units().iter().filter(|u| u.side == side).collect();
    let mut best: Option<(f64, usize, usize)> = None;
    for kind in game.recruitment_options(side) {
        if required.is_some_and(|r| !r.accepts(game, kind)) {
            continue;
        }
        let tiles: Vec<_> = game
            .placement_tiles(leader, kind, true)
            .into_iter()
            .filter(|t| !reserved.contains(t))
            .collect();
        if tiles.is_empty() {
            continue;
        }
        let rule = &game.rules().units[kind];
        let mut recruit = Unit {
            id: 0,
            kind,
            side,
            tile: tiles[0],
            hp: rule.hp,
            moved: true,
            attacked: true,
            experience: None,
            name: None,
            role: None,
            retreating: None,
            rebuilding: None,
        };
        // Route to legal attack origins, not the enemy's occupied tile. Ranged
        // recruits can therefore help across water even when they cannot enter it.
        let matchups: Vec<_> = enemies
            .iter()
            .map(|target| {
                rule.attacks
                    .iter()
                    .enumerate()
                    .map(|(weapon, attack)| {
                        let origins = game.attack_origins(attack.category, attack.range);
                        let mut goals = Vec::new();
                        origins[target.tile].for_each(|tile| {
                            if game.cost(kind, tile) != u16::MAX {
                                goals.push(tile);
                            }
                        });
                        (
                            game.damage(&recruit, target, weapon).min(target.hp) as f64,
                            game.routes(kind, &goals),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let copies = allies
            .iter()
            .filter(|u| {
                !game.rules().units[u.kind].leader
                    && game.rules().units[u.kind].family == rule.family
            })
            .count();
        let healing_needs: Vec<_> = if rule.healing > 0 {
            allies
                .iter()
                .map(|ally| {
                    let r = &game.rules().units[ally.kind];
                    let covered = allies
                        .iter()
                        .filter(|u| {
                            u.id != ally.id && game.board().distance(u.tile, ally.tile) <= 1
                        })
                        .map(|u| game.rules().units[u.kind].healing)
                        .max()
                        .unwrap_or(0);
                    let exposed = enemies.iter().any(|e| {
                        game.board().distance(ally.tile, e.tile)
                            <= game.rules().units[e.kind].r#move as u16 + 1
                    });
                    let demand =
                        (r.hp - ally.hp) as f64 + if exposed { r.hp as f64 * 0.2 } else { 0.0 };
                    let goals: Vec<_> = game.board().adjacent[ally.tile]
                        .iter()
                        .copied()
                        .filter(|&t| game.cost(kind, t) != u16::MAX)
                        .collect();
                    (
                        (demand - covered as f64).max(0.0).min(rule.healing as f64),
                        game.routes(kind, &goals),
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        for tile in tiles {
            recruit.tile = tile;
            let (offense, access) = if enemies.is_empty() {
                (
                    rule.attacks
                        .iter()
                        .map(|a| (a.damage * i32::from(a.count)) as f64)
                        .fold(0.0, f64::max),
                    1.0,
                )
            } else {
                let mut offense = 0.0;
                let mut access = 0.0;
                for weapons in &matchups {
                    let mut value: f64 = 0.0;
                    let mut reach: f64 = 0.0;
                    for (damage, routes) in weapons {
                        if routes[tile] == u16::MAX {
                            continue;
                        }
                        let turns = routes[tile] as f64 / rule.r#move.max(1) as f64;
                        let weight = 1.0 / (1.0 + turns * 0.35);
                        value = value.max(damage * weight);
                        reach = reach.max(weight);
                    }
                    offense += value;
                    access += reach;
                }
                (
                    offense / enemies.len() as f64,
                    access / enemies.len() as f64,
                )
            };
            // Resistance and spawn terrain use the same damage calculation as play.
            let incoming = enemies
                .iter()
                .map(|enemy| {
                    let r = &game.rules().units[enemy.kind];
                    let distance = game.board().distance(tile, enemy.tile) as f64;
                    let weight = 1.0 / (1.0 + distance / r.r#move.max(1) as f64);
                    (0..r.attacks.len())
                        .map(|w| game.damage(enemy, &recruit, w) as f64)
                        .fold(0.0, f64::max)
                        * weight
                })
                .sum::<f64>()
                / enemies.len().max(1) as f64;
            let durability = rule.hp as f64 / (1.0 + incoming / rule.hp.max(1) as f64);
            // Healing demand is local and falls when existing healers cover it.
            let healing = healing_needs
                .iter()
                .filter(|(_, routes)| {
                    routes[tile] != u16::MAX && routes[tile] as usize <= rule.r#move
                })
                .map(|(demand, _)| demand)
                .sum::<f64>();
            let score = (offense * 1.5 + durability * access * 0.35 + healing * 2.0)
                / rule.cost.max(1) as f64
                / (1.0 + copies as f64 * 0.2)
                + super::jitter(
                    seed,
                    &format!("situational:{}:{}:{tile}", game.round(), rule.id),
                ) * 0.01;
            if best.is_none_or(|(previous, _, _)| score > previous) {
                best = Some((score, kind, tile));
            }
        }
    }
    best.map(|(_, kind, tile)| crate::ai::recruit_command(game, leader.id, kind, tile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Game;

    // Controlled alternatives test the decision, independent of authored balance.
    fn fixture() -> (Rules, usize, usize) {
        let mut rules = Rules::load().unwrap();
        let a = rules.types["swordsman"];
        let b = rules.types["archer"];
        let mut alternative = rules.units[a].clone();
        alternative.hp = 40;
        alternative.cost = 10;
        alternative.healing = 0;
        alternative.attacks.truncate(1);
        alternative.attacks[0].damage = 10;
        alternative.attacks[0].count = 1;
        alternative.attacks[0].category = Category::Melee;
        alternative.attacks[0].range = 1;
        alternative.damage_modifiers = DamageModifiers {
            blade: 0,
            pierce: 0,
            impact: 0,
        };
        for environment in &mut alternative.environment {
            *environment = [[0; 3]; 2];
        }
        alternative.id = rules.units[a].id.clone();
        rules.units[a] = alternative.clone();
        alternative.id = rules.units[b].id.clone();
        rules.units[b] = alternative;
        rules.route_classes[a] = a;
        rules.route_classes[b] = b;
        (rules, a, b)
    }

    fn battle(rules: &Rules, a: usize, b: usize) -> Game<'_> {
        let mut game = Game::new(rules, "mirror");
        game.units.retain(|u| rules.units[u.kind].leader);
        game.roster_changed();
        game.pools = Some([vec![a, b], vec![a, b]]);
        game.gold[0] = 10;
        game
    }

    fn selected(game: &Game<'_>, orders: &RecruitmentOrders) -> Option<usize> {
        choose(&BattleView::new(game), 7, orders).map(|command| match command {
            Command::Recruit { kind, .. } => game.kind(&kind.0),
            _ => panic!("recruiter emitted a non-recruit command"),
        })
    }

    #[test]
    fn enemy_resistance_changes_selection() {
        let (mut rules, a, b) = fixture();
        rules.units[a].attacks[0].damage_type = DamageType::Blade;
        rules.units[b].attacks[0].damage_type = DamageType::Pierce;
        let enemy = battle(&rules, a, b)
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        for (blade, pierce, expected) in [(-10, 0, b), (0, -10, a)] {
            rules.units[enemy].damage_modifiers.blade = blade;
            rules.units[enemy].damage_modifiers.pierce = pierce;
            for environment in &mut rules.units[enemy].environment {
                *environment = [[0; 3]; 2];
            }
            assert_eq!(
                selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
                Some(expected)
            );
        }
    }

    #[test]
    fn terrain_access_and_campaign_requirements_override_free_selection() {
        let (mut rules, a, b) = fixture();
        let water = rules.terrains["water"];
        rules.units[a].costs[water] = None;
        rules.units[b].costs[water] = Some(1);
        let mut game = battle(&rules, a, b);
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .clone();
        let enemy = game
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .clone();
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, water);
        }
        game.set_terrain(leader.tile, rules.terrains["keep"]);
        game.set_terrain(enemy.tile, rules.terrains["keep"]);
        for tile in game.board.adjacent[leader.tile].clone() {
            game.set_terrain(tile, rules.terrains["plains"]);
        }
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
        let mut orders = RecruitmentOrders {
            requirements: vec![crate::ai::orders::RecruitRequirement::EnsureUnits {
                selector: crate::ai::orders::UnitSelector::Type(rules.units[a].id.as_str().into()),
                count: 1,
            }],
            ..Default::default()
        };
        assert_eq!(selected(&game, &orders), Some(a));
        orders.reserved = game
            .placement_tiles(&leader, a, true)
            .iter()
            .map(|&t| game.board.ids[t].as_str().into())
            .collect();
        assert_eq!(selected(&game, &orders), None);
        orders.reserved.clear();
        game.gold[0] = 0;
        assert_eq!(selected(&game, &orders), None);
    }

    #[test]
    fn wounded_company_creates_healing_demand() {
        let (mut rules, a, b) = fixture();
        rules.units[b].healing = 20;
        rules.units[b].attacks[0].damage = 0;
        let mut game = battle(&rules, a, b);
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, a, true)[0];
        let id = game.add(a, Side::Player, tile);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
        let index = game.index(id).unwrap();
        game.units[index].hp = 1;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
    }
}
