//! Adaptive recruitment using engine matchups and terrain routes, without role quotas.
use super::{Recruiter, RecruiterKind};
use crate::ai::{context::BattleView, orders::RecruitmentOrders};
use crate::data::*;

pub(super) struct Situational;
impl Recruiter for Situational {
    fn kind(&self) -> Option<super::RecruiterKind> {
        Some(super::RecruiterKind::Situational)
    }
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
    choose_with_forecast(game, seed, orders, false)
}

pub(super) fn choose_with_forecast(
    game: &BattleView<'_, '_>,
    seed: u32,
    orders: &RecruitmentOrders,
    predict: bool,
) -> Option<Command> {
    choose_with_weights(
        game,
        seed,
        orders,
        predict,
        game.rules().recruitment.standard,
    )
}

pub(super) fn choose_with_weights(
    game: &BattleView<'_, '_>,
    seed: u32,
    orders: &RecruitmentOrders,
    predict: bool,
    weights: RecruitmentWeights,
) -> Option<Command> {
    let side = game.turn();
    let required = orders.requirements.iter().find(|r| r.deficit(game) > 0);
    let reserved: rustc_hash::FxHashSet<_> =
        orders.reserved.iter().map(|t| game.site(&t.0)).collect();
    let mut choices = Vec::new();
    for leader in game
        .units()
        .iter()
        .filter(|u| game.belongs_to_phase(u, side) && u.is_leader())
    {
        for kind in game.recruitment_options_for(leader) {
            if required.is_some_and(|r| !r.accepts(game, kind)) {
                continue;
            }
            let tiles: Vec<_> = game
                .placement_tiles(leader, kind, false)
                .into_iter()
                .filter(|t| !reserved.contains(t))
                .collect();
            let Some(&tile) = tiles.first() else {
                continue;
            };
            let rule = &game.rules().units[kind];
            let price = game.recruit_cost(leader, kind);
            let mut recruit = Unit {
                faction: leader.faction.clone(),
                items: vec![],
                ai_group: None,
                id: 0,
                kind,
                side,
                tile,
                hp: rule.hp,
                moved: true,
                attacked: true,
                experience: None,
                name: None,
                role: None,
                retreating: None,
                rebuilding: None,
            };
            let assessment =
                super::evaluation::Assessment::with_weights(game, &recruit, predict, weights);
            let copies = game
                .units()
                .iter()
                .filter(|u| {
                    game.belongs_to_phase(u, side)
                        && !u.is_leader()
                        && game.rules().units[u.kind].family == rule.family
                })
                .count();
            if price > game.gold_for_unit(leader) && game.income(side) == 0 {
                continue;
            }
            // An unaffordable unit must justify the income spent waiting rather
            // than fielding an available alternative now.
            let wait = f64::from((price - game.gold_for_unit(leader)).max(0))
                / f64::from(game.income(side).max(1));
            for tile in tiles {
                recruit.tile = tile;
                let score = assessment.recruitment_breakdown(game, &recruit).total
                    / price.max(1) as f64
                    / (1.0 + copies as f64 * weights.duplicate_penalty)
                    / (1.0 + wait * weights.wait_penalty)
                    + super::random(
                        seed,
                        &format!("situational:{}:{}:{tile}", game.round(), rule.id),
                    ) * 0.0005;
                choices.push((score, leader, kind, tile));
            }
        }
    }
    let (_, leader, kind, tile) = choices.into_iter().max_by(|a, b| a.0.total_cmp(&b.0))?;
    if game.recruit_cost(leader, kind) > game.gold_for_unit(leader) {
        return None;
    }
    Some(crate::ai::recruit_command(game, leader.id, kind, tile))
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
        game.units.retain(|u| u.is_leader());
        game.roster_changed();
        game.pools = Some([vec![a, b], vec![a, b]]);
        game.gold["player"] = 10;
        game
    }

    fn selected(game: &Game<'_>, orders: &RecruitmentOrders) -> Option<usize> {
        choose(&BattleView::new(game), 7, orders).map(|command| {
            let Command::Recruit { kind, .. } = command else {
                panic!("recruiter emitted a non-recruit command");
            };
            game.kind(&kind.0)
        })
    }

    #[test]
    fn one_strike_is_preferred_to_equal_damage_multiple_strikes_against_thorns() {
        let (mut rules, a, b) = fixture();
        rules.units[a].attacks[0].damage = 6;
        rules.units[a].attacks[0].count = 3;
        rules.units[b].attacks[0].damage = 18;
        let enemy = battle(&rules, a, b)
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        rules.units[enemy].hp = 100;
        rules.units[enemy].thorns = 4;
        assert_eq!(
            selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
            Some(b)
        );
    }

    #[test]
    fn defense_and_own_thorns_change_the_selected_recruit() {
        for armor in [false, true] {
            let (mut rules, a, b) = fixture();
            if armor {
                rules.units[b].damage_modifiers.blade = -6;
            } else {
                rules.units[b].thorns = 12;
            }
            let enemy = battle(&rules, a, b)
                .units
                .iter()
                .find(|u| u.side == Side::Enemy)
                .unwrap()
                .kind;
            rules.units[enemy].attacks.truncate(1);
            rules.units[enemy].attacks[0].category = Category::Melee;
            rules.units[enemy].attacks[0].range = 1;
            rules.units[enemy].attacks[0].damage_type = DamageType::Blade;
            rules.units[enemy].attacks[0].damage = 8;
            rules.units[enemy].attacks[0].count = 3;
            assert_eq!(
                selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
                Some(b)
            );
        }
    }

    #[test]
    fn valuable_expensive_recruit_can_wait_for_income() {
        let (mut rules, a, b) = fixture();
        rules.units[b].cost = 12;
        rules.units[b].hp = 80;
        rules.units[b].attacks[0].damage = 20;
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain
            .iter()
            .enumerate()
            .find(|(_, t)| **t == rules.terrains["village"])
            .unwrap()
            .0;
        game.owners[village] = Some("player".into());
        game.gold["player"] = 10;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), None);
        game.round += 1;
        game.gold["player"] += game.income(Side::Player);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
        game.gold["player"] = 10;
        game.owners[village] = None;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
    }

    #[test]
    fn modest_upgrade_does_not_delay_an_affordable_recruit() {
        let (mut rules, a, b) = fixture();
        rules.units[b].cost = 12;
        rules.units[b].hp = 54;
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain
            .iter()
            .enumerate()
            .find(|(_, t)| **t == rules.terrains["village"])
            .unwrap()
            .0;
        game.owners[village] = Some("player".into());
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
        game.gold["player"] = 12;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
    }

    #[test]
    fn movement_and_armor_per_strike_affect_matchups() {
        let (mut rules, a, b) = fixture();
        rules.units[b].r#move = rules.units[a].r#move * 2;
        assert_eq!(
            selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
            Some(b)
        );
        rules.units[b].r#move = rules.units[a].r#move;
        rules.units[a].attacks[0].damage = 6;
        rules.units[a].attacks[0].count = 3;
        rules.units[b].attacks[0].damage = 18;
        let enemy = battle(&rules, a, b)
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        let damage_type = rules.units[a].attacks[0].damage_type;
        rules.units[b].attacks[0].damage_type = damage_type;
        rules.units[enemy].damage_modifiers = DamageModifiers {
            blade: -4,
            pierce: -4,
            impact: -4,
        };
        assert_eq!(
            selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
            Some(b)
        );
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
        game.gold["player"] = 0;
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
        let mut healer = leader.clone();
        healer.id = 0;
        healer.kind = b;
        healer.hp = rules.units[b].hp;
        healer.tile = game.placement_tiles(&leader, b, true)[0];
        let assess = |game: &Game<'_>| {
            let view = BattleView::new(game);
            crate::ai::recruiter::evaluation::Assessment::new(&view, &healer)
                .recruitment_breakdown(&view, &healer)
        };
        let healthy = assess(&game);
        let index = game.index(id).unwrap();
        game.units[index].hp = 1;
        let wounded = assess(&game);
        // Wounds create healing value; they do not guarantee that buying a
        // healer beats every alternative under every combat valuation.
        assert!(healthy.support > 0.0);
        assert!(wounded.support > healthy.support);
        assert!(wounded.total > healthy.total);
    }
}
