//! Route policies are authored in the simulation catalog, never inferred from missions.
use crate::{data::*, logic::Game};
pub(super) fn player_goals(game: &Game<'_>, route: &str) -> Option<Vec<Objective>> {
    if game.turn() != Side::Player {
        return None;
    }
    let authored = game.campaign_view();
    let campaign = authored.campaign()?;
    let plan = game
        .catalogue()
        .campaign_defaults
        .objectives
        .get(&campaign.id)?
        .get(route)?
        .stages
        .get(game.map_id())?;
    if !plan
        .ignore_optional_when
        .as_ref()
        .is_some_and(|condition| authored.event_condition_met(condition))
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
