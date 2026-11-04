//! Runtime-owned execution contexts; evaluation cannot construct these capabilities.
use crate::ai::campaigner::mission::{CampaignInput, MissionPolicy};
use crate::{
    ai::{
        PlanningWorkspace,
        orders::*,
        recruiter::{Recruiter, RecruiterKind},
    },
    data::*,
    logic::Game,
};
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
) -> Result<(Vec<Command>, RecruitmentReport), CommandError> {
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

pub(crate) struct CampaignView<'a, 'catalog> {
    mission: MissionPolicy<'a, 'catalog>,
    input: CampaignInput<'a>,
}
impl<'a, 'catalog> CampaignView<'a, 'catalog> {
    pub(in crate::ai) fn new(game: &'a Game<'catalog>, config: super::TurnConfig<'a>) -> Self {
        Self {
            mission: MissionPolicy::new(game.campaign_view(), game.phase_forecasts()),
            input: CampaignInput {
                goals: config.goals,
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
    commands: Vec<Command>,
    recruitment: RecruitmentReport,
}
impl<'catalog> PlanningContext<'catalog> {
    pub(super) fn fork(game: &Game<'catalog>) -> Self {
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
            commands: self.commands,
            recruitment,
            planning,
        }
    }
}
pub(crate) struct RecruitmentContext<'a, 'catalog> {
    draft: &'a mut ActionForecast<'catalog>,
    commands: &'a mut Vec<Command>,
    seed: u32,
    budget: Option<usize>,
}
impl RecruitmentContext<'_, '_> {
    pub(in crate::ai) fn seed(&self) -> u32 {
        self.seed
    }
    pub(in crate::ai) fn budget(&self) -> Option<usize> {
        self.budget
    }
    pub(in crate::ai) fn battle(&self) -> &BattleView<'_, '_> {
        self.draft
    }
    pub(in crate::ai) fn emit(&mut self, command: Command) -> Result<(), CommandError> {
        if !self.draft.permits(&command) {
            return Err(CommandError::IllegalIntent);
        }
        self.draft.execute(command.clone())?;
        self.commands.push(command);
        Ok(())
    }
}
pub(crate) struct TacticalContext<'a, 'catalog> {
    pub(super) combat_boundary: bool,
    recruitment: &'a mut RecruitmentReport,
    draft: &'a mut ActionForecast<'catalog>,
    commands: &'a mut Vec<Command>,
    workspace: &'a mut PlanningWorkspace,
}
impl TacticalContext<'_, '_> {
    pub(in crate::ai) fn is_boundary(&self) -> bool {
        self.combat_boundary
    }

    pub(in crate::ai) fn turn_parts(&mut self) -> super::PlannerTurn<'_> {
        self.workspace.turn_parts()
    }
    pub(in crate::ai) fn classic_parts(
        &mut self,
    ) -> (
        &BattleView<'_, '_>,
        PhaseForecasts<'_, '_>,
        ActionForecastSource<'_, '_>,
        super::PlannerParts<'_, crate::ai::planner::classic::Session>,
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
        &BattleView<'_, '_>,
        PhaseForecasts<'_, '_>,
        ActionForecastSource<'_, '_>,
        super::PlannerParts<'_, crate::ai::planner::search::Session>,
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
        &BattleView<'_, '_>,
        PhaseForecasts<'_, '_>,
        ActionForecastSource<'_, '_>,
        super::PlannerParts<'_, crate::ai::planner::queued::Session>,
    ) {
        (
            &*self.draft,
            self.draft.phase_forecasts(),
            self.draft.action_forecasts(),
            self.workspace.queued_parts(),
        )
    }
    pub(in crate::ai) fn battle(&self) -> &BattleView<'_, '_> {
        self.draft
    }
    pub(in crate::ai) fn emit(&mut self, command: Command) -> Result<(), CommandError> {
        if !self.draft.permits(&command) {
            return Err(CommandError::IllegalIntent);
        }
        let before = self.draft.control_roster();
        self.draft.execute(command.clone())?;
        let control_changed = command != Command::EndTurn && before != self.draft.control_roster();
        self.combat_boundary = command.is_attack() || control_changed;
        if control_changed {
            self.workspace.state_mut().finished = false;
        }
        let moved = if let Command::Move { unit, .. } = &command {
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
                    actor.attacked,
                ))?;
            }
            self.emit(Command::Attack {
                unit,
                target,
                weapon,
            })?;
            return Ok(());
        }
        Ok(())
    }
}
