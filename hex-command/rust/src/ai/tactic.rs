//! Fixed block composition. Each turn uses one draft and returns engine commands.
use super::{
    PlanningWorkspace,
    campaigner::Campaigner,
    context::{CampaignView, PlanningContext},
    orders::TurnPlan,
    planner::Planner,
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
        let mut orders = workspace.orders.clone().unwrap_or_else(|| {
            self.campaigner
                .objectives(CampaignView::new(game, workspace))
        });
        let mut draft = PlanningContext::fork(game);
        let mut recruitment_context =
            draft.recruitment_context(workspace.seed, workspace.recruitment_remaining);
        let recruitment = self
            .recruiter
            .recruit(&mut recruitment_context, &orders.recruitment)?;
        orders.tactics.promotions = self.recruiter.promotions(recruitment_context.battle());
        workspace.recruitment_remaining = Some(0);
        workspace.orders = Some(orders.clone());
        workspace
            .report
            .mandatory_recruits
            .extend(recruitment.mandatory.iter().copied());
        let planning = self
            .planner
            .plan(&mut draft.tactical_context(workspace), &orders.tactics)?;
        Ok(draft.finish(recruitment, planning))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type PresetTactic = Tactic<super::Recruiter, super::CampaignBlock, super::Planner>;
#[cfg(not(target_arch = "wasm32"))]
impl PresetTactic {
    #[cfg(test)]
    pub(crate) fn preset(policy: super::Policy) -> Self {
        Self::from_blocks(policy.blocks())
    }
    pub(crate) fn from_blocks(blocks: super::Blocks) -> Self {
        Self::new(blocks.recruiter, blocks.campaign, blocks.planner)
    }
    pub(crate) fn composition(&self) -> super::Blocks {
        super::Blocks {
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
            assert!(recruits.iter().all(|unit| unit.moved && unit.attacked));
            assert!(view.gold()[view.turn().index()] < self.initial_gold);
            self.calls.set(self.calls.get() + 1);
            Ok(PlanningReport::default())
        }
    }
    #[test]
    fn recruitment_and_planning_share_a_validated_draft_with_fixed_components() {
        let rules = Rules::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        game.units.retain(|unit| rules.units[unit.kind].leader);
        game.roster_changed();
        game.gold[game.turn.index()] = 1000;
        let kind = game.recruitment_options(game.turn)[0];
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
                first_new_id: game.next_id,
                initial_gold: 1000,
                calls: calls.clone(),
            },
        );
        for _ in 0..2 {
            let mut workspace = PlanningWorkspace::new(7, 2, None, Diplomacy::Restrained);
            let plan = tactic.run_turn(&game, &mut workspace).unwrap();
            assert_eq!(plan.recruitment.recruited.len(), 2);
            assert_eq!(plan.recruitment.mandatory.len(), 1);
            assert!(plan.recruitment.requirements.is_empty());
            assert_eq!(game.gold[game.turn.index()], 1000);
            let mut replay = game.fork();
            for command in plan.commands {
                replay.execute(command).unwrap();
            }
            assert_eq!(replay.units.len(), game.units.len() + 2);
        }
        assert_eq!(calls.get(), 4);
    }
}
