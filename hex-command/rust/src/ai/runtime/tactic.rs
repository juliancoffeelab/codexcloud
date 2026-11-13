//! Fixed block composition. Each turn uses one draft and returns engine commands.
use super::context::{CampaignView, PlanningContext};
use crate::ai::{
    PlanningWorkspace, campaigner::Campaigner, orders::TurnPlan, planner::Planner,
    recruiter::Recruiter,
};
use crate::logic::{CommandError, Game};

pub(crate) struct Tactic<R, C, P> {
    recruiter: R,
    campaigner: C,
    planner: P,
}
impl<R: Recruiter, C: Campaigner, P: Planner> Tactic<R, C, P> {
    pub(crate) fn new(recruiter: R, campaigner: C, planner: P) -> Self {
        Self {
            recruiter,
            campaigner,
            planner,
        }
    }
    pub(crate) fn run_turn(
        &mut self,
        game: &Game<'_>,
        workspace: &mut PlanningWorkspace,
    ) -> Result<TurnPlan, CommandError> {
        workspace.refresh_control(game);
        let mut orders = workspace.state.orders.clone().unwrap_or_else(|| {
            self.campaigner
                .objectives(CampaignView::new(game, workspace.config()))
        });
        let mut draft = PlanningContext::fork(game);
        draft.restrict_actors(&orders.tactics.inactive);
        draft.restrict_positions(&orders.tactics.scoped);
        let mut recruitment_context =
            draft.recruitment_context(workspace.state.seed, workspace.state.recruitment_remaining);
        let recruitment = self
            .recruiter
            .recruit(&mut recruitment_context, &orders.recruitment)?;
        orders.tactics.promotions = self
            .recruiter
            .promotions(recruitment_context.battle(), workspace.state.seed);
        #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
        {
            let observed = workspace
                .state
                .observation_orders
                .get_or_insert_with(|| orders.tactics.clone());
            observed.promotions = orders.tactics.promotions.clone();
        }
        workspace.state.recruitment_remaining = if self.planner.recruits_after_move() {
            workspace
                .state
                .recruitment_remaining
                .map(|left| left.saturating_sub(recruitment.recruited.len()))
        } else {
            Some(0)
        };
        workspace.state.recruitment_opportunity = if self.planner.recruits_after_move() {
            self.recruiter
                .kind()
                .map(|kind| super::context::RecruitmentOpportunity {
                    kind,
                    seed: workspace.state.seed,
                    budget: workspace.state.recruitment_remaining,
                    orders: orders.recruitment.clone(),
                })
        } else {
            None
        };
        workspace.state.orders = Some(orders.clone());
        workspace
            .state
            .report
            .mandatory_recruits
            .extend(recruitment.mandatory.iter().copied());
        let planning = self
            .planner
            .plan(&mut draft.tactical_context(workspace), &orders.tactics)?;
        #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
        if workspace.finished() {
            draft.record_stop(
                workspace
                    .state
                    .observation_orders
                    .as_ref()
                    .expect("observation orders"),
            );
        }
        Ok(draft.finish(recruitment, planning))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type PresetTactic =
    Tactic<crate::ai::Recruiter, crate::ai::CampaignBlock, crate::ai::Planner>;
#[cfg(not(target_arch = "wasm32"))]
impl PresetTactic {
    #[cfg(test)]
    pub(crate) fn preset(policy: crate::ai::Policy) -> Self {
        Self::from_blocks(policy.blocks())
    }
    pub(crate) fn from_blocks(blocks: crate::ai::Blocks) -> Self {
        Self::new(blocks.recruiter, blocks.campaign, blocks.planner)
    }
    pub(crate) fn composition(&self) -> crate::ai::Blocks {
        crate::ai::Blocks {
            recruiter: self.recruiter,
            campaign: self.campaigner,
            planner: self.planner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ai::{Diplomacy, Recruiter, context::TacticalContext, orders::*},
        data::*,
    };
    use std::{cell::Cell, rc::Rc};

    struct Orders {
        requirements: Vec<RecruitRequirement>,
        calls: Rc<Cell<usize>>,
    }
    impl Campaigner for Orders {
        fn objectives(&mut self, _view: CampaignView<'_, '_>) -> CampaignOrders {
            self.calls.set(self.calls.get() + 1);
            CampaignOrders {
                recruitment: RecruitmentOrders {
                    requirements: self.requirements.clone(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    }
    struct Observe {
        first_new_id: u32,
        initial_gold: i32,
        calls: Rc<Cell<usize>>,
    }
    impl Planner for Observe {
        fn plan(
            &mut self,
            context: &mut TacticalContext<'_, '_>,
            _orders: &TacticalOrders,
        ) -> Result<PlanningReport, CommandError> {
            let view = context.battle();
            let recruits: Vec<_> = view
                .units()
                .iter()
                .filter(|unit| unit.id >= self.first_new_id)
                .collect();
            assert_eq!(recruits.len(), 2);
            assert!(recruits.iter().all(|unit| unit.moved && unit.attack_spent));
            assert!(view.gold_for_phase(view.turn()) < self.initial_gold);
            self.calls.set(self.calls.get() + 1);
            Ok(PlanningReport::default())
        }
    }
    #[test]
    fn recruitment_and_planning_share_a_validated_draft_with_fixed_components() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        let mut checkpoint = game.state();
        checkpoint.units.retain(|unit| {
            game.units()
                .iter()
                .any(|u| format!("u{}", u.id) == unit.id && u.is_leader())
        });
        checkpoint.gold[game.turn().faction_id()] = 1000;
        game = Game::try_from_state(&rules, checkpoint).unwrap();
        let kind = game.recruitment_options(game.turn())[0];
        let calls = Rc::new(Cell::new(0));
        let mut tactic = Tactic::new(
            Recruiter::Rotation,
            Orders {
                requirements: vec![
                    RecruitRequirement::EnsureUnits {
                        selector: UnitSelector::Type(UnitTypeId(rules.units[kind].id.clone())),
                        count: 1,
                    },
                    RecruitRequirement::EnsureUnits {
                        selector: UnitSelector::Family(rules.units[kind].family),
                        count: 1,
                    },
                ],
                calls: calls.clone(),
            },
            Observe {
                first_new_id: game.state().next_id,
                initial_gold: 1000,
                calls: calls.clone(),
            },
        );
        for _ in 0..2 {
            let mut workspace = PlanningWorkspace::new(7, 2, None, Diplomacy::Diplomatic);
            let plan = tactic.run_turn(&game, &mut workspace).unwrap();
            assert_eq!(plan.recruitment.recruited.len(), 2);
            assert_eq!(plan.recruitment.mandatory.len(), 1);
            assert!(plan.recruitment.requirements.is_empty());
            assert_eq!(game.gold_for_phase(game.turn()), 1000);
            let mut replay = game.fork();
            for command in plan.commands {
                replay.execute_proposal(command).unwrap();
            }
            assert_eq!(replay.units().len(), game.units().len() + 2);
        }
        assert_eq!(calls.get(), 4);
    }
}
