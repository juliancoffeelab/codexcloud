//! Independently selectable AI blocks. Policy names are compatibility presets.
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
    /// Resolve a live preset. Every policy has a complete composition.
    pub const fn blocks(self) -> Blocks {
        match self {
            Self::Classic => Blocks {
                recruiter: Recruiter::Rotation,
                campaign: CampaignBlock::Mission,
                planner: Planner::Classic,
            },
            Self::Rush => Blocks {
                recruiter: Recruiter::Frontline,
                campaign: CampaignBlock::Mission,
                planner: Planner::Full,
            },
            Self::Mixed => Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::Mission,
                planner: Planner::Full,
            },
            Self::Priority => Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::Mission,
                planner: Planner::Queued,
            },
            Self::Battler => Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::Arena,
                planner: Planner::Queued,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                | Command::SetUnitIntent { .. }
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue => false,
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
        let blocks = Policy::Priority.blocks();
        let mut custom =
            PlanningWorkspace::new(7, 1, None, Diplomacy::Restrained).with_blocks(blocks);
        let mut preset = PlanningWorkspace::new(7, 1, None, Diplomacy::Restrained);
        for _ in 0..64 {
            let command = choose_game_command(&rules, &game, Policy::Classic, &mut custom).unwrap();
            assert_eq!(
                command,
                choose_game_command(&rules, &game, Policy::Priority, &mut preset).unwrap()
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
        fn kind(command: &Command) -> &UnitTypeId {
            match command {
                Command::Recruit { kind, .. } => kind,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Advance { .. }
                | Command::SetUnitIntent { .. }
                | Command::EndTurn
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue => panic!("expected specialist recruitment"),
            }
        }
        assert_eq!(
            rules.units[game.kind(&kind(&command).0)].family,
            Family::Adept
        );
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
        assert!(!required.tiles.contains(&"6,4".to_owned()));
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

    #[test]
    fn escort_corridor_clears_a_short_arrival_instead_of_an_idle_detour() {
        let rules = Rules::load().unwrap();
        for policy in [
            Policy::Mixed,
            Policy::Classic,
            Policy::Rush,
            Policy::Priority,
        ] {
            let mut game = Game::start_campaign(
                &rules,
                "borrowed-tide",
                0,
                vec![],
                0,
                vec![],
                vec![],
                vec![],
            );
            game.units
                .retain(|u| u.side == Side::Player || rules.units[u.kind].leader);
            for (id, tile) in [
                (1, "7,4"),
                (2, "8,3"),
                (3, "8,2"),
                (4, "9,2"),
                (5, "7,5"),
                (6, "11,7"),
            ] {
                let tile = game.site(tile);
                game.units.iter_mut().find(|u| u.id == id).unwrap().tile = tile;
            }
            for (id, kind, hp) in [(3, "tidehunter", 22), (4, "tidesage", 12)] {
                let kind = game.kind(kind);
                let unit = game.units.iter_mut().find(|u| u.id == id).unwrap();
                unit.kind = kind;
                unit.hp = hp;
            }
            let mut screen = game.unit(2).unwrap().clone();
            for (id, kind, tile) in [
                (10, "reefguard", "7,6"),
                (12, "wavemender", "8,4"),
                (13, "merfolk-maceguard", "7,3"),
                (15, "reefguard", "6,3"),
            ] {
                screen.id = id;
                screen.kind = game.kind(kind);
                screen.tile = game.site(tile);
                screen.hp = rules.units[screen.kind].hp;
                game.units.push(screen.clone());
            }
            game.next_id = 16;
            for (side, tiles) in [
                (Side::Player, vec!["2,2", "6,2", "5,4", "8,5", "5,7"]),
                (Side::Enemy, vec!["9,3", "10,5"]),
            ] {
                for tile in tiles {
                    let tile = game.site(tile);
                    game.owners[tile] = Some(side);
                }
            }
            game.campaign
                .as_mut()
                .unwrap()
                .marks
                .push("event:nets".into());
            game.units
                .iter_mut()
                .find(|u| u.id == 6)
                .unwrap()
                .rebuilding = Some(true);
            game.roster_changed();
            for _ in 0..16 {
                let active = if game.turn == Side::Player {
                    policy
                } else {
                    Policy::Classic
                };
                game.play_policy(active, 1011, 0, None, Diplomacy::Restrained)
                    .unwrap();
                if game.winner.is_some() {
                    break;
                }
                game.end_turn();
            }
            assert_eq!(
                game.winner,
                Some(Side::Player),
                "{policy:?}: {:?}",
                game.units
            );
        }
    }

    #[test]
    fn every_live_campaign_planner_clears_a_blocked_vigil_and_holds_it() {
        let rules = Rules::load().unwrap();
        for policy in [
            Policy::Classic,
            Policy::Rush,
            Policy::Mixed,
            Policy::Priority,
        ] {
            let mut game =
                Game::start_campaign(&rules, "green-accord", 1, vec![], 0, vec![], vec![], vec![]);
            game.units.retain(|u| u.side == Side::Player);
            std::rc::Rc::make_mut(game.mission.as_mut().unwrap())
                .waves
                .clear();
            let mut screen = game.unit(3).unwrap().clone();
            for (id, kind, tile) in [(9, "boar", "5,3"), (10, "boar", "6,5"), (11, "wolf", "7,4")] {
                screen.id = id;
                screen.kind = game.kind(kind);
                screen.tile = game.site(tile);
                screen.hp = rules.units[screen.kind].hp;
                game.units.push(screen.clone());
            }
            for (id, tile) in [(1, "4,5"), (2, "4,4"), (3, "5,4"), (4, "5,5")] {
                let tile = game.site(tile);
                game.units.iter_mut().find(|u| u.id == id).unwrap().tile = tile;
            }
            game.next_id = 12;
            game.roster_changed();
            for _ in 0..16 {
                game.play_policy(policy, 1004, 0, None, Diplomacy::Restrained)
                    .unwrap();
                if game.winner.is_some() {
                    break;
                }
                game.end_turn();
            }
            assert_eq!(
                game.winner,
                Some(Side::Player),
                "{policy:?}: {:?}",
                game.units
            );
            assert_eq!(
                game.campaign
                    .as_ref()
                    .unwrap()
                    .vigil
                    .as_ref()
                    .unwrap()
                    .held_turns,
                3
            );
        }
    }
}
