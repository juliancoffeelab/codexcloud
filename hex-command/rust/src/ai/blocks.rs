//! Independently selectable AI blocks. Policy names are compatibility presets.
use super::{CampaignBlock, Planner, Policy, Recruiter};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Blocks {
    pub recruiter: Recruiter,
    pub campaign: CampaignBlock,
    pub planner: Planner,
}

impl Policy {
    /// Resolve a live preset. Historical policies use their frozen executable.
    pub const fn blocks(self) -> Option<Blocks> {
        match self {
            Self::Classic => Some(Blocks {
                recruiter: Recruiter::Rotation,
                campaign: CampaignBlock::Objectives,
                planner: Planner::Classic,
            }),
            Self::Rush => Some(Blocks {
                recruiter: Recruiter::Frontline,
                campaign: CampaignBlock::Objectives,
                planner: Planner::Rush,
            }),
            Self::Mixed => Some(Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::Objectives,
                planner: Planner::Mixed,
            }),
            Self::Priority => Some(Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::Adaptive,
                planner: Planner::Queued,
            }),
            Self::Battler => Some(Blocks {
                recruiter: Recruiter::Balanced,
                campaign: CampaignBlock::ArenaOnly,
                planner: Planner::Queued,
            }),
            Self::ClassicOld | Self::RushOld | Self::MixedOld => None,
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
    fn independent_blocks_survive_a_saved_queued_phase_under_another_preset() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let blocks = Policy::Priority.blocks().unwrap();
        let mut custom =
            PlanningWorkspace::new(7, 1, None, Diplomacy::Restrained).with_blocks(blocks);
        let mut preset = PlanningWorkspace::new(7, 1, None, Diplomacy::Restrained);
        for _ in 0..64 {
            let command = choose_game_command(&rules, &game, Policy::Classic, &mut custom).unwrap();
            assert_eq!(
                command,
                choose_game_command(&rules, &game, Policy::Priority, &mut preset).unwrap()
            );
            let saved = custom.snapshot();
            assert_eq!(saved.blocks, Some(blocks));
            saved.validate(&rules, &game.state()).unwrap();
            custom = PlanningWorkspace::from_snapshot(
                serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap(),
            );
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
        assert!(!crate::ai::campaign::terminal_attack(
            &game, &actor, &target
        ));
        game.units
            .retain(|u| u.side == Side::Player || u.id == target.id);
        game.roster_changed();
        assert!(crate::ai::campaign::terminal_attack(&game, &actor, &target));
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
