//! Compound move-and-recruit alternatives for search planners.
use super::safety::Threats;
use crate::{
    ai::{
        context::BattleView, orders::TacticalOrders, recruiter::RecruitmentOpportunity,
        tactics::Plan,
    },
    data::*,
};

/// Exposure beyond the actor's remaining life cannot represent an additional loss.
pub(super) fn exposure(unit: &Unit, incoming: i32, important: bool) -> i32 {
    if important {
        incoming.min(unit.hp.max(1))
    } else {
        incoming
    }
}

pub(super) fn plan(
    game: &BattleView<'_, '_>,
    unit: &Unit,
    orders: &TacticalOrders,
    opportunity: Option<&RecruitmentOpportunity>,
) -> Option<Plan> {
    let opportunity = opportunity?;
    if !unit.is_leader()
        || unit.moved
        || unit.attacked
        || !game.can_act(unit.id)
        || opportunity.budget == Some(0)
    {
        return None;
    }
    let (keeps, _) = orders.rebuild_plan(game, unit.id, orders.moving_actor(unit.id));
    if keeps.is_empty() {
        return None;
    }
    let destinations = orders.authored_destinations(
        game,
        unit,
        game.reachable(unit).iter().map(|&(tile, _)| tile).collect(),
    );
    let mut threats = Threats::protected(game, unit.side, orders);
    let before = exposure(unit, threats.at(unit, unit.tile, None), true);
    let danger = if orders.protects(unit.id) {
        40.0 / unit.hp.max(1) as f64
    } else {
        6.0
    };
    let risk_cost =
        |incoming: i32| incoming as f64 * danger + if incoming >= unit.hp { 40.0 } else { 0.0 };
    let rebuilding = orders.rebuild_objectives(game);
    let goal = rebuilding
        .iter()
        .find(|g| crate::ai::objectives::actor_matches(&g.actor, unit.id))?;
    let route = game.routes(unit.kind, &keeps);
    let mut best: Option<Plan> = None;
    for tile in destinations
        .into_iter()
        .filter(|tile| keeps.contains(tile) && *tile != unit.tile)
    {
        let gold = game.gold_for_unit(unit).max(0);
        let requirement = opportunity
            .orders
            .requirements
            .iter()
            .find(|r| r.deficit(game) > 0);
        let mut slots = 0;
        let mut costs = Vec::new();
        for kind in game.recruitment_options_for(unit) {
            if requirement.is_some_and(|r| !r.accepts(game, kind)) {
                continue;
            }
            let cost = game.recruit_cost(unit, kind);
            if cost > gold {
                continue;
            }
            let available = game.board().adjacent[tile]
                .iter()
                .filter(|&&site| {
                    (site == unit.tile || game.at(site).is_none())
                        && game.cost(kind, site) != u16::MAX
                        && !opportunity
                            .orders
                            .reserved
                            .iter()
                            .any(|reserved| game.site(&reserved.0) == site)
                })
                .count();
            if available > 0 {
                slots = slots.max(available);
                costs.push(cost);
            }
        }
        if costs.is_empty() {
            continue;
        }
        let average = costs.iter().sum::<i32>() / costs.len() as i32;
        let capacity = slots.min(opportunity.budget.unwrap_or(usize::MAX));
        let value = gold.min((capacity as i32).saturating_mul(average));
        let after = exposure(unit, threats.at(unit, tile, None), true);
        // Reaching the existing rebuilding objective, plus a budget/placement estimate of new army value.
        let score = route[unit.tile] as f64 * goal.weight / 4.0
            + goal.weight * 0.6
            + value as f64
            + risk_cost(before)
            - risk_cost(after)
            + (before - after).max(0) as f64 * 3.0;
        if score > 0.1 && best.as_ref().is_none_or(|p| score > p.score) {
            best = Some(Plan {
                unit: unit.id,
                tile,
                score,
                target: None,
                weapon: None,
                advance: None,
                withdrawing: false,
            });
        }
    }
    best
}
