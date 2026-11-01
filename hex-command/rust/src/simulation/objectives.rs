//! Route policies are authored in the simulation catalog, never inferred from missions.
use crate::{data::*, logic::Game};
pub(super) fn player_goals(game: &Game<'_>, route: &str) -> Option<Vec<Objective>> {
    if game.turn != Side::Player {
        return None;
    }
    let campaign = game.campaign.as_ref()?;
    let plan = game
        .rules
        .campaign_defaults
        .objectives
        .get(&campaign.id)?
        .get(route)?
        .stages
        .get(&game.map_id)?;
    if !plan
        .ignore_optional_when
        .as_ref()
        .is_some_and(|condition| game.event_condition_met(condition))
    {
        return None;
    }
    Some(
        game.objectives(Side::Player)
            .into_iter()
            .filter(|goal| !goal.id.starts_with("optional:"))
            .collect(),
    )
}
