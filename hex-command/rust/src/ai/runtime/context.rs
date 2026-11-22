//! Runtime-owned execution contexts; evaluation cannot construct these capabilities.
use crate::ai::campaigner::mission::{CampaignInput, MissionPolicy};
use crate::{
    ai::{
        PlanningWorkspace,
        orders::*,
        recruiter::{Recruiter, RecruiterKind},
    },
    data::*,
    logic::BattleSession,
};
use hex_command_battle::RuntimeCommand;
use hex_command_battle::{ActionForecast, ActionForecastSource, BattleView, PhaseForecasts};

/// Execution-only recruiter information. Evaluators receive EstimateInput instead.
#[derive(Clone)]
pub(super) struct RecruitmentOpportunity {
    pub(super) kind: RecruiterKind,
    pub(super) seed: u32,
    pub(super) budget: Option<usize>,
    pub(super) orders: RecruitmentOrders,
}
fn recruit(
    draft: &mut ActionForecast<'_>,
    opportunity: &RecruitmentOpportunity,
) -> Result<(Vec<RuntimeCommand>, RecruitmentReport), CommandError> {
    let mut commands = Vec::new();
    let mut kind = opportunity.kind;
    let report = kind.recruit(
        &mut RecruitmentContext {
            draft,
            commands: &mut commands,
            seed: opportunity.seed,
            budget: opportunity.budget,
        },
        &opportunity.orders,
    )?;
    Ok((commands, report))
}

/// Read-only inputs supplied to the top-level Campaigner block.
pub(crate) struct CampaignerContext<'a, 'catalog> {
    mission: MissionPolicy<'a, 'catalog>,
    input: CampaignInput<'a>,
}
impl<'a, 'catalog> CampaignerContext<'a, 'catalog> {
    pub(in crate::ai) fn new(
        game: &'a BattleSession<'catalog>,
        config: super::TurnConfig<'a>,
    ) -> Self {
        Self {
            mission: MissionPolicy::new(game.campaign_view(), game.phase_forecasts()),
            input: CampaignInput {
                seed: config.seed,
                goals: config.goals,
                objective_progress: config.objective_progress,
                opponent_progress: config.opponent_progress,
                diplomacy: config.diplomacy,
            },
        }
    }
    pub(in crate::ai) fn orders(
        &self,
        kind: crate::ai::campaigner::CampaignerKind,
    ) -> CampaignOrders {
        kind.campaign_orders(&self.mission, self.input)
    }
}
/// Exactly one isolated engine state for recruitment and tactical execution.
pub(super) struct PlanningContext<'catalog> {
    draft: ActionForecast<'catalog>,
    commands: Vec<RuntimeCommand>,
    recruitment: RecruitmentReport,
}
impl<'catalog> PlanningContext<'catalog> {
    pub(super) fn fork(game: &BattleSession<'catalog>) -> Self {
        Self {
            draft: game.action_forecasts().fork(),
            commands: Vec::new(),
            recruitment: RecruitmentReport::default(),
        }
    }
    pub(super) fn restrict_positions(&mut self, scoped: &[ScopedOrder]) {
        self.draft.restrict_positions(scoped);
    }
    pub(super) fn restrict_actors(&mut self, inactive: &[UnitId]) {
        self.draft.restrict_actors(inactive);
    }
    pub(super) fn recruitment_context<'a>(
        &'a mut self,
        seed: u32,
        budget: Option<usize>,
    ) -> RecruitmentContext<'a, 'catalog> {
        RecruitmentContext {
            draft: &mut self.draft,
            commands: &mut self.commands,
            seed,
            budget,
        }
    }
    pub(super) fn tactical_context<'a>(
        &'a mut self,
        workspace: &'a mut PlanningWorkspace,
    ) -> TacticalContext<'a, 'catalog> {
        TacticalContext {
            draft: &mut self.draft,
            commands: &mut self.commands,
            workspace,
            recruitment: &mut self.recruitment,
            combat_boundary: false,
        }
    }
    #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
    pub(super) fn record_stop(&self, orders: &TacticalOrders) {
        if self.draft.winner().is_none() {
            crate::training::record(&self.draft, orders, &RuntimeCommand::EndTurn);
        }
    }
    pub(super) fn finish(
        self,
        mut recruitment: RecruitmentReport,
        planning: PlanningReport,
    ) -> TurnPlan {
        if !self.recruitment.recruited.is_empty() {
            recruitment.requirements = self.recruitment.requirements;
        }
        recruitment.recruited.extend(self.recruitment.recruited);
        recruitment.mandatory.extend(self.recruitment.mandatory);
        TurnPlan {
            commands: self.commands.into_iter().map(Into::into).collect(),
            recruitment,
            planning,
        }
    }
}
pub(crate) struct RecruitmentContext<'a, 'catalog> {
    draft: &'a mut ActionForecast<'catalog>,
    commands: &'a mut Vec<RuntimeCommand>,
    seed: u32,
    budget: Option<usize>,
}
impl<'context, 'catalog> RecruitmentContext<'context, 'catalog> {
    pub(in crate::ai) fn seed(&self) -> u32 {
        self.seed
    }
    pub(in crate::ai) fn budget(&self) -> Option<usize> {
        self.budget
    }
    pub(in crate::ai) fn battle(&self) -> &BattleView<'_, 'catalog> {
        self.draft
    }
    pub(in crate::ai) fn emit(&mut self, command: RuntimeCommand) -> Result<(), CommandError> {
        if !self.draft.permits_runtime(&command) {
            return Err(CommandError::IllegalIntent);
        }
        self.draft.execute_runtime(command.clone())?;
        self.commands.push(command);
        Ok(())
    }
}
pub(crate) struct TacticalContext<'a, 'catalog> {
    pub(super) combat_boundary: bool,
    recruitment: &'a mut RecruitmentReport,
    draft: &'a mut ActionForecast<'catalog>,
    commands: &'a mut Vec<RuntimeCommand>,
    workspace: &'a mut PlanningWorkspace,
}
impl<'context, 'catalog> TacticalContext<'context, 'catalog> {
    pub(in crate::ai) fn is_boundary(&self) -> bool {
        self.combat_boundary
    }

    pub(in crate::ai) fn turn_parts(&mut self) -> super::PlannerTurn<'_> {
        self.workspace.turn_parts()
    }
    pub(in crate::ai) fn classic_parts(
        &mut self,
    ) -> (
        &BattleView<'_, 'catalog>,
        PhaseForecasts<'_, 'catalog>,
        ActionForecastSource<'_, 'catalog>,
        super::PlannerParts<'_, crate::ai::planner::classic::ClassicPlannerSession>,
    ) {
        (
            &*self.draft,
            self.draft.phase_forecasts(),
            self.draft.action_forecasts(),
            self.workspace.classic_parts(),
        )
    }
    pub(in crate::ai) fn full_parts(
        &mut self,
    ) -> (
        &BattleView<'_, 'catalog>,
        PhaseForecasts<'_, 'catalog>,
        ActionForecastSource<'_, 'catalog>,
        super::PlannerParts<'_, crate::ai::planner::search::SearchPlannerSession>,
    ) {
        (
            &*self.draft,
            self.draft.phase_forecasts(),
            self.draft.action_forecasts(),
            self.workspace.full_parts(),
        )
    }
    pub(in crate::ai) fn queued_parts(
        &mut self,
    ) -> (
        &BattleView<'_, 'catalog>,
        PhaseForecasts<'_, 'catalog>,
        ActionForecastSource<'_, 'catalog>,
        super::PlannerParts<'_, crate::ai::planner::queued::QueuedPlannerSession>,
    ) {
        (
            &*self.draft,
            self.draft.phase_forecasts(),
            self.draft.action_forecasts(),
            self.workspace.queued_parts(),
        )
    }
    pub(in crate::ai) fn battle(&self) -> &BattleView<'_, 'catalog> {
        self.draft
    }
    #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
    pub(in crate::ai) fn observation_orders(&self) -> &TacticalOrders {
        self.workspace
            .state
            .observation_orders
            .as_ref()
            .expect("observation orders")
    }
    pub(in crate::ai) fn emit(&mut self, command: RuntimeCommand) -> Result<(), CommandError> {
        if !self.draft.permits_runtime(&command) {
            return Err(CommandError::IllegalIntent);
        }
        #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
        crate::training::record(
            self.draft,
            &self
                .workspace
                .state
                .observation_orders
                .as_ref()
                .expect("observation orders"),
            &command,
        );
        let before = self.draft.control_roster();
        self.draft.execute_runtime(command.clone())?;
        let control_changed =
            command != RuntimeCommand::EndTurn && before != self.draft.control_roster();
        self.combat_boundary = command.is_attack() || control_changed;
        if control_changed {
            self.workspace.state_mut().finished = false;
        }
        let moved = if let RuntimeCommand::Move { unit, .. } = &command {
            self.draft
                .unit(unit.0)
                .is_some_and(|u| u.is_leader() && self.draft.is_keep(u.tile))
        } else {
            false
        };
        self.commands.push(command);
        if moved {
            if let Some(opportunity) = &self.workspace.state_mut().recruitment_opportunity {
                let (commands, report) = recruit(self.draft, opportunity)?;
                self.workspace.state_mut().recruitment_remaining = self
                    .workspace
                    .state_mut()
                    .recruitment_remaining
                    .map(|left| left.saturating_sub(report.recruited.len()));
                self.workspace
                    .state_mut()
                    .recruitment_opportunity
                    .as_mut()
                    .unwrap()
                    .budget = self.workspace.state_mut().recruitment_remaining;
                self.workspace
                    .state_mut()
                    .report
                    .mandatory_recruits
                    .extend(report.mandatory.iter().copied());
                self.recruitment.recruited.extend(report.recruited);
                self.recruitment.mandatory.extend(report.mandatory);
                self.recruitment.requirements = report.requirements;
                self.commands.extend(commands);
            }
        }
        Ok(())
    }
    pub(in crate::ai) fn flush(&mut self) -> Result<(), CommandError> {
        while let Some(command) = self.workspace.state_mut().commands.pop_front() {
            if self.draft.winner().is_some() {
                self.workspace.state_mut().commands.clear();
                self.workspace.state_mut().attack_continuation = None;
                self.workspace.state_mut().finished = true;
                return Ok(());
            }
            self.emit(command)?;
            if self.combat_boundary {
                self.workspace.state_mut().commands.clear();
                return Ok(());
            }
        }
        if self.draft.winner().is_some() {
            self.workspace.state_mut().attack_continuation = None;
            self.workspace.state_mut().finished = true;
            return Ok(());
        }
        if let Some((unit, target, weapon)) = self.workspace.state_mut().attack_continuation.take()
        {
            let actor = self
                .draft
                .unit(unit.0)
                .ok_or(CommandError::UnknownUnit(unit))?
                .clone();
            if actor.retreating.is_some() {
                self.emit(crate::ai::intent_command(
                    &actor,
                    None,
                    actor.rebuilding,
                    actor.attack_spent,
                ))?;
            }
            self.emit(RuntimeCommand::Attack {
                unit,
                target,
                weapon,
            })?;
            return Ok(());
        }
        Ok(())
    }
}
