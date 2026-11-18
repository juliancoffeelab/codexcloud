//! Independently selectable AI blocks. Policy names select fixed presets.
use super::{CampaignerKind, PlannerKind, Policy, RecruiterKind};
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
                recruiter: RecruiterKind::Classic,
                campaign: CampaignerKind::Mission,
                planner: PlannerKind::Classic,
            },
            Self::Rush => Blocks {
                recruiter: RecruiterKind::Frontline,
                campaign: CampaignerKind::Mission,
                planner: PlannerKind::Queued,
            },
            Self::Balanced => Blocks {
                recruiter: RecruiterKind::Balanced,
                campaign: CampaignerKind::Mission,
                planner: PlannerKind::Queued,
            },
            Self::Rotated => Blocks {
                recruiter: RecruiterKind::Rotation,
                campaign: CampaignerKind::Mission,
                planner: PlannerKind::Queued,
            },
            Self::Situational => Blocks {
                recruiter: RecruiterKind::Situational,
                campaign: CampaignerKind::Mission,
                planner: PlannerKind::Queued,
            },
            Self::Predictor => Blocks {
                recruiter: RecruiterKind::Predictor,
                campaign: CampaignerKind::Mission,
                planner: PlannerKind::Queued,
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
        logic::BattleSession,
    };
    use hex_command_battle::RuntimeCommand;

    fn end_turn(command: &RuntimeCommand) -> bool {
        fn boundary(command: &RuntimeCommand) -> bool {
            match command {
                RuntimeCommand::EndTurn => true,
                RuntimeCommand::Move { .. }
                | RuntimeCommand::Attack { .. }
                | RuntimeCommand::Recruit { .. }
                | RuntimeCommand::Advance { .. }
                | RuntimeCommand::SetUnitIntent { .. }
                | RuntimeCommand::DismissDialogue
                | RuntimeCommand::RecallMission { .. }
                | RuntimeCommand::RetryMission
                | RuntimeCommand::PreviousDialogue
                | RuntimeCommand::DropItem { .. }
                | RuntimeCommand::SkipDialogue => false,
            }
        }
        boundary(command)
    }

    #[test]
    fn player_and_enemy_rebuilding_emit_explicit_hold_targets() {
        let rules = Catalogue::load().unwrap();
        let mut game = BattleSession::new(&rules, "mirror");
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
            let orders = CampaignerKind::Mission.campaign_orders(
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
        let rules = Catalogue::load().unwrap();
        let mut game = BattleSession::new(&rules, "mirror");
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
