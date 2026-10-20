//! Earned promotions use distance-independent mechanical matchups and support demand.
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
        .filter(|u| u.side == game.turn())
        .collect();
    units.sort_by_key(|u| u.id);
    units
        .into_iter()
        .filter_map(|unit| {
            let options = game.promotion_options(unit);
            if options.is_empty() {
                return None;
            }
            let kind = match policy {
                super::RecruiterKind::Classic => options[0],
                super::RecruiterKind::Situational | super::RecruiterKind::Predictor => {
                    choose(game, unit, policy == super::RecruiterKind::Predictor)?
                }
                super::RecruiterKind::Balanced | super::RecruiterKind::Frontline => {
                    let draw = super::random(
                        seed,
                        &format!("promotion:{}:{}", unit.id, game.rules().units[unit.kind].id),
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
                UnitTypeId(game.rules().units[kind].id.clone()),
            ))
        })
        .collect()
}

fn choose(game: &BattleView<'_, '_>, unit: &Unit, predict: bool) -> Option<usize> {
    let wounds = game.rules().units[unit.kind].hp - unit.hp;
    game.promotion_options(unit).into_iter().max_by(|&a, &b| {
        let score = |kind| {
            let mut promoted = unit.clone();
            promoted.kind = kind;
            promoted.hp = (game.rules().units[kind].hp - wounds).max(1);
            super::evaluation::Assessment::with_forecast(game, &promoted, predict)
                .score(game, &promoted)
        };
        score(a).total_cmp(&score(b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Game;

    fn fixture() -> (Rules, usize, usize, usize) {
        let mut rules = Rules::load().unwrap();
        let base = rules.types["swordsman"];
        let a = rules.types["soldier"];
        let b = rules.types["hammerman"];
        rules.units[base].advances_to = vec![rules.units[a].id.clone(), rules.units[b].id.clone()];
        rules.units[base].experience_needed = Some(1);
        let mut branch = rules.units[a].clone();
        branch.attacks.truncate(1);
        branch.attacks[0].count = 1;
        branch.attacks[0].damage = 10;
        branch.attacks[0].drain = 0;
        branch.healing = 0;
        branch.thorn_aura = 0;
        for kind in [a, b] {
            let id = rules.units[kind].id.clone();
            rules.units[kind] = branch.clone();
            rules.units[kind].id = id;
        }
        (rules, base, a, b)
    }
    fn battle(rules: &Rules, base: usize) -> (Game<'_>, u32) {
        let mut game = Game::new(rules, "mirror");
        game.units.retain(|u| rules.units[u.kind].leader);
        game.roster_changed();
        let leader = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, base, false)[0];
        let id = game.add(base, Side::Player, tile);
        let i = game.index(id).unwrap();
        game.units[i].experience = Some(1);
        game.units[i].moved = false;
        game.units[i].attacked = false;
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
                [rules.units[a].id.clone(), rules.units[b].id.clone()]
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
            .units
            .iter()
            .find(|u| rules.units[u.kind].leader && u.side == Side::Player)
            .unwrap()
            .clone();
        let tile = game.placement_tiles(&leader, base, false)[0];
        let second = game.add(base, Side::Player, tile);
        let index = game.index(second).unwrap();
        game.units[index].experience = Some(1);
        let result = choices(
            &BattleView::new(&game),
            super::super::RecruiterKind::Rotation,
            7,
        );
        assert_eq!(
            result[0],
            (UnitId(id), UnitTypeId(rules.units[a].id.clone()))
        );
        assert_eq!(
            result[1],
            (UnitId(second), UnitTypeId(rules.units[b].id.clone()))
        );
        let index = game.index(id).unwrap();
        game.units[index].kind = a;
        game.units[index].experience = Some(0);
        game.units.retain(|u| u.id != id);
        game.roster_changed();
        game.round += 1;
        assert_eq!(
            choices(
                &BattleView::new(&game),
                super::super::RecruiterKind::Rotation,
                7
            )[0]
            .1
            .0,
            rules.units[b].id
        );
    }

    #[test]
    fn resistance_selects_either_branch_without_changing_authored_order() {
        let (mut rules, base, a, b) = fixture();
        rules.units[a].attacks[0].damage_type = DamageType::Blade;
        rules.units[b].attacks[0].damage_type = DamageType::Pierce;
        let enemy = battle(&rules, base)
            .0
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        for (blade, pierce, expected) in [(-10, 0, b), (0, -10, a)] {
            rules.units[enemy].damage_modifiers.blade = blade;
            rules.units[enemy].damage_modifiers.pierce = pierce;
            let (mut game, id) = battle(&rules, base);
            assert_eq!(
                choose(&BattleView::new(&game), game.unit(id).unwrap(), false),
                Some(expected)
            );
            let i = game.index(id).unwrap();
            game.units[i].experience = Some(0);
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
            rules.units[kind].attacks[0].damage_type = damage_type;
            rules.units[kind].attacks[0].damage = damage;
            rules.units[kind].attacks[0].spread = 0;
        }
        let observed = battle(&rules, base)
            .0
            .units
            .iter()
            .find(|u| u.side == Side::Enemy)
            .unwrap()
            .kind;
        rules.units[observed].damage_modifiers = DamageModifiers {
            blade: 0,
            pierce: 0,
            impact: 0,
        };
        for environment in &mut rules.units[observed].environment {
            *environment = [[0; 3]; 2];
        }
        for attack in &mut rules.units[observed].attacks {
            attack.damage = 0;
            attack.spread = 0;
        }
        let future = rules.types["scout"];
        let mut foe = rules.units[observed].clone();
        foe.id = rules.units[future].id.clone();
        foe.leader = false;
        foe.damage_modifiers.blade = -10;
        rules.units[future] = foe;
        rules.route_classes[future] = future;
        let (mut game, actor) = battle(&rules, base);
        game.pools = Some([vec![], vec![]]);
        for anticipate in [false, true] {
            if anticipate {
                game.pools.as_mut().unwrap()[1] = vec![future];
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
                assert_eq!(selected.0, rules.units[expected].id);
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
            rules.units[b].attacks[0].damage = 0;
            if thorns {
                rules.units[b].thorn_aura = 20;
            } else {
                rules.units[b].healing = 20;
            }
            let enemy = battle(&rules, base)
                .0
                .units
                .iter()
                .find(|u| u.side == Side::Enemy)
                .unwrap()
                .kind;
            // A committed multi-strike melee enemy makes aura support useful.
            rules.units[enemy].attacks.truncate(1);
            rules.units[enemy].attacks[0].category = Category::Melee;
            rules.units[enemy].attacks[0].range = 1;
            rules.units[enemy].attacks[0].count = 3;
            let (mut game, id) = battle(&rules, base);
            let unit = game.unit(id).unwrap().clone();
            let tiles: Vec<_> = game.board.adjacent[unit.tile]
                .iter()
                .copied()
                .filter(|&t| game.at(t).is_none())
                .collect();
            if !thorns {
                assert_eq!(choose(&BattleView::new(&game), &unit, false), Some(a));
            }
            let ally = game.add(base, Side::Player, tiles[0]);
            let i = game.index(ally).unwrap();
            game.units[i].hp = 1;
            let enemy = game
                .units
                .iter()
                .find(|u| u.side == Side::Enemy)
                .unwrap()
                .id;
            let i = game.index(enemy).unwrap();
            game.units[i].tile = tiles[1];
            game.roster_changed();
            assert_eq!(choose(&BattleView::new(&game), &unit, false), Some(b));
        }
    }
    #[test]
    fn every_planner_executes_the_recruiters_selected_branch() {
        use crate::ai::{
            Diplomacy, PlanningWorkspace, campaigner::CampaignerKind, planner::PlannerKind,
            recruiter::Recruiter, tactic::Tactic,
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
                            UnitTypeId(game.rules().units[self.0].id.clone()),
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
                let mut workspace = PlanningWorkspace::new(7, 0, None, Diplomacy::Restrained);
                let plan = tactic.run_turn(&game, &mut workspace).unwrap();
                assert!(
                    plan.commands.contains(&crate::logic::Command::Advance {
                        unit: UnitId(id),
                        kind: UnitTypeId(rules.units[branch].id.clone()),
                    }),
                    "{planner:?} did not execute selected branch"
                );
                let mut replay = game.fork();
                for command in plan.commands {
                    replay.execute(command).unwrap();
                }
                assert_eq!(replay.unit(id).unwrap().kind, branch);
            }
        }
    }
}
