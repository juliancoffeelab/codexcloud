//! Borrowed block inputs. Only CampaignView exposes the authored mission view.
use super::{CampaignBlock, Planner, PlanningWorkspace, Recruiter, orders::*};
use crate::{data::*, logic::Game};

pub(super) struct CampaignView<'a, 'catalog> {
    game: &'a Game<'catalog>,
    workspace: &'a PlanningWorkspace,
}
impl<'a, 'catalog> CampaignView<'a, 'catalog> {
    pub(super) fn new(game: &'a Game<'catalog>, workspace: &'a PlanningWorkspace) -> Self {
        Self { game, workspace }
    }
}
pub(super) struct RecruitmentContext<'a, 'catalog> {
    game: &'a Game<'catalog>,
    seed: u32,
}
impl<'a, 'catalog> RecruitmentContext<'a, 'catalog> {
    pub(super) fn new(game: &'a Game<'catalog>, seed: u32) -> Self {
        Self { game, seed }
    }
}
pub(super) struct TacticalContext<'a, 'catalog> {
    game: &'a Game<'catalog>,
    workspace: &'a mut PlanningWorkspace,
}
impl<'a, 'catalog> TacticalContext<'a, 'catalog> {
    pub(super) fn new(game: &'a Game<'catalog>, workspace: &'a mut PlanningWorkspace) -> Self {
        Self { game, workspace }
    }
}
pub(super) trait Campaigner {
    fn objectives(&self, view: CampaignView<'_, '_>) -> CampaignOrders;
}
pub(super) trait RecruitmentStrategy {
    fn recruit(
        &self,
        context: RecruitmentContext<'_, '_>,
        orders: &RecruitmentOrders,
    ) -> Option<Command>;
}
pub(super) trait TacticalPlanner {
    fn plan(&self, context: TacticalContext<'_, '_>) -> Result<(), CommandError>;
}
impl Campaigner for CampaignBlock {
    fn objectives(&self, view: CampaignView<'_, '_>) -> CampaignOrders {
        self.campaign_orders(view.game, view.workspace)
    }
}
impl RecruitmentStrategy for Recruiter {
    fn recruit(
        &self,
        context: RecruitmentContext<'_, '_>,
        orders: &RecruitmentOrders,
    ) -> Option<Command> {
        self.choose_orders(context.game, context.seed, orders)
    }
}
impl TacticalPlanner for Planner {
    fn plan(&self, context: TacticalContext<'_, '_>) -> Result<(), CommandError> {
        self.choose(context.game, context.workspace)
    }
}
