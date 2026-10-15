//! Earned promotion choices use mechanical matchups and nearby support demand.
use crate::{ai::context::BattleView, data::*};

pub(super) fn choices(game: &BattleView<'_, '_>, adaptive: bool) -> Vec<(UnitId, UnitTypeId)> {
    game.units()
        .iter()
        .filter(|unit| unit.side == game.turn())
        .filter_map(|unit| {
            let kind = if adaptive {
                choose(game, unit)
            } else {
                game.promotion_options(unit).first().copied()
            }?;
            Some((
                UnitId(unit.id),
                UnitTypeId(game.rules().units[kind].id.clone()),
            ))
        })
        .collect()
}

fn choose(game: &BattleView<'_, '_>, unit: &Unit) -> Option<usize> {
    let options = game.promotion_options(unit);
    if options.len() <= 1 {
        return options.first().copied();
    }
    let wounds = game.rules().units[unit.kind].hp - unit.hp;
    let enemies: Vec<_> = game
        .units()
        .iter()
        .filter(|u| u.side != unit.side)
        .collect();
    let mut best = None;
    for kind in options {
        let rule = &game.rules().units[kind];
        let mut promoted = unit.clone();
        promoted.kind = kind;
        promoted.hp = (rule.hp - wounds).max(1);
        let mut offense = 0.0;
        let mut incoming = 0.0;
        let mut weights = 0.0;
        for enemy in &enemies {
            let distance = game.board().distance(unit.tile, enemy.tile) as f64;
            let weight = 1.0 / (1.0 + distance);
            let damage = rule
                .attacks
                .iter()
                .enumerate()
                .map(|(w, attack)| {
                    let origins = game.attack_origins(attack.category, attack.range);
                    let mut goals = Vec::new();
                    origins[enemy.tile].for_each(|tile| {
                        if game.cost(kind, tile) != u16::MAX {
                            goals.push(tile);
                        }
                    });
                    let route = game.routes(kind, &goals)[unit.tile];
                    if route == u16::MAX {
                        return 0.0;
                    }
                    let turns = route as f64 / rule.r#move.max(1) as f64;
                    let damage = game.damage(&promoted, enemy, w).min(enemy.hp) as f64;
                    // Drain remains useful after future exchanges, even when healthy now.
                    damage * (1.0 + attack.drain as f64 / 200.0) / (1.0 + turns * 0.35)
                })
                .fold(0.0, f64::max);
            offense += damage * weight;
            let enemy_rule = &game.rules().units[enemy.kind];
            incoming += (0..enemy_rule.attacks.len())
                .map(|w| game.damage(enemy, &promoted, w) as f64)
                .fold(0.0, f64::max)
                * weight;
            weights += weight;
        }
        if weights > 0.0 {
            offense /= weights;
            incoming /= weights;
        }
        let mut support = 0.0;
        for ally in game
            .units()
            .iter()
            .filter(|a| a.side == unit.side && a.id != unit.id)
        {
            if game.board().distance(unit.tile, ally.tile) as usize > rule.r#move + 1 {
                continue;
            }
            let exposed = enemies.iter().any(|e| {
                game.board().distance(ally.tile, e.tile)
                    <= game.rules().units[e.kind].r#move as u16 + 1
            });
            let ally_rule = &game.rules().units[ally.kind];
            let mut healing = 0;
            let mut aura = 0;
            for other in game.units().iter().filter(|a| {
                a.side == unit.side
                    && a.id != unit.id
                    && a.id != ally.id
                    && game.board().distance(a.tile, ally.tile) == 1
            }) {
                healing = healing.max(game.rules().units[other.kind].healing);
                aura = aura.max(game.rules().units[other.kind].thorn_aura);
            }
            let demand = ally_rule.hp - ally.hp + if exposed { ally_rule.hp / 5 } else { 0 };
            let heal = (demand - healing).max(0).min(rule.healing);
            let hits = ally_rule
                .attacks
                .iter()
                .filter(|a| a.category == Category::Melee)
                .map(|a| a.count)
                .max()
                .unwrap_or(0);
            let thorns = if exposed {
                (rule.thorn_aura - aura).max(0) * i32::from(hits)
            } else {
                0
            };
            let distance = game.board().distance(unit.tile, ally.tile).max(1) as f64;
            support += (heal + thorns) as f64 / distance;
        }
        let durability = promoted.hp as f64 / (1.0 + incoming / promoted.hp as f64);
        let score = offense * 1.5 + durability * 0.35 + support * 2.0 + rule.r#move as f64 * 0.25;
        if best.is_none_or(|(previous, _)| score > previous) {
            best = Some((score, kind));
        }
    }
    best.map(|(_, kind)| kind)
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
                choose(&BattleView::new(&game), game.unit(id).unwrap()),
                Some(expected)
            );
            let i = game.index(id).unwrap();
            game.units[i].experience = Some(0);
            assert_eq!(
                choose(&BattleView::new(&game), game.unit(id).unwrap()),
                None
            );
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
            let (mut game, id) = battle(&rules, base);
            let unit = game.unit(id).unwrap().clone();
            let tiles: Vec<_> = game.board.adjacent[unit.tile]
                .iter()
                .copied()
                .filter(|&t| game.at(t).is_none())
                .collect();
            assert_eq!(choose(&BattleView::new(&game), &unit), Some(a));
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
            assert_eq!(choose(&BattleView::new(&game), &unit), Some(b));
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
            fn promotions(&mut self, game: &BattleView<'_, '_>) -> Vec<(UnitId, UnitTypeId)> {
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
                fn advances(command: &crate::logic::Command, id: u32, expected: &str) -> bool {
                    match command {
                        crate::logic::Command::Advance { unit, kind } => {
                            unit.0 == id && kind.0 == expected
                        }
                        crate::logic::Command::Move { .. }
                        | crate::logic::Command::Attack { .. }
                        | crate::logic::Command::Recruit { .. }
                        | crate::logic::Command::SetUnitIntent { .. }
                        | crate::logic::Command::DismissDialogue
                        | crate::logic::Command::RecallMission { .. }
                        | crate::logic::Command::RetryMission
                        | crate::logic::Command::PreviousDialogue
                        | crate::logic::Command::SkipDialogue
                        | crate::logic::Command::EndTurn => false,
                    }
                }
                assert!(
                    plan.commands.iter().any(|command| advances(
                        command,
                        id,
                        &rules.units[branch].id
                    )),
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
