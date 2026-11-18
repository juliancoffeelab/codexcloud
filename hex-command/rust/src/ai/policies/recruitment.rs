//! Shared recruitment-value estimate; it predicts spending capacity, not recruiter choices.
use crate::{
    ai::{context::BattleView, orders::RecruitmentOrders},
    data::*,
};

/// Evaluation has no recruiter policy, random seed or execution context.
#[derive(Clone, Copy)]
pub(in crate::ai) struct EstimateInput<'a> {
    pub orders: &'a RecruitmentOrders,
    pub remaining_budget: Option<usize>,
}

/// Preserve the common keep-value approximation used by current planners.
/// The actor's origin is treated as vacant, but movement events are not simulated.
/// Among affordable kinds satisfying the first unmet requirement, use the maximum
/// compatible slot count and truncated mean cost. A real recruiter may spend less.
pub(in crate::ai) fn estimate_capacity(
    game: &BattleView<'_, '_>,
    actor: &Unit,
    destination: usize,
    input: EstimateInput<'_>,
) -> Option<i32> {
    let gold = game.gold_for_unit(actor).max(0);
    let requirement = input
        .orders
        .requirements
        .iter()
        .find(|r| r.deficit(game) > 0);
    let mut slots = 0;
    let mut cost_sum = 0;
    let mut cost_count = 0;
    for kind in game.recruitment_pool_for(actor) {
        if requirement.is_some_and(|r| !r.accepts(game, kind)) {
            continue;
        }
        let cost = game.recruit_cost(actor, kind);
        if cost > gold {
            continue;
        }
        let available = game.board().adjacent[destination]
            .iter()
            .filter(|&&site| {
                (site == actor.tile || game.at(site).is_none())
                    && game.step_cost(&game.prospective(kind, actor.faction, site), site)
                        != u16::MAX
                    && !input
                        .orders
                        .reserved
                        .iter()
                        .any(|reserved| game.site(&reserved.0) == site)
            })
            .count();
        if available > 0 {
            slots = slots.max(available);
            cost_sum += cost;
            cost_count += 1;
        }
    }
    if cost_count == 0 {
        return None;
    }
    let average = cost_sum / cost_count;
    let capacity = slots.min(input.remaining_budget.unwrap_or(usize::MAX));
    Some(gold.min((capacity as i32).saturating_mul(average)))
}

/// Composition policy helpers, independent of Catalogue mechanics.
pub(crate) trait RecruitRolePolicy: Sized {
    const ALL: [Self; 4];
    fn index(self) -> usize;
    fn weight(self) -> f64;
    fn is_ranged(self) -> bool;
}
impl RecruitRolePolicy for UnitRole {
    const ALL: [Self; 4] = [Self::Warrior, Self::Flank, Self::Ranger, Self::Healer];
    fn index(self) -> usize {
        match self {
            Self::Warrior => 0,
            Self::Flank => 1,
            Self::Ranger => 2,
            Self::Healer => 3,
        }
    }
    fn weight(self) -> f64 {
        match self {
            Self::Warrior => 0.3,
            Self::Flank | Self::Ranger | Self::Healer => 0.2,
        }
    }
    fn is_ranged(self) -> bool {
        match self {
            Self::Ranger | Self::Healer => true,
            Self::Warrior | Self::Flank => false,
        }
    }
}
