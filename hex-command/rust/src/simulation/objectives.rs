//! Route policies are authored in the simulation catalog, never inferred from missions.
use crate::{data::*, logic::BattleSession};
pub(super) fn player_goals(game: &BattleSession<'_>, route: &str) -> Option<Vec<Objective>> {
    if game.turn() != TurnPhase::Player {
        return None;
    }
    let campaign = game.campaign_state()?;
    let plan = game
        .catalogue()
        .meta()
        .simulation
        .campaign
        .campaigns
        .get(&campaign.id)?
        .routes
        .get(route)?
        .stages
        .get(game.map_id())?;
    if !plan
        .ignore_optional_when
        .as_ref()
        .is_some_and(|condition| game.event_condition_met(condition, Some("player")))
    {
        return None;
    }
    Some(
        crate::ai::campaigner::authored_goals(game.campaign_view(), game.phase_forecasts())
            .into_iter()
            .filter(|goal| !goal.id.starts_with("optional:"))
            .collect(),
    )
}
