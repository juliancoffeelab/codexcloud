//! Route policies are authored in the simulation catalog, never inferred from missions.
use crate::{data::*, logic::Game};
pub(super) fn player_goals(game: &Game<'_>, route: &str) -> Option<Vec<Objective>> {
    if game.turn() != Side::Player {
        return None;
    }
    let campaign = game.campaign_state()?;
    let plan = game
        .catalogue()
        .campaign_defaults
        .campaigns
        .get(&campaign.id)?
        .routes
        .get(route)?
        .stages
        .get(game.map_id())?;
    if !plan
        .ignore_optional_when
        .as_ref()
        .is_some_and(|condition| game.event_condition_met(condition))
    {
        return None;
    }
    Some(
        crate::ai::campaigner::authored_goals(
            game.campaign_view(),
            game.phase_forecasts(),
            Side::Player,
        )
        .into_iter()
        .filter(|goal| !goal.id.starts_with("optional:"))
        .collect(),
    )
}
