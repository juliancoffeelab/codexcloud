//! Adaptive recruitment using engine matchups and terrain routes, without role quotas.
use super::{Recruiter, RecruiterKind};
use crate::ai::{context::BattleView, orders::RecruitmentOrders};
use crate::data::*;
use hex_command_battle::RuntimeCommand;

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
) -> Option<RuntimeCommand> {
    choose_with_forecast(game, seed, orders, false)
}

pub(super) fn choose_with_forecast(
    game: &BattleView<'_, '_>,
    seed: u32,
    orders: &RecruitmentOrders,
    predict: bool,
) -> Option<RuntimeCommand> {
    choose_with_weights(
        game,
        seed,
        orders,
        predict,
        game.catalogue().meta().recruiter_weights.standard,
    )
}

pub(super) fn choose_with_weights(
    game: &BattleView<'_, '_>,
    seed: u32,
    orders: &RecruitmentOrders,
    predict: bool,
    weights: RecruitmentWeights,
) -> Option<RuntimeCommand> {
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
        if price > gold && income == 0 {
            continue;
        }
        let mut recruit = game.prospective(kind, leader.faction, tile);
        let rule = game.unit_definition(recruit.kind);
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
            .filter(|u| game.unit_definition(u.kind).family() == rule.family())
            .count();
        // An unaffordable unit must justify the income spent waiting rather
        // than fielding an available alternative now.
        let wait = f64::from((price - gold).max(0)) / f64::from(income.max(1));
        let draw =
            crate::ai::policies::tie_breaking::RecruitmentDraw::new(seed, game.round(), rule.id());
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
    use crate::logic::BattleSession;
    use hex_command_battle::RuntimeCommand;

    // Controlled alternatives test the decision, independent of authored balance.
    fn fixture() -> (JsonBundle, usize, usize) {
        let mut bundle = crate::catalogue_fixture::neutral_combat();
        let a = crate::catalogue_fixture::kind(&bundle, "swordsman");
        let b = crate::catalogue_fixture::kind(&bundle, "archer");
        crate::catalogue_fixture::unit_kind(&mut bundle, a, |unit| {
            unit["hp"] = serde_json::json!(40);
            unit["cost"] = serde_json::json!(10);
            unit["damageModifiers"] = serde_json::json!({"blade":0,"pierce":0,"impact":0});
            unit["attacks"].as_array_mut().unwrap().truncate(1);
            unit["attacks"][0]["damage"] = serde_json::json!(10);
            unit["attacks"][0]["count"] = serde_json::json!(1);
            unit["attacks"][0]["category"] = serde_json::json!("melee");
            unit["attacks"][0]["range"] = serde_json::json!(1);
        });
        crate::effect_fixture::healing(&mut bundle, "swordsman", None);
        crate::catalogue_fixture::environment(&mut bundle, "swordsman", 0);
        crate::catalogue_fixture::copy_unit(&mut bundle, a, b);
        (bundle, a, b)
    }

    fn battle(bundle: &JsonBundle, a: usize, b: usize) -> BattleSession<'static> {
        let rules = crate::catalogue_fixture::frozen(bundle);
        let mut game = BattleSession::new(rules, "mirror");
        fixtures::units(&mut game, |units| {
            units.retain(|u| u.is_leader());
        });
        game.configure_arena(Some([vec![a, b], vec![a, b]]), None);
        fixtures::edit(&mut game, |state| state.gold["player"] = 10);
        game
    }

    fn selected(game: &BattleSession<'_>, orders: &RecruitmentOrders) -> Option<usize> {
        choose(&BattleView::new(game), 7, orders).map(|command| {
            let RuntimeCommand::Recruit { kind, .. } = command else {
                panic!("recruiter emitted a non-recruit command");
            };
            kind.0
        })
    }

    #[test]
    fn one_strike_is_preferred_to_equal_damage_multiple_strikes_against_thorns() {
        let (mut rules, a, b) = fixture();
        {
            let fixture_value = serde_json::json!(6);
            crate::catalogue_fixture::unit_kind(&mut rules, a, |unit| {
                unit["attacks"][0]["damage"] = fixture_value
            });
        }
        {
            let fixture_value = serde_json::json!(3);
            crate::catalogue_fixture::unit_kind(&mut rules, a, |unit| {
                unit["attacks"][0]["count"] = fixture_value
            });
        }
        {
            let fixture_value = serde_json::json!(18);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                unit["attacks"][0]["damage"] = fixture_value
            });
        }
        let enemy = battle(&rules, a, b)
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
            .unwrap()
            .kind;
        {
            let fixture_value = serde_json::json!(100);
            crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                unit["hp"] = fixture_value
            });
        }
        crate::effect_fixture::thorns_kind(&mut rules, enemy, 4);
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
                {
                    let fixture_value = serde_json::json!(-6);
                    crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                        unit["damageModifiers"]["blade"] = fixture_value
                    });
                }
            } else {
                crate::effect_fixture::thorns_kind(&mut rules, b, 12);
            }
            let enemy = battle(&rules, a, b)
                .units()
                .iter()
                .find(|u| u.phase == TurnPhase::Ai)
                .unwrap()
                .kind;
            crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                unit["attacks"].as_array_mut().unwrap().truncate(1)
            });
            {
                let fixture_value = serde_json::json!(Category::Melee);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["attacks"][0]["category"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(1);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["attacks"][0]["range"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(DamageType::Blade);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["attacks"][0]["damageType"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(8);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["attacks"][0]["damage"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(3);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["attacks"][0]["count"] = fixture_value
                });
            }
            assert_eq!(
                selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
                Some(b)
            );
        }
    }

    #[test]
    fn valuable_expensive_recruit_can_wait_for_income() {
        let (mut rules, a, b) = fixture();
        {
            let fixture_value = serde_json::json!(12);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| unit["cost"] = fixture_value);
        }
        {
            let fixture_value = serde_json::json!(80);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| unit["hp"] = fixture_value);
        }
        {
            let fixture_value = serde_json::json!(20);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                unit["attacks"][0]["damage"] = fixture_value
            });
        }
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain()
            .iter()
            .enumerate()
            .find(|(_, t)| {
                **t == crate::catalogue_fixture::frozen(&rules)
                    .terrains()
                    .position(|terrain| terrain.id().as_str() == "village")
                    .unwrap()
            })
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
        {
            let fixture_value = serde_json::json!(12);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| unit["cost"] = fixture_value);
        }
        {
            let fixture_value = serde_json::json!(54);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| unit["hp"] = fixture_value);
        }
        let mut game = battle(&rules, a, b);
        let village = game
            .terrain()
            .iter()
            .enumerate()
            .find(|(_, t)| {
                **t == crate::catalogue_fixture::frozen(&rules)
                    .terrains()
                    .position(|terrain| terrain.id().as_str() == "village")
                    .unwrap()
            })
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
        {
            let fixture_value = serde_json::json!(
                (crate::catalogue_fixture::value(&rules, a)["move"]
                    .as_i64()
                    .unwrap() as usize)
                    * 2
            );
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| unit["move"] = fixture_value);
        }
        assert_eq!(
            selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
            Some(b)
        );
        {
            let fixture_value = serde_json::json!(
                (crate::catalogue_fixture::value(&rules, a)["move"]
                    .as_i64()
                    .unwrap() as usize)
            );
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| unit["move"] = fixture_value);
        }
        {
            let fixture_value = serde_json::json!(6);
            crate::catalogue_fixture::unit_kind(&mut rules, a, |unit| {
                unit["attacks"][0]["damage"] = fixture_value
            });
        }
        {
            let fixture_value = serde_json::json!(3);
            crate::catalogue_fixture::unit_kind(&mut rules, a, |unit| {
                unit["attacks"][0]["count"] = fixture_value
            });
        }
        {
            let fixture_value = serde_json::json!(18);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                unit["attacks"][0]["damage"] = fixture_value
            });
        }
        let enemy = battle(&rules, a, b)
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
            .unwrap()
            .kind;
        let damage_type = serde_json::from_value::<DamageType>(
            crate::catalogue_fixture::value(&rules, a)["attacks"][0]["damageType"].clone(),
        )
        .unwrap();
        {
            let fixture_value = serde_json::json!(damage_type);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                unit["attacks"][0]["damageType"] = fixture_value
            });
        }
        {
            let fixture_value = serde_json::json!(DamageModifiers {
                blade: -4,
                pierce: -4,
                impact: -4,
            });
            crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                unit["damageModifiers"] = fixture_value
            });
        }
        assert_eq!(
            selected(&battle(&rules, a, b), &RecruitmentOrders::default()),
            Some(b)
        );
    }

    #[test]
    fn enemy_resistance_changes_selection() {
        let (mut rules, a, b) = fixture();
        {
            let fixture_value = serde_json::json!(DamageType::Blade);
            crate::catalogue_fixture::unit_kind(&mut rules, a, |unit| {
                unit["attacks"][0]["damageType"] = fixture_value
            });
        }
        {
            let fixture_value = serde_json::json!(DamageType::Pierce);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                unit["attacks"][0]["damageType"] = fixture_value
            });
        }
        let enemy = battle(&rules, a, b)
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
            .unwrap()
            .kind;
        for (blade, pierce, expected) in [(-10, 0, b), (0, -10, a)] {
            {
                let fixture_value = serde_json::json!(blade);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["damageModifiers"]["blade"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(pierce);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["damageModifiers"]["pierce"] = fixture_value
                });
            }
            {
                let id = crate::catalogue_fixture::id(&rules, enemy);
                crate::catalogue_fixture::environment(&mut rules, &id, 0);
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
        let water = crate::catalogue_fixture::frozen(&rules)
            .terrains()
            .position(|terrain| terrain.id().as_str() == "water")
            .unwrap();
        crate::catalogue_fixture::unit_kind(&mut rules, a, |unit| {
            unit["terrainModifiers"]["water"]["passable"] = serde_json::json!(false);
            unit["terrainModifiers"]["water"]["cost"] = serde_json::json!(0);
        });
        crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
            unit["terrainModifiers"]["water"]["passable"] = serde_json::json!(true);
            unit["terrainModifiers"]["water"]["cost"] = serde_json::json!(0);
        });
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
                tile.terrain = crate::catalogue_fixture::frozen(&rules)
                    .terrains()
                    .nth(water)
                    .unwrap()
                    .id()
                    .as_str()
                    .into();
            }
        });
        fixtures::terrain(
            &mut game,
            leader.tile,
            crate::catalogue_fixture::frozen(&rules)
                .terrains()
                .position(|terrain| terrain.id().as_str() == "keep")
                .unwrap(),
        );
        fixtures::terrain(
            &mut game,
            enemy.tile,
            crate::catalogue_fixture::frozen(&rules)
                .terrains()
                .position(|terrain| terrain.id().as_str() == "keep")
                .unwrap(),
        );
        for tile in game.board().adjacent[leader.tile].clone() {
            fixtures::terrain(
                &mut game,
                tile,
                crate::catalogue_fixture::frozen(&rules)
                    .terrains()
                    .position(|terrain| terrain.id().as_str() == "plains")
                    .unwrap(),
            );
        }
        assert_eq!(selected(&game, &RecruitmentOrders::default()), Some(b));
        let mut orders = RecruitmentOrders {
            requirements: vec![crate::ai::orders::RecruitRequirement::EnsureUnits {
                selector: crate::ai::orders::UnitSelector::Type(
                    crate::catalogue_fixture::id(&rules, a).as_str().into(),
                ),
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
        crate::effect_fixture::healing_kind(
            &mut rules,
            b,
            (20 > 0).then_some((20, 1, false, crate::data::HealingTargets::Allies)),
        );
        {
            let fixture_value = serde_json::json!(0);
            crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                unit["attacks"][0]["damage"] = fixture_value
            });
        }
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
        healer.hp = (crate::catalogue_fixture::value(&rules, b)["hp"]
            .as_i64()
            .unwrap() as i32);
        healer.tile = game.placement_tiles(&leader, b, true)[0];
        let assess = |game: &BattleSession<'_>| {
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
