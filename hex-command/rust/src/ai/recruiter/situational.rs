//! Adaptive recruitment using engine matchups and terrain routes, without role quotas.
use super::{Recruiter, RecruiterKind};
use crate::ai::{context::BattleView, orders::RecruitmentOrders};
use crate::data::*;
use crate::logic::Action as Command;
#[cfg(test)]
use crate::reuse::vec;

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
    let leader = game
        .units()
        .iter()
        .find(|u| u.side == side && game.rules().units[u.kind].leader)?;
    let required = orders.requirements.iter().find(|r| r.deficit(game) > 0);
    let reserved = &orders.reserved;
    let gold = game.gold()[side.index()];
    let income = game.income(side);
    let mut best: Option<(f64, usize, usize)> = None;
    for kind in game.recruitment_options(side) {
        if required.is_some_and(|r| !r.accepts(game, kind)) {
            continue;
        }
        let rule = &game.rules().units[kind];
        if rule.cost > gold && income == 0 {
            continue;
        }
        let tiles: smallvec::SmallVec<[_; 8]> = game
            .placement_tiles(leader, kind, false)
            .into_iter()
            .filter(|t| !reserved.iter().any(|r| game.site(&r.0) == *t))
            .collect();
        let Some(&tile) = tiles.first() else {
            continue;
        };
        let mut recruit = Unit {
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
                u.side == side
                    && !game.rules().units[u.kind].leader
                    && game.rules().units[u.kind].family == rule.family
            })
            .count();
        // An unaffordable unit must justify the income spent waiting rather
        // than fielding an available alternative now.
        let wait = f64::from((rule.cost - gold).max(0)) / f64::from(income.max(1));
        for tile in tiles {
            recruit.tile = tile;
            let score = assessment.recruitment_breakdown(game, &recruit).total
                / rule.cost.max(1) as f64
                / (1.0 + copies as f64 * weights.duplicate_penalty)
                / (1.0 + wait * weights.wait_penalty)
                + super::random(
                    seed,
                    format_args!("situational:{}:{}:{tile}", game.round(), rule.id),
                ) * 0.0005;
            if best
                .as_ref()
                .is_none_or(|current| score.total_cmp(&current.0).is_ge())
            {
                best = Some((score, kind, tile));
            }
        }
    }
    let (_, kind, tile) = best?;
    if game.rules().units[kind].cost > game.gold()[side.index()] {
        return None;
    }
    Some(crate::ai::recruit_command(game, leader.id, kind, tile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Game;
    use crate::reuse::vec;

    // Controlled alternatives test the decision, independent of authored balance.
    fn fixture() -> (Rules, usize, usize) {
        let mut rules = Rules::load().unwrap();
        let a = rules.types["swordsman"];
        let b = rules.types["archer"];
        let mut alternative = rules.units[a].clone();
        alternative.hp = 40;
        alternative.cost = 10;
        alternative.healing = 0;
        alternative.attacks.to_mut().truncate(1);
        alternative.attacks.to_mut()[0].damage = 10;
        alternative.attacks.to_mut()[0].count = 1;
        alternative.attacks.to_mut()[0].category = Category::Melee;
        alternative.attacks.to_mut()[0].range = 1;
        alternative.damage_modifiers = DamageModifiers {
            blade: 0,
            pierce: 0,
            impact: 0,
        };
        for environment in alternative.environment.to_mut().iter_mut() {
            *environment = [[0; 3]; 2];
        }
        alternative.id = rules.units[a].id.clone();
        rules.units.to_mut()[a] = alternative.clone();
        alternative.id = rules.units[b].id.clone();
        rules.units.to_mut()[b] = alternative;
        rules.route_classes.to_mut()[a] = a;
        rules.route_classes.to_mut()[b] = b;
        (rules, a, b)
    }

    fn battle(rules: &Rules, a: usize, b: usize) -> Game<'_> {
        let mut game = Game::new(rules, "mirror");
        game.units.retain(|u| rules.units[u.kind].leader);
        game.roster_changed();
        game.pools = Some(std::rc::Rc::new([std::vec![a, b], std::vec![a, b]]));
        game.gold[0] = 10;
        game
    }

    fn selected(game: &Game<'_>, orders: &RecruitmentOrders) -> Option<usize> {
        choose(&BattleView::new(game), 7, orders).map(|command| {
            let Command::Recruit { kind, .. } = command else {
                panic!("recruiter emitted a non-recruit command");
            };
            kind
        })
    }

    #[test]
    fn one_strike_is_preferred_to_equal_damage_multiple_strikes_against_thorns() {
        let (mut rules, a, b) = fixture();
        rules.units.to_mut()[a].attacks.to_mut()[0].damage = 6;
        rules.units.to_mut()[a].attacks.to_mut()[0].count = 3;
        rules.units.to_mut()[b].attacks.to_mut()[0].damage = 18;
        let enemy = battle(&rules, a, b)
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        rules.units.to_mut()[enemy].hp = 100;
        rules.units.to_mut()[enemy].thorns = 4;
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
                rules.units.to_mut()[b].damage_modifiers.blade = -6;
            } else {
                rules.units.to_mut()[b].thorns = 12;
            }
            let enemy = battle(&rules, a, b)
                .units
                .iter()
                .find(|u| u.side == Side::Enemy)
                .unwrap()
                .kind;
            rules.units.to_mut()[enemy].attacks.to_mut().truncate(1);
            rules.units.to_mut()[enemy].attacks.to_mut()[0].category = Category::Melee;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].range = 1;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].damage_type = DamageType::Blade;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].damage = 8;
            rules.units.to_mut()[enemy].attacks.to_mut()[0].count = 3;
            assert_eq!(
                selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
                Some(b)
            );
        }
    }

    #[test]
    fn valuable_expensive_recruit_can_wait_for_income() {
        let (mut rules, a, b) = fixture();
        rules.units.to_mut()[b].cost = 12;
        rules.units.to_mut()[b].hp = 80;
        rules.units.to_mut()[b].attacks.to_mut()[0].damage = 20;
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain
            .iter()
            .enumerate()
            .find(|(_, t)| **t == rules.terrains["village"])
            .unwrap()
            .0;
        game.owners[village] = Some(Side::Player);
        game.gold[0] = 10;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), None);
        game.round += 1;
        game.gold[0] += game.income(Side::Player);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
        game.gold[0] = 10;
        game.owners[village] = None;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
    }

    #[test]
    fn modest_upgrade_does_not_delay_an_affordable_recruit() {
        let (mut rules, a, b) = fixture();
        rules.units.to_mut()[b].cost = 12;
        rules.units.to_mut()[b].hp = 54;
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain
            .iter()
            .enumerate()
            .find(|(_, t)| **t == rules.terrains["village"])
            .unwrap()
            .0;
        game.owners[village] = Some(Side::Player);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
        game.gold[0] = 12;
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
    }

    #[test]
    fn movement_and_armor_per_strike_affect_matchups() {
        let (mut rules, a, b) = fixture();
        rules.units.to_mut()[b].r#move = rules.units.to_mut()[a].r#move * 2;
        assert_eq!(
            selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
            Some(b)
        );
        rules.units.to_mut()[b].r#move = rules.units.to_mut()[a].r#move;
        rules.units.to_mut()[a].attacks.to_mut()[0].damage = 6;
        rules.units.to_mut()[a].attacks.to_mut()[0].count = 3;
        rules.units.to_mut()[b].attacks.to_mut()[0].damage = 18;
        let enemy = battle(&rules, a, b)
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        let damage_type = rules.units[a].attacks[0].damage_type;
        rules.units.to_mut()[b].attacks.to_mut()[0].damage_type = damage_type;
        rules.units.to_mut()[enemy].damage_modifiers = DamageModifiers {
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
        rules.units.to_mut()[a].attacks.to_mut()[0].damage_type = DamageType::Blade;
        rules.units.to_mut()[b].attacks.to_mut()[0].damage_type = DamageType::Pierce;
        let enemy = battle(&rules, a, b)
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        for (blade, pierce, expected) in [(-10, 0, b), (0, -10, a)] {
            rules.units.to_mut()[enemy].damage_modifiers.blade = blade;
            rules.units.to_mut()[enemy].damage_modifiers.pierce = pierce;
            for environment in rules.units.to_mut()[enemy].environment.to_mut().iter_mut() {
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
        rules.units.to_mut()[a].costs.to_mut()[water] = None;
        rules.units.to_mut()[b].costs.to_mut()[water] = Some(1);
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
                selector: crate::ai::orders::UnitSelector::Type(rules.units[a].id.as_ref().into()),
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
        rules.units.to_mut()[b].healing = 20;
        rules.units.to_mut()[b].attacks.to_mut()[0].damage = 0;
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
