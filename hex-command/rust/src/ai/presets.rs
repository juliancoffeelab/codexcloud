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
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::DropItem { .. }
                | Command::SkipDialogue => false,
            }
        }
        boundary(command)
    }

    #[test]
    fn player_and_enemy_rebuilding_emit_explicit_hold_targets() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        game.units.retain(|u| u.is_leader());
        game.roster_changed();
        for side in [Side::Player, Side::Enemy] {
            game.begin_turn(side);
            game.gold[side.faction_id()] = 100;
            let leader = game.units.iter().find(|u| u.side == side).unwrap().id;
            let workspace = PlanningWorkspace::new(7, 0, None, Diplomacy::Diplomatic);
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
                assert!(crate::ai::policies::objectives::actor_matches(&goal.actor, leader));
                assert_eq!(goal.hold, Some(true));
            }
        }
    }

    #[test]
    fn independent_blocks_keep_the_selected_planner_under_another_preset() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let blocks = Policy::Balanced.blocks();
        let mut custom =
            PlanningWorkspace::new(7, 1, None, Diplomacy::Diplomatic).with_blocks(blocks);
        let mut preset = PlanningWorkspace::new(7, 1, None, Diplomacy::Diplomatic);
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
    fn escalated_refuge_targets_patrol_and_terminal_attack_requires_its_last_enemy() {
        let rules = Rules::load().unwrap();
        let mut game =
            Game::start_campaign(&rules, "ashen-oath", 1, vec![], 0, vec![], vec![], vec![]);
        let state = game.campaign.as_mut().unwrap();
        state.set_flag("peace-lost", true, 1);
        let scripts = game.scripts.as_mut().unwrap();
        scripts.active.retain(|id| id != "refuge-peace");
        scripts.failed.push("refuge-peace".into());
        scripts.index_active(&game.mission.as_ref().unwrap().scripted_events);
        let required = game
            .objectives(Side::Player)
            .into_iter()
            .find(|g| g.id == "required:refuge-war")
            .unwrap();
        assert!(!required.tiles.contains(&"hex-0923-ec307e92cf".to_owned()));
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
            .find(|u| u.side == Side::Enemy && u.is_leader())
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
