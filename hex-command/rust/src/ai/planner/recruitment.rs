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
        let mut draft = game.fork();
        if draft
            .execute(crate::ai::move_command(game, unit.id, tile))
            .is_err()
        {
            continue;
        }
        let first_id = draft.next_id();
        let Ok((_, report)) = draft.recruit(opportunity) else {
            continue;
        };
        if report.recruited.is_empty() {
            continue;
        }
        let value: i32 = draft
            .units()
            .iter()
            .filter(|u| u.id >= first_id)
            .map(|u| draft.recruit_cost(u, u.kind))
            .sum();
        let actor = draft.unit(unit.id)?;
        let mut after_threats = Threats::protected(&draft, actor.side, orders);
        let after = exposure(actor, after_threats.at(actor, tile, None), true);
        // Reaching the existing rebuilding objective, plus actual purchased army value.
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
