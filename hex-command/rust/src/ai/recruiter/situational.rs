//! Adaptive recruitment using engine matchups and terrain routes, without role quotas.
use super::{Recruiter, RecruiterKind};
use crate::ai::{context::BattleView, orders::RecruitmentOrders};
use crate::data::*;
use hex_command_battle::RuntimeCommand as Command;

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
    let mut candidates = Vec::new();
    for leader in game
        .units()
        .iter()
        .filter(|u| game.belongs_to_faction(u, game.observer_faction()) && u.is_leader())
    {
        let gold = game.gold_for_unit(leader);
        for kind in game.recruitment_options_for(leader) {
            if required.is_some_and(|r| !r.accepts(game, kind)) {
                continue;
            }
            let tiles: hex_command_battle::model::PlacementTiles = game
                .placement_tiles(leader, kind, false)
                .into_iter()
                .filter(|t| !reserved.contains(t))
                .collect();
            if tiles.is_empty() {
                continue;
            }
            candidates.push((leader, kind, tiles, game.recruit_cost(leader, kind), gold));
        }
    }
    // Waiting can beat an affordable purchase, so retain unaffordable options
    // in the comparison. If none can be bought, every possible winner waits.
    if !candidates
        .iter()
        .any(|(_, _, _, price, gold)| price <= gold)
    {
        return None;
    }
    let income = game.income(game.observer_faction());
    let company: Vec<_> = game
        .units()
        .iter()
        .filter(|u| game.belongs_to_faction(u, game.observer_faction()) && !u.is_leader())
        .collect();
    let mut forecast_leader = None;
    let mut forecasts = None;
    let mut best: Option<(f64, &Unit, usize, usize)> = None;
    for (leader, kind, tiles, price, gold) in candidates {
        let tile = tiles[0];
        let rule = &game.rules().units[kind];
        if price > gold && income == 0 {
            continue;
        }
        let mut recruit = Unit {
            field_available: true,
            perception: PerceptionId(0),
            vision_group: leader.vision_group,
            id: 0,
            kind,
            phase: side,
            tile,
            hp: rule.hp,
            moved: true,
            attack_spent: true,
            experience: None,
            role: None,
            retreating: None,
            rebuilding: None,
            faction: leader.faction,
            ai_group: None,
            character: None,
            inventory: InventoryId::default(),
        };
        if forecast_leader != Some(leader.id) {
            forecasts = Some(std::rc::Rc::new(super::evaluation::AssessmentScene::new(
                game,
                &recruit,
                predict,
                weights.future_enemies,
            )));
            forecast_leader = Some(leader.id);
        }
        let assessment = super::evaluation::Assessment::with_prepared(
            game,
            &recruit,
            weights,
            forecasts.as_ref().unwrap().clone(),
        );
        let copies = company
            .iter()
            .filter(|u| game.rules().units[u.kind].family == rule.family)
            .count();
        // An unaffordable unit must justify the income spent waiting rather
        // than fielding an available alternative now.
        let wait = f64::from((price - gold).max(0)) / f64::from(income.max(1));
        let draw =
            crate::ai::policies::tie_breaking::RecruitmentDraw::new(seed, game.round(), &rule.id);
        for tile in tiles {
            recruit.tile = tile;
            let score = assessment.recruitment_score(game, &recruit)
                / price.max(1) as f64
                / (1.0 + copies as f64 * weights.duplicate_penalty)
                / (1.0 + wait * weights.wait_penalty)
                + draw.at_tile(tile) * 0.0005;
            if best
                .as_ref()
                .is_none_or(|previous| score.total_cmp(&previous.0).is_ge())
            {
                best = Some((score, leader, kind, tile));
            }
        }
    }
    let (_, leader, kind, tile) = best?;
    if game.recruit_cost(leader, kind) > game.gold_for_unit(leader) {
        return None;
    }
    Some(crate::ai::recruit_command(leader.id, kind, tile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::recruiter::fixtures;
    use crate::logic::Game;
    use hex_command_battle::RuntimeCommand as Command;

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
        fixtures::units(&mut game, |units| {
            units.retain(|u| u.is_leader());
        });
        game.configure_arena(Some([vec![a, b], vec![a, b]]), None);
        fixtures::edit(&mut game, |state| state.gold["player"] = 10);
        game
    }

    fn selected(game: &Game<'_>, orders: &RecruitmentOrders) -> Option<usize> {
        choose(&BattleView::new(game), 7, orders).map(|command| {
            let Command::Recruit { kind, .. } = command else {
                panic!("recruiter emitted a non-recruit command");
            };
            kind.0
        })
    }

    #[test]
    fn one_strike_is_preferred_to_equal_damage_multiple_strikes_against_thorns() {
        let (mut rules, a, b) = fixture();
        rules.units[a].attacks[0].damage = 6;
        rules.units[a].attacks[0].count = 3;
        rules.units[b].attacks[0].damage = 18;
        let enemy = battle(&rules, a, b)
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
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
                .units()
                .iter()
                .find(|u| u.phase == TurnPhase::Ai)
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
            .terrain()
            .iter()
            .enumerate()
            .find(|(_, t)| **t == rules.terrains["village"])
            .unwrap()
            .0;
        fixtures::edit(&mut game, |state| {
            state.tiles[village].owner = Some("player".into())
        });
        fixtures::edit(&mut game, |state| state.gold["player"] = 10);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), None);
        fixtures::edit(&mut game, |state| state.round += 1);
        let income = game.income("player");
        fixtures::edit(&mut game, |state| state.gold["player"] += income);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
        fixtures::edit(&mut game, |state| state.gold["player"] = 10);
        fixtures::edit(&mut game, |state| state.tiles[village].owner = None);
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
    }

    #[test]
    fn modest_upgrade_does_not_delay_an_affordable_recruit() {
        let (mut rules, a, b) = fixture();
        rules.units[b].cost = 12;
        rules.units[b].hp = 54;
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain()
            .iter()
            .enumerate()
            .find(|(_, t)| **t == rules.terrains["village"])
            .unwrap()
            .0;
        fixtures::edit(&mut game, |state| {
            state.tiles[village].owner = Some("player".into())
        });
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(a));
        fixtures::edit(&mut game, |state| state.gold["player"] = 12);
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
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
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
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
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
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Player)
            .unwrap()
            .clone();
        let enemy = game
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
            .unwrap()
            .clone();
        fixtures::edit(&mut game, |state| {
            for tile in &mut state.tiles {
                tile.terrain = rules.terrain_names[water].clone();
            }
        });
        fixtures::terrain(&mut game, leader.tile, rules.terrains["keep"]);
        fixtures::terrain(&mut game, enemy.tile, rules.terrains["keep"]);
        for tile in game.board().adjacent[leader.tile].clone() {
            fixtures::terrain(&mut game, tile, rules.terrains["plains"]);
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
            .map(|&t| game.board().ids[t].into())
            .collect();
        assert_eq!(selected(&game, &orders), None);
        orders.reserved.clear();
        fixtures::edit(&mut game, |state| state.gold["player"] = 0);
        assert_eq!(selected(&game, &orders), None);
    }

    #[test]
    fn wounded_company_creates_healing_demand() {
        let (mut rules, a, b) = fixture();
        rules.units[b].healing = 20;
        rules.units[b].attacks[0].damage = 0;
        let mut game = battle(&rules, a, b);
        let leader = game
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, a, true)[0];
        let id = fixtures::add(&mut game, a, "player", tile);
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
        fixtures::units(&mut game, |units| {
            units[index].hp = 1;
        });
        let wounded = assess(&game);
        // Wounds create healing value; they do not guarantee that buying a
        // healer beats every alternative under every combat valuation.
        assert!(healthy.support > 0.0);
        assert!(wounded.support > healthy.support);
        assert!(wounded.total > healthy.total);
    }
}
