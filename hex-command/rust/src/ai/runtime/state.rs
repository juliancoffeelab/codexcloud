//! Runtime owns turn execution and lends specific state to selected algorithms.
#[cfg(any(feature = "wasm", test))]
use crate::ai::{Blocks, Policy};
use crate::ai::{Diplomacy, orders::*, planner, policies};
use crate::data::*;
use hex_command_battle::RuntimeCommand;
use std::collections::VecDeque;

/// Public continuation handle. Its storage is private to runtime, not AI siblings.
pub struct PlanningWorkspace {
    pub(super) state: TurnState,
}

pub(super) struct TurnState {
    scope: Option<hex_command_battle::BattleScope>,
    planning_revision: Option<u64>,
    observed_roster: Vec<(u32, NameKey)>,
    acting_faction: Option<String>,
    #[cfg(any(feature = "wasm", test))]
    pub(super) proposal_faction: Option<String>,
    phase: Option<TurnPhase>,
    pub(super) control_roster: Option<Vec<(u32, TurnPhase, Option<NameKey>, Option<Role>)>>,
    #[cfg(any(feature = "wasm", test))]
    pub(super) controller_index: usize,
    #[cfg(any(feature = "wasm", test))]
    pub(super) controller_factions: Option<Vec<(String, Option<Policy>)>>,
    #[cfg(any(feature = "wasm", test))]
    pub(super) controller_workspace: Option<Box<PlanningWorkspace>>,
    pub(super) seed: u32,
    pub(super) recruitment_remaining: Option<usize>,
    pub(super) recruitment_opportunity: Option<super::context::RecruitmentOpportunity>,
    pub(super) goals: Option<Vec<Objective>>,
    pub(super) diplomacy: Diplomacy,
    #[cfg(any(feature = "wasm", test))]
    pub(super) initialized: bool,
    pub(super) finished: bool,
    pub(super) commands: VecDeque<RuntimeCommand>,
    pub(super) attack_continuation: Option<(UnitId, UnitId, WeaponKey)>,
    #[cfg(any(feature = "wasm", test))]
    pub(super) blocks: Option<Blocks>,
    pub(super) orders: Option<CampaignOrders>,
    #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
    pub(super) observation_orders: Option<TacticalOrders>,
    pub(super) report: PlanningReport,
    #[cfg(any(feature = "wasm", test))]
    pub(super) turn_plan_ready: bool,
    classic: planner::classic::ClassicPlannerSession,
    full: planner::search::SearchPlannerSession,
    queued: planner::queued::QueuedPlannerSession,
    coordination: policies::cooperation::CooperationState,
}

#[derive(Clone, Copy)]
pub(crate) struct TurnConfig<'a> {
    pub seed: u32,
    pub goals: Option<&'a [Objective]>,
    pub diplomacy: Diplomacy,
}

/// A planner can propose commands and reports, but cannot access execution storage.
pub(in crate::ai) struct PlannerTurn<'a> {
    pub seed: u32,
    pub goals: Option<&'a [Objective]>,
    pub orders: &'a mut CampaignOrders,
    pub recruitment: Option<policies::recruitment::EstimateInput<'a>>,
    pub commands: &'a mut VecDeque<RuntimeCommand>,
    pub report: &'a mut PlanningReport,
    pub finished: &'a mut bool,
    pub attack_continuation: &'a mut Option<(UnitId, UnitId, WeaponKey)>,
    pub coordination: &'a mut policies::cooperation::CooperationState,
}
pub(in crate::ai) struct PlannerParts<'a, S> {
    pub turn: PlannerTurn<'a>,
    pub session: &'a mut S,
}

// Expands field borrows at the call site so each selected session stays disjoint.
macro_rules! borrow_turn {
    ($state:expr) => {{
        PlannerTurn {
            seed: ($state).seed,
            goals: ($state).goals.as_deref(),
            orders: ($state).orders.as_mut().expect("turn orders"),
            recruitment: ($state)
                .recruitment_opportunity
                .as_ref()
                .map(|opportunity| policies::recruitment::EstimateInput {
                    orders: &opportunity.orders,
                    remaining_budget: opportunity.budget,
                }),
            commands: &mut ($state).commands,
            report: &mut ($state).report,
            finished: &mut ($state).finished,
            attack_continuation: &mut ($state).attack_continuation,
            coordination: &mut ($state).coordination,
        }
    }};
}

impl PlanningWorkspace {
    pub fn new(
        seed: u32,
        recruitment_limit: impl Into<Option<usize>>,
        goals: Option<Vec<Objective>>,
        diplomacy: Diplomacy,
    ) -> Self {
        Self {
            state: TurnState {
                scope: None,
                acting_faction: None,
                #[cfg(any(feature = "wasm", test))]
                proposal_faction: None,
                phase: None,
                control_roster: None,
                planning_revision: None,
                observed_roster: vec![],
                #[cfg(any(feature = "wasm", test))]
                controller_index: 0,
                #[cfg(any(feature = "wasm", test))]
                controller_factions: None,
                #[cfg(any(feature = "wasm", test))]
                controller_workspace: None,
                seed,
                recruitment_remaining: recruitment_limit.into(),
                recruitment_opportunity: None,
                goals,
                diplomacy,
                #[cfg(any(feature = "wasm", test))]
                initialized: false,
                finished: false,
                commands: VecDeque::new(),
                attack_continuation: None,
                #[cfg(any(feature = "wasm", test))]
                blocks: None,
                orders: None,
                #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
                observation_orders: None,
                report: PlanningReport::default(),
                #[cfg(any(feature = "wasm", test))]
                turn_plan_ready: false,
                classic: Default::default(),
                full: Default::default(),
                queued: Default::default(),
                coordination: Default::default(),
            },
        }
    }
    pub(in crate::ai::runtime) fn state_mut(&mut self) -> &mut TurnState {
        &mut self.state
    }
    pub(crate) fn config(&self) -> TurnConfig<'_> {
        TurnConfig {
            seed: self.state.seed,
            goals: self.state.goals.as_deref(),
            diplomacy: self.state.diplomacy,
        }
    }
    #[cfg(test)]
    pub(crate) fn orders(&self) -> Option<&CampaignOrders> {
        self.state.orders.as_ref()
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn finished(&self) -> bool {
        self.state.finished
    }
    pub(in crate::ai::runtime) fn turn_parts(&mut self) -> PlannerTurn<'_> {
        let state = &mut self.state;
        borrow_turn!(state)
    }
    pub(in crate::ai::runtime) fn classic_parts(
        &mut self,
    ) -> PlannerParts<'_, planner::classic::ClassicPlannerSession> {
        let state = &mut self.state;
        let turn = borrow_turn!(state);
        PlannerParts {
            turn,
            session: &mut state.classic,
        }
    }
    pub(in crate::ai::runtime) fn full_parts(
        &mut self,
    ) -> PlannerParts<'_, planner::search::SearchPlannerSession> {
        let state = &mut self.state;
        let turn = borrow_turn!(state);
        PlannerParts {
            turn,
            session: &mut state.full,
        }
    }
    pub(in crate::ai::runtime) fn queued_parts(
        &mut self,
    ) -> PlannerParts<'_, planner::queued::QueuedPlannerSession> {
        let state = &mut self.state;
        let turn = borrow_turn!(state);
        PlannerParts {
            turn,
            session: &mut state.queued,
        }
    }
    pub(super) fn refresh_scope(&mut self, game: &crate::logic::BattleSession<'_>) {
        let changed = self.state.scope.as_ref().is_some_and(|scope| {
            !game.in_scope(scope)
                || self.state.acting_faction.as_deref() != game.active_faction()
                || self.state.phase != Some(game.turn())
        });
        if changed {
            self.reset_planning();
            self.state.recruitment_opportunity = None;
            self.state.control_roster = None;
            #[cfg(any(feature = "wasm", test))]
            {
                self.state.controller_index = 0;
                self.state.controller_factions = None;
                self.state.controller_workspace = None;
            }
        }
        if self.state.scope.is_none() || changed {
            self.state.scope = Some(game.scope());
            self.state.acting_faction = game.active_faction().map(str::to_owned);
            self.state.phase = Some(game.turn());
        }
    }
    pub(super) fn refresh_control(&mut self, game: &crate::logic::BattleSession<'_>) -> bool {
        self.refresh_scope(game);
        let revision = game.planning_revision();
        let observed: Vec<_> = game
            .units()
            .iter()
            .filter(|unit| game.unit_faction(&unit) != game.observer_faction())
            .map(|unit| (unit.id, unit.faction))
            .collect();
        let observation_changed = self
            .state
            .planning_revision
            .is_some_and(|old| old != revision)
            || (self.state.planning_revision.is_some() && self.state.observed_roster != observed);
        self.state.planning_revision = Some(revision);
        self.state.observed_roster = observed;
        let roster = game.control_roster();
        let changed = self.state.control_roster.as_ref().is_some_and(|before| {
            before.iter().any(|old| {
                roster
                    .iter()
                    .find(|unit| unit.0 == old.0)
                    .map_or(old.3.is_some_and(Role::is_escort), |unit| {
                        unit.1 != old.1 || unit.2 != old.2 || unit.3 != old.3
                    })
            })
        });
        self.state.control_roster = Some(roster);
        if changed || observation_changed {
            self.reset_planning();
        }
        changed || observation_changed
    }
    fn reset_planning(&mut self) {
        let state = &mut self.state;
        state.orders = None;
        #[cfg(all(feature = "training", not(target_arch = "wasm32")))]
        {
            state.observation_orders = None;
        }
        state.commands.clear();
        state.attack_continuation = None;
        state.classic = Default::default();
        state.full = Default::default();
        state.queued = Default::default();
        state.coordination = Default::default();
        state.finished = false;
        #[cfg(any(feature = "wasm", test))]
        {
            state.turn_plan_ready = false;
        }
    }
    #[cfg(any(feature = "wasm", test))]
    pub(crate) fn proposal_faction(&self) -> Option<&str> {
        self.state.proposal_faction.as_deref()
    }
    #[cfg(any(feature = "wasm", test))]
    pub fn with_blocks(mut self, blocks: Blocks) -> Self {
        assert!(
            !self.state.initialized,
            "cannot replace blocks during an active phase"
        );
        self.state.blocks = Some(blocks);
        self
    }
    #[cfg(any(feature = "wasm", test))]
    pub(super) fn effective_blocks(&self, policy: Policy) -> Blocks {
        self.state.blocks.unwrap_or_else(|| policy.blocks())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn diagnostics(
        &self,
        report: PlanningReport,
        recruitment: RecruitmentReport,
    ) -> Option<TurnDiagnostics> {
        let orders = self.state.orders.as_ref()?;
        orders.tactics.campaign.then(|| TurnDiagnostics {
            orders: orders.clone(),
            report,
            recruitment,
        })
    }
}
