//! Independently selectable AI blocks. Policy names select fixed presets.
use super::{CampaignBlock, Planner, Policy, Recruiter};
use super::{campaigner::CampaignerKind, planner::PlannerKind, recruiter::RecruiterKind};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Blocks {
    pub recruiter: RecruiterKind,
    pub campaign: CampaignerKind,
    pub planner: PlannerKind,
}

impl Policy {
    /// Classic keeps its live recruitment cap; simulated player presets have none.
    pub const fn default_recruitment_limit(self) -> Option<usize> {
        match self {
            Self::Classic => Some(2),
            Self::Rush | Self::Balanced | Self::Rotated | Self::Situational | Self::Predictor => {
                None
            }
        }
    }

    /// Resolve a live preset. Every policy has a complete composition.
    pub const fn blocks(self) -> Blocks {
        match self {
            Self::Classic => Blocks {
                recruiter: Recruiter::Classic,
                campaign: CampaignBlock::Mission,
                planner: Planner::Classic,
            },
            Self::Rush => Blocks {
                recruiter: Recruiter::Frontline,
                campaign: CampaignBlock::Mission,
                planner: Planner::Queued,
            },
            Self::Balanced => Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::Mission,
                planner: Planner::Queued,
            },
            Self::Rotated => Blocks {
                recruiter: Recruiter::Rotation,
                campaign: CampaignBlock::Mission,
                planner: Planner::Queued,
            },
            Self::Situational => Blocks {
                recruiter: Recruiter::Situational,
                campaign: CampaignBlock::Mission,
                planner: Planner::Queued,
            },
            Self::Predictor => Blocks {
                recruiter: Recruiter::Predictor,
                campaign: CampaignBlock::Mission,
                planner: Planner::Queued,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::Action as Command;
    use crate::{
        ai::{Diplomacy, PlanningWorkspace, choose_game_command},
        data::*,
        logic::Game,
    };

    fn end_turn(command: &Command) -> bool {
        fn boundary(command: &Command) -> bool {
            match command {
                Command::EndTurn => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::SetUnitIntent { .. } => false,
            }
        }
        boundary(command)
    }

    #[test]
    fn player_and_enemy_rebuilding_emit_explicit_hold_targets() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        game.units.retain(|u| rules.units[u.kind].leader);
        game.roster_changed();
        for side in [Side::Player, Side::Enemy] {
            game.begin_turn(side);
            game.gold[side.index()] = 100;
            let leader = game.units.iter().find(|u| u.side == side).unwrap().id;
            let workspace = PlanningWorkspace::new(7, 0, None, Diplomacy::Restrained);
            let orders = CampaignBlock::Mission.campaign_orders(&game, &workspace);
            let emitted: Vec<_> = orders
                .tactics
                .targets
                .iter()
                .filter(|g| g.id.starts_with("rebuild:"))
                .collect();
            let refreshed = orders
                .tactics
                .rebuild_objectives(&crate::ai::context::BattleView::new(&game));
            assert!(!emitted.is_empty() && !refreshed.is_empty());
            for goal in emitted.into_iter().chain(refreshed.iter()) {
                assert!(crate::ai::objectives::actor_matches(&goal.actor, leader));
                assert_eq!(goal.hold, Some(true));
            }
        }
    }

    #[test]
    fn unnamed_seal_specialist_gets_arrival_and_escort_without_survival_or_occupation() {
        let rules = Rules::load().unwrap();
        let mut game =
            Game::start_campaign(&rules, "lantern-road", 2, vec![], 0, vec![], vec![], vec![]);
        let adept = game
            .units
            .iter()
            .find(|u| rules.units[u.kind].family == Family::Adept)
            .unwrap()
            .clone();
        assert!(adept.name.is_none());
        let tile = game.board.adjacent[adept.tile]
            .iter()
            .copied()
            .find(|&t| game.at(t).is_none())
            .unwrap();
        let enemy = game
            .units
            .iter_mut()
            .find(|u| u.side == Side::Enemy && !rules.units[u.kind].leader)
            .unwrap();
        enemy.tile = tile;
        game.roster_changed();
        let mut workspace = PlanningWorkspace::new(7, 0, None, Diplomacy::Restrained);
        workspace.orders = Some(CampaignBlock::Mission.campaign_orders(&game, &workspace));
        crate::ai::cooperation::assign(&crate::ai::context::BattleView::new(&game), &mut workspace);
        let orders = &workspace.orders.as_ref().unwrap().tactics;
        assert!(!orders.protects(adept.id));
        assert!(orders.constraints.iter().all(|constraint| {
            let crate::ai::orders::TacticalConstraint::Survive { actor } = constraint;
            game.unit(actor.0).unwrap().name.is_some()
        }));
        let mut reach = false;
        let mut escort = false;
        for order in &orders.objectives {
            match &order.objective {
                crate::ai::orders::TacticalObjective::GoTo { actor, .. } => {
                    reach |= actor.0 == adept.id
                }
                crate::ai::orders::TacticalObjective::Escort { actor, .. } => {
                    escort |= actor.0 == adept.id
                }
                crate::ai::orders::TacticalObjective::Occupy { actor, .. } => {
                    assert_ne!(actor.0, adept.id)
                }
                crate::ai::orders::TacticalObjective::ClearPassage { .. } => {}
            }
        }
        assert!(reach && escort);
        let target = orders
            .targets
            .iter()
            .find(|g| crate::ai::objectives::actor_matches(&g.actor, adept.id))
            .unwrap();
        assert_eq!(target.hold, Some(false));
    }

    #[test]
    fn independent_blocks_keep_the_selected_planner_under_another_preset() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let blocks = Policy::Balanced.blocks();
        let mut custom =
            PlanningWorkspace::new(7, 1, None, Diplomacy::Restrained).with_blocks(blocks);
        let mut preset = PlanningWorkspace::new(7, 1, None, Diplomacy::Restrained);
        for _ in 0..64 {
            let command = choose_game_command(&game, Policy::Classic, &mut custom).unwrap();
            assert_eq!(
                command,
                choose_game_command(&game, Policy::Balanced, &mut preset).unwrap()
            );
            assert_eq!(custom.blocks, Some(blocks));
            if end_turn(&command) {
                return;
            }
            game.execute(command).unwrap();
        }
        panic!("queued phase failed to finish");
    }

    #[test]
    fn campaign_requests_a_missing_seal_specialist_from_a_frontline_recruiter() {
        let rules = Rules::load().unwrap();
        let mut game =
            Game::start_campaign(&rules, "lantern-road", 2, vec![], 0, vec![], vec![], vec![]);
        game.units
            .retain(|u| u.side == Side::Player && rules.units[u.kind].family != Family::Adept);
        let mut screen = game
            .units
            .iter()
            .find(|u| !rules.units[u.kind].leader)
            .unwrap()
            .clone();
        for (id, tile) in [
            (20, "2,0"),
            (21, "3,0"),
            (22, "4,0"),
            (23, "5,0"),
            (24, "6,0"),
        ] {
            screen.id = id;
            screen.tile = game.site(tile);
            game.units.push(screen.clone());
        }
        game.next_id = 25;
        game.gold[Side::Player.index()] = 100;
        game.roster_changed();
        assert_eq!(game.required_recruit(Side::Player), Some(Family::Adept));
        let leader = game
            .units
            .iter()
            .find(|u| rules.units[u.kind].leader)
            .unwrap()
            .id;
        assert_eq!(game.leader_rebuild_plan(leader, false).1, Some(true));
        let command = Recruiter::Frontline.choose(&game, 7, None, false).unwrap();
        fn kind(command: &Command) -> usize {
            match command {
                Command::Recruit { kind, .. } => *kind,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Advance { .. }
                | Command::SetUnitIntent { .. }
                | Command::EndTurn => panic!("expected specialist recruitment"),
            }
        }
        assert_eq!(rules.units[kind(&command)].family, Family::Adept);
        game.execute(command).unwrap();
        assert_eq!(game.required_recruit(Side::Player), None);
    }

    #[test]
    fn escalated_refuge_targets_patrol_and_terminal_attack_requires_its_last_enemy() {
        let rules = Rules::load().unwrap();
        let mut game =
            Game::start_campaign(&rules, "ashen-oath", 1, vec![], 0, vec![], vec![], vec![]);
        game.escalate_refuge();
        let required = game
            .objectives(Side::Player)
            .into_iter()
            .find(|g| g.id == "required:0")
            .unwrap();
        assert!(!required.tiles.iter().any(|t| t.as_str() == "6,4"));
        assert!(
            required
                .tiles
                .iter()
                .all(|t| game.at(game.site(t)).is_some_and(|u| u.side == Side::Enemy))
        );
        let actor = game
            .units
            .iter()
            .find(|u| u.side == Side::Player)
            .unwrap()
            .clone();
        let target = game
            .units
            .iter()
            .find(|u| u.side == Side::Enemy && rules.units[u.kind].leader)
            .unwrap()
            .clone();
        assert!(!crate::ai::campaigner::mission::terminal_attack(
            &game, &actor, &target
        ));
        game.units
            .retain(|u| u.side == Side::Player || u.id == target.id);
        game.roster_changed();
        assert!(crate::ai::campaigner::mission::terminal_attack(
            &game, &actor, &target
        ));
    }
}
