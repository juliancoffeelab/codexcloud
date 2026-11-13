//! Independently selectable AI blocks. Policy names select fixed presets.
use super::{CampaignBlock, Planner, Policy, Recruiter};
use super::{campaigner::CampaignerKind, planner::PlannerKind, recruiter::RecruiterKind};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
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
    use crate::ai::campaigner::mission::{CampaignInput, MissionPolicy};
    use crate::ai::recruiter::fixtures;
    use crate::{
        ai::{Diplomacy, PlanningWorkspace, choose_game_command},
        data::*,
        logic::Game,
    };
    use hex_command_battle::RuntimeCommand as Command;

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
        fixtures::units(&mut game, |units| units.retain(|u| u.is_leader()));
        for faction in ["player", "enemy"] {
            let phase = game.faction(faction).unwrap().phase;
            game.begin_turn(phase);
            fixtures::edit(&mut game, |state| state.gold[faction] = 100);
            game.set_active_faction(Some(faction.into()));
            let leader = game
                .units()
                .iter()
                .find(|u| game.belongs_to_faction(u, faction))
                .unwrap()
                .id;
            let mission = MissionPolicy::new(game.campaign_view(), game.phase_forecasts());
            let orders = CampaignBlock::Mission.campaign_orders(
                &mission,
                CampaignInput {
                    seed: 0,
                    goals: None,
                    diplomacy: Diplomacy::Diplomatic,
                },
            );
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
                assert!(crate::ai::policies::objectives::actor_matches(
                    &goal.actor,
                    leader
                ));
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

            if end_turn(&command) {
                return;
            }
            game.execute_runtime(command).unwrap();
        }
        panic!("queued phase failed to finish");
    }
}
