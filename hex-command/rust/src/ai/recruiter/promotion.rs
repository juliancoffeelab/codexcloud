//! Earned promotions use distance-independent mechanical matchups and support demand.
use crate::sort::SharedSort;
use crate::{ai::context::BattleView, data::*};

pub(super) fn choices(
    game: &BattleView<'_, '_>,
    policy: super::RecruiterKind,
    seed: u32,
) -> Vec<(UnitId, UnitTypeId)> {
    let mut cycles = std::collections::BTreeMap::<Vec<usize>, usize>::new();
    // The round advances each tree's cycle even when promoted siblings die.
    // Eligible units alternate within the round, independently per tree.
    let mut units: Vec<_> = game
        .units()
        .iter()
        .filter(|u| game.belongs_to_faction(u, game.observer_faction()))
        .collect();
    units.order_by_key(|u| u.id);
    units
        .into_iter()
        .filter_map(|unit| {
            let options = game.promotion_options(unit);
            if options.is_empty() {
                return None;
            }
            let kind = match policy {
                super::RecruiterKind::Classic => options[0],
                super::RecruiterKind::Situational
                | super::RecruiterKind::Predictor
                | super::RecruiterKind::AggressivePredictor => {
                    choose(game, unit, policy != super::RecruiterKind::Situational)?
                }
                super::RecruiterKind::Balanced | super::RecruiterKind::Frontline => {
                    let draw = super::random(
                        seed,
                        &format!(
                            "promotion:{}:{}",
                            unit.id,
                            game.unit_definition(unit.kind).id()
                        ),
                    );
                    options[(draw * options.len() as f64) as usize]
                }
                super::RecruiterKind::Rotation => {
                    let index = cycles
                        .entry(options.clone())
                        .or_insert(game.round().saturating_sub(1) as usize);
                    let result = options[*index % options.len()];
                    *index += 1;
                    result
                }
            };
            Some((
                UnitId(unit.id),
                UnitTypeId(game.unit_definition(kind).id().to_owned()),
            ))
        })
        .collect()
}

fn choose(game: &BattleView<'_, '_>, unit: &Unit, predict: bool) -> Option<usize> {
    game.promotion_options(unit).into_iter().max_by(|&a, &b| {
        let score = |kind| {
            let promoted = game.promotion(unit, kind);
            super::evaluation::Assessment::with_forecast(game, &promoted, predict)
                .score(game, &promoted)
        };
        score(a).total_cmp(&score(b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::recruiter::fixtures;
    use crate::logic::BattleSession;
    use hex_command_battle::RuntimeCommand;

    fn fixture() -> (JsonBundle, usize, usize, usize) {
        let mut bundle = crate::catalogue_fixture::bundle();
        let base = crate::catalogue_fixture::kind(&bundle, "swordsman");
        let a = crate::catalogue_fixture::kind(&bundle, "soldier");
        let b = crate::catalogue_fixture::kind(&bundle, "hammerman");
        crate::catalogue_fixture::unit_kind(&mut bundle, base, |unit| {
            unit["advancesTo"] = serde_json::json!(["soldier", "hammerman"]);
            unit["experienceNeeded"] = serde_json::json!(1);
        });
        crate::catalogue_fixture::unit_kind(&mut bundle, a, |unit| {
            unit["attacks"].as_array_mut().unwrap().truncate(1);
            unit["attacks"][0]["count"] = serde_json::json!(1);
            unit["attacks"][0]["damage"] = serde_json::json!(10);
            unit["attacks"][0]["drain"] = serde_json::json!(0);
        });
        crate::effect_fixture::healing(&mut bundle, "soldier", None);
        crate::effect_fixture::thorn_aura(&mut bundle, "soldier", 0);
        crate::catalogue_fixture::copy_unit(&mut bundle, a, b);
        (bundle, base, a, b)
    }
    fn battle(bundle: &JsonBundle, base: usize) -> (BattleSession<'static>, u32) {
        let rules = crate::catalogue_fixture::frozen(bundle);
        let mut game = BattleSession::new(rules, "mirror");
        fixtures::units(&mut game, |units| {
            units.retain(|u| u.is_leader());
        });
        let leader = game
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, base, false)[0];
        let id = fixtures::add(&mut game, base, "player", tile);
        let i = game.index(id).unwrap();
        fixtures::units(&mut game, |units| {
            units[i].experience = Some(1);
        });
        fixtures::units(&mut game, |units| {
            units[i].moved = false;
        });
        fixtures::units(&mut game, |units| {
            units[i].attack_spent = false;
        });
        (game, id)
    }
    #[test]
    fn random_promotion_is_reproducible_and_reaches_both_branches() {
        let (rules, base, a, b) = fixture();
        let (game, _) = battle(&rules, base);
        let view = BattleView::new(&game);
        for policy in [
            super::super::RecruiterKind::Balanced,
            super::super::RecruiterKind::Frontline,
        ] {
            let outcomes: std::collections::BTreeSet<_> = (0..64)
                .map(|seed| {
                    let result = choices(&view, policy, seed);
                    assert_eq!(result, choices(&view, policy, seed));
                    result[0].1.0.clone()
                })
                .collect();
            assert_eq!(
                outcomes,
                [
                    crate::catalogue_fixture::id(&rules, a),
                    crate::catalogue_fixture::id(&rules, b)
                ]
                .into_iter()
                .collect()
            );
        }
    }

    #[test]
    fn rotation_alternates_promotions_in_one_phase_and_across_turns() {
        let (rules, base, a, b) = fixture();
        let (mut game, id) = battle(&rules, base);
        let leader = game
            .units()
            .iter()
            .find(|u| u.is_leader() && u.phase == TurnPhase::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, base, false)[0];
        let second = fixtures::add(&mut game, base, "player", tile);
        let index = game.index(second).unwrap();
        fixtures::units(&mut game, |units| {
            units[index].experience = Some(1);
        });
        let result = choices(
            &BattleView::new(&game),
            super::super::RecruiterKind::Rotation,
            7,
        );
        assert_eq!(
            result[0],
            (
                UnitId(id),
                UnitTypeId(crate::catalogue_fixture::id(&rules, a))
            )
        );
        assert_eq!(
            result[1],
            (
                UnitId(second),
                UnitTypeId(crate::catalogue_fixture::id(&rules, b))
            )
        );
        let index = game.index(id).unwrap();
        fixtures::units(&mut game, |units| {
            units[index].kind = a;
        });
        fixtures::units(&mut game, |units| {
            units[index].experience = Some(0);
        });
        fixtures::units(&mut game, |units| {
            units.retain(|u| u.id != id);
        });
        fixtures::edit(&mut game, |state| state.round += 1);
        assert_eq!(
            choices(
                &BattleView::new(&game),
                super::super::RecruiterKind::Rotation,
                7
            )[0]
            .1
            .0,
            crate::catalogue_fixture::id(&rules, b)
        );
    }

    #[test]
    fn resistance_selects_either_branch_without_changing_authored_order() {
        let (mut rules, base, a, b) = fixture();
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
        let enemy = battle(&rules, base)
            .0
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
            let (mut game, id) = battle(&rules, base);
            assert_eq!(
                choose(&BattleView::new(&game), game.unit(id).unwrap(), false),
                Some(expected)
            );
            let i = game.index(id).unwrap();
            fixtures::units(&mut game, |units| {
                units[i].experience = Some(0);
            });
            assert_eq!(
                choose(&BattleView::new(&game), game.unit(id).unwrap(), false),
                None
            );
        }
    }
    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn predictor_promotion_diagnostics_match_choices_with_future_enemy_recruits() {
        use super::super::RecruiterKind;
        let (mut rules, base, a, b) = fixture();
        for (kind, damage_type, damage) in [(a, DamageType::Blade, 11), (b, DamageType::Pierce, 10)]
        {
            {
                let fixture_value = serde_json::json!(damage_type);
                crate::catalogue_fixture::unit_kind(&mut rules, kind, |unit| {
                    unit["attacks"][0]["damageType"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(damage);
                crate::catalogue_fixture::unit_kind(&mut rules, kind, |unit| {
                    unit["attacks"][0]["damage"] = fixture_value
                });
            }
            {
                let fixture_value = serde_json::json!(0);
                crate::catalogue_fixture::unit_kind(&mut rules, kind, |unit| {
                    unit["attacks"][0]["spread"] = fixture_value
                });
            }
        }
        let observed = battle(&rules, base)
            .0
            .units()
            .iter()
            .find(|u| u.phase == TurnPhase::Ai)
            .unwrap()
            .kind;
        {
            let fixture_value = serde_json::json!(DamageModifiers {
                blade: 0,
                pierce: 0,
                impact: 0,
            });
            crate::catalogue_fixture::unit_kind(&mut rules, observed, |unit| {
                unit["damageModifiers"] = fixture_value
            });
        }
        {
            let id = crate::catalogue_fixture::id(&rules, observed);
            crate::catalogue_fixture::environment(&mut rules, &id, 0);
        }
        crate::catalogue_fixture::unit_kind(&mut rules, observed, |unit| {
            for attack in unit["attacks"].as_array_mut().unwrap() {
                attack["damage"] = serde_json::json!(0);
                attack["spread"] = serde_json::json!(0);
            }
        });
        let future = crate::catalogue_fixture::kind(&rules, "scout");
        crate::catalogue_fixture::copy_unit(&mut rules, observed, future);
        crate::catalogue_fixture::unit_kind(&mut rules, future, |unit| {
            unit["damageModifiers"]["blade"] = serde_json::json!(-10)
        });
        let (mut game, actor) = battle(&rules, base);
        game.configure_arena(Some([vec![], vec![]]), None);
        for anticipate in [false, true] {
            if anticipate {
                game.configure_arena(Some([vec![], vec![future]]), None);
            }
            for policy in [RecruiterKind::Situational, RecruiterKind::Predictor] {
                let expected = if anticipate && policy == RecruiterKind::Predictor {
                    b
                } else {
                    a
                };
                let selected = choices(&BattleView::new(&game), policy, 7)
                    .into_iter()
                    .find(|(id, _)| id.0 == actor)
                    .unwrap()
                    .1;
                assert_eq!(selected.0, crate::catalogue_fixture::id(&rules, expected));
                let rows = super::super::diagnostics(&game, policy);
                let best = rows
                    .iter()
                    .filter(|r| r["choice"] == "promotion" && r["actor"] == actor)
                    .max_by(|a, b| {
                        a["score"]
                            .as_f64()
                            .unwrap()
                            .total_cmp(&b["score"].as_f64().unwrap())
                    })
                    .unwrap();
                assert_eq!(best["unit"], selected.0);
            }
        }
    }
    #[test]
    fn wounded_and_exposed_allies_make_support_branch_useful() {
        for thorns in [false, true] {
            let (mut rules, base, a, b) = fixture();
            {
                let fixture_value = serde_json::json!(0);
                crate::catalogue_fixture::unit_kind(&mut rules, b, |unit| {
                    unit["attacks"][0]["damage"] = fixture_value
                });
            }
            if thorns {
                crate::effect_fixture::thorn_aura_kind(&mut rules, b, 20);
            } else {
                crate::effect_fixture::healing_kind(
                    &mut rules,
                    b,
                    (20 > 0).then_some((20, 1, false, crate::data::HealingTargets::Allies)),
                );
            }
            let enemy = battle(&rules, base)
                .0
                .units()
                .iter()
                .find(|u| u.phase == TurnPhase::Ai)
                .unwrap()
                .kind;
            // A committed multi-strike melee enemy makes aura support useful.
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
                let fixture_value = serde_json::json!(3);
                crate::catalogue_fixture::unit_kind(&mut rules, enemy, |unit| {
                    unit["attacks"][0]["count"] = fixture_value
                });
            }
            let (mut game, id) = battle(&rules, base);
            let unit = game.unit(id).unwrap().clone();
            let tiles: Vec<_> = game.board().adjacent[unit.tile]
                .iter()
                .copied()
                .filter(|&t| game.at(t).is_none())
                .collect();
            if !thorns {
                assert_eq!(choose(&BattleView::new(&game), &unit, false), Some(a));
            }
            let ally = fixtures::add(&mut game, base, "player", tiles[0]);
            let i = game.index(ally).unwrap();
            fixtures::units(&mut game, |units| {
                units[i].hp = 1;
            });
            let enemy = game
                .units()
                .iter()
                .find(|u| u.phase == TurnPhase::Ai)
                .unwrap()
                .id;
            let i = game.index(enemy).unwrap();
            fixtures::units(&mut game, |units| {
                units[i].tile = tiles[1];
            });
            assert_eq!(choose(&BattleView::new(&game), &unit, false), Some(b));
        }
    }
    #[test]
    fn every_planner_executes_the_recruiters_selected_branch() {
        use crate::ai::{
            Diplomacy, PlanningWorkspace, campaigner::CampaignerKind, planner::PlannerKind,
            recruiter::Recruiter, runtime::tactic::Tactic,
        };
        struct Fixed(usize);
        impl Recruiter for Fixed {
            fn recruit(
                &mut self,
                _: &mut crate::ai::context::RecruitmentContext<'_, '_>,
                _: &crate::ai::orders::RecruitmentOrders,
            ) -> Result<crate::ai::orders::RecruitmentReport, CommandError> {
                Ok(Default::default())
            }
            fn promotions(
                &mut self,
                game: &BattleView<'_, '_>,
                _: u32,
            ) -> Vec<(UnitId, UnitTypeId)> {
                game.units()
                    .iter()
                    .filter(|u| game.promotion_options(u).contains(&self.0))
                    .map(|u| {
                        (
                            UnitId(u.id),
                            UnitTypeId(game.unit_definition(self.0).id().to_owned()),
                        )
                    })
                    .collect()
            }
        }
        let (rules, base, a, b) = fixture();
        for branch in [a, b] {
            for planner in [PlannerKind::Classic, PlannerKind::Full, PlannerKind::Queued] {
                let (game, id) = battle(&rules, base);
                let mut tactic = Tactic::new(Fixed(branch), CampaignerKind::Arena, planner);
                let mut workspace = PlanningWorkspace::new(7, 0, None, Diplomacy::Diplomatic);
                let plan = tactic.run_turn(&game, &mut workspace).unwrap();
                assert!(
                    plan.commands.contains(
                        &RuntimeCommand::Advance {
                            unit: UnitId(id),
                            kind: UnitKind(branch),
                        }
                        .into()
                    ),
                    "{planner:?} did not execute selected branch"
                );
                let mut replay = game.fork();
                for command in plan.commands {
                    replay.execute_proposal(command).unwrap();
                }
                assert_eq!(replay.unit(id).unwrap().kind, branch);
            }
        }
    }
}
