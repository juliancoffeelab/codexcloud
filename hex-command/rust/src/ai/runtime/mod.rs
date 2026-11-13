//! The turn driver owns execution capabilities and the orchestration continuation.
use hex_command_battle::RuntimeCommand as Command;
pub(crate) mod context;
mod state;
pub use state::PlanningWorkspace;
pub(crate) use state::TurnConfig;
pub(in crate::ai) use state::{PlannerParts, PlannerTurn};

pub(crate) mod tactic;
#[cfg(not(target_arch = "wasm32"))]
use crate::ai::{Blocks, Diplomacy};
use crate::{ai::Policy, data::*, logic::Game};

#[cfg(any(feature = "wasm", test))]
pub(crate) fn choose_game_command(
    game: &Game<'_>,
    policy: Policy,
    workspace: &mut PlanningWorkspace,
) -> Result<Command, CommandError> {
    workspace.refresh_scope(game);
    workspace.state.proposal_faction = game.active_faction().map(str::to_owned);
    if !game.factions().is_empty() && game.active_faction().is_none() {
        let controllers = workspace
            .state
            .controller_factions
            .get_or_insert_with(|| automatic_factions(game))
            .clone();
        while workspace.state.controller_index < controllers.len() {
            let (faction, authored_policy) = &controllers[workspace.state.controller_index];
            let selected = authored_policy.unwrap_or(policy);
            let child = workspace.state.controller_workspace.get_or_insert_with(|| {
                Box::new(PlanningWorkspace::new(
                    workspace
                        .state
                        .seed
                        .wrapping_add(workspace.state.controller_index as u32),
                    workspace.state.recruitment_remaining,
                    workspace.state.goals.clone(),
                    workspace.state.diplomacy,
                ))
            });
            if !child.state.initialized && authored_policy.is_none() {
                child.state.blocks = workspace.state.blocks;
            }
            let mut scoped = game.fork();
            scoped.set_active_faction(Some(faction.clone()));
            let command = choose_game_command(&scoped, selected, child)?;
            if command == Command::EndTurn {
                workspace.state.controller_index += 1;
                workspace.state.controller_workspace = None;
            } else {
                workspace.state.proposal_faction = Some(faction.clone());
                return Ok(command);
            }
        }
        return Ok(Command::EndTurn);
    }
    workspace.refresh_control(game);
    let blocks = workspace.effective_blocks(policy);
    if !blocks.campaign.supports_campaign() && game.is_campaign() {
        return Err(CommandError::BattleOnlyPolicy);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _profile = crate::ai::profiling::Decision::start(policy, game.turn());
    workspace.state.initialized = true;
    workspace.state.blocks.get_or_insert(blocks);
    if game.winner().is_some() {
        return Ok(Command::EndTurn);
    }
    if !workspace.state.turn_plan_ready {
        let mut tactic = tactic::Tactic::new(blocks.recruiter, blocks.campaign, blocks.planner);
        let plan = tactic.run_turn(game, workspace)?;
        workspace.state.report = plan.planning;
        workspace.state.commands = plan
            .commands
            .into_iter()
            .map(|proposal| proposal.into_command())
            .collect();
        workspace.state.turn_plan_ready = true;
    }
    let command = workspace
        .state
        .commands
        .pop_front()
        .unwrap_or(Command::EndTurn);
    if command.is_attack() {
        workspace.state.turn_plan_ready = false;
    }
    Ok(command)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn play_blocks(
    game: &mut Game<'_>,
    policy: Policy,
    blocks: Option<Blocks>,
    seed: u32,
    limit: impl Into<Option<usize>>,
    goals: Option<&[Objective]>,
    diplomacy: Diplomacy,
) -> Result<(), CommandError> {
    let limit = limit.into();
    if !game.factions().is_empty() && game.active_faction().is_none() {
        let controllers = automatic_factions(game);
        for (index, (faction, authored_policy)) in controllers.into_iter().enumerate() {
            if game.winner().is_some() {
                break;
            }
            let selected = authored_policy.unwrap_or(policy);
            let selected_blocks = if authored_policy.is_none() {
                blocks
            } else {
                None
            };
            game.set_active_faction(Some(faction));
            let result = play_blocks(
                game,
                selected,
                selected_blocks,
                seed.wrapping_add(index as u32),
                limit,
                goals,
                diplomacy,
            );
            game.set_active_faction(None);
            result?;
        }
        return Ok(());
    }
    let blocks = blocks.unwrap_or_else(|| policy.blocks());
    if !blocks.campaign.supports_campaign() && game.is_campaign() {
        return Err(CommandError::BattleOnlyPolicy);
    }
    if game.winner().is_some() {
        return Ok(());
    }
    let mut workspace = PlanningWorkspace::new(
        seed,
        limit.or_else(|| policy.default_recruitment_limit()),
        goals.map(<[Objective]>::to_vec),
        diplomacy,
    );
    let mut tactic = tactic::Tactic::new(blocks.recruiter, blocks.campaign, blocks.planner);
    loop {
        let plan = {
            let _profile = crate::ai::profiling::Decision::start(policy, game.turn());
            tactic.run_turn(game, &mut workspace)?
        };
        execute_proposals(game, &mut workspace, plan.commands)?;
        if workspace.finished() || game.winner().is_some() {
            break;
        }
    }
    Ok(())
}
fn automatic_factions(game: &Game<'_>) -> Vec<(String, Option<Policy>)> {
    game.automatic_factions()
        .into_iter()
        .map(|(id, policy)| (id, policy.map(Into::into)))
        .collect()
}

/// Shared execution boundary for native simulation and ordinary native play.
/// Browser play admits one proposal through the same live engine and refreshes
/// its continuation before handing out the next command.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn execute_proposals(
    game: &mut Game<'_>,
    workspace: &mut PlanningWorkspace,
    proposals: Vec<hex_command_battle::ActionProposal>,
) -> Result<(), CommandError> {
    for proposal in proposals {
        if game.winner().is_some() {
            break;
        }
        let command = proposal.into_command();
        if game.active_faction().is_some() && command == Command::EndTurn {
            continue;
        }
        let outcome = game.execute_proposal(command.into())?;
        let changed = workspace.refresh_control(game);
        match outcome {
            hex_command_battle::ProposalOutcome::Finished
            | hex_command_battle::ProposalOutcome::Replan => break,
            hex_command_battle::ProposalOutcome::Applied if changed => break,
            hex_command_battle::ProposalOutcome::Applied => {}
        }
    }
    Ok(())
}
