//! Compound rebuild-move decisions combine the shared estimate with route and risk policies.
use crate::ai::policies::threat::{Threats, cap_exposure};
use crate::{
    ai::{context::BattleView, orders::TacticalOrders, planner::proposal::Plan},
    data::*,
};

pub(super) fn plan(
    game: &BattleView<'_, '_>,
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    unit: &Unit,
    orders: &TacticalOrders,
    opportunity: Option<crate::ai::policies::recruitment::EstimateInput<'_>>,
) -> Option<Plan> {
    let opportunity = opportunity?;
    if !unit.is_leader()
        || unit.moved
        || unit.attacked
        || !game.can_act(unit.id)
        || opportunity.remaining_budget == Some(0)
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
    let mut threats = Threats::protected(phases, unit.side, orders);
    let before = cap_exposure(unit, threats.at(unit, unit.tile, None), true);
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
        .find(|g| crate::ai::policies::objectives::actor_matches(&g.actor, unit.id))?;
    let route = game.routes(unit.kind, &keeps);
    let mut best: Option<Plan> = None;
    for tile in destinations
        .into_iter()
        .filter(|tile| keeps.contains(tile) && *tile != unit.tile)
    {
        let Some(value) =
            crate::ai::policies::recruitment::estimate_capacity(game, unit, tile, opportunity)
        else {
            continue;
        };
        let after = cap_exposure(unit, threats.at(unit, tile, None), true);
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
