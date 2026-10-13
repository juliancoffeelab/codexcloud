//! Policies choose typed commands from a read-only battle. A workspace retains
//! turn-local commitments and the continuation of a classic unit's action.
#![deny(clippy::wildcard_enum_match_arm)]
pub mod campaigner;
pub mod context;
mod presets;
pub(crate) mod tactic;
use planner::{
    assignments as cooperation, candidates as cheap, escort, queued as priority, search as tactics,
};
pub mod objectives;
pub mod orders;
pub mod planner;
#[cfg(not(target_arch = "wasm32"))]
pub mod profiling;
pub mod recruiter;
mod roles;
mod snapshot;
use crate::{
    data::*,
    logic::{Battle, Command, CommandError, Game},
};
pub use campaigner::CampaignerKind as CampaignBlock;
use context::forecast as threats;
pub use planner::PlannerKind as Planner;
pub use presets::Blocks;
pub use priority::PrioritySnapshot;
pub use recruiter::RecruiterKind as Recruiter;
use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};
pub use tactics::Random;

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Tactical and recruitment policy, preserving each policy's stable choices.
pub enum Policy {
    #[default]
    Classic,
    Rush,
    Mixed,
    Priority,
    Battler,
    ClassicOld,
    RushOld,
    MixedOld,
}
impl Policy {
    /// Complete supported policies in their stable authored order.
    pub const ALL: [Self; 8] = [
        Self::Classic,
        Self::Rush,
        Self::Mixed,
        Self::Priority,
        Self::Battler,
        Self::ClassicOld,
        Self::RushOld,
        Self::MixedOld,
    ];
    /// Frozen native reference implementations, built with their original engine.
    pub const fn legacy(self) -> Option<&'static str> {
        match self {
            Self::ClassicOld => Some("classic"),
            Self::RushOld => Some("rush"),
            Self::MixedOld => Some("mixed"),
            Self::Classic | Self::Rush | Self::Mixed | Self::Priority | Self::Battler => None,
        }
    }
    /// Arena-only policies are excluded from campaign schedules and saves.
    pub const fn supports_campaign(self) -> bool {
        match self {
            Self::Classic
            | Self::Rush
            | Self::Mixed
            | Self::Priority
            | Self::ClassicOld
            | Self::RushOld
            | Self::MixedOld => true,
            Self::Battler => false,
        }
    }
    /// Stable JSON/CLI policy identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Rush => "rush",
            Self::Mixed => "mixed",
            Self::Priority => "priority",
            Self::Battler => "battler",
            Self::ClassicOld => "classicold",
            Self::RushOld => "rushold",
            Self::MixedOld => "mixedold",
        }
    }
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Treatment of retreating and mediated opponents during campaign battles.
pub enum Diplomacy {
    #[default]
    Restrained,
    Pragmatic,
}
impl Diplomacy {
    /// Stable JSON/CLI treatment identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restrained => "restrained",
            Self::Pragmatic => "pragmatic",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Tactical placement grouping of the unit's explicitly authored role.
pub enum TacticalRole {
    Frontline,
    Ranged,
    Healer,
}
/// Reusable turn-local planning state. Create a fresh workspace for each turn
/// or restore its saved snapshot when resuming the same battle and policy.
pub struct PlanningWorkspace {
    policy: Option<Policy>,
    seed: u32,
    recruitment_remaining: usize,
    goals: Option<Vec<Objective>>,
    prepared_goals: Option<Vec<orders::Target>>,
    diplomacy: Diplomacy,
    initialized: bool,
    finished: bool,
    classic_objectives: Vec<orders::Target>,
    classic_ids: VecDeque<u32>,
    classic_revision: u32,
    tactical_remaining: Option<usize>,
    commands: VecDeque<Command>,
    attack_continuation: Option<(UnitId, UnitId, WeaponId)>,
    routes: HashMap<(usize, Vec<usize>), Rc<Vec<u16>>>,
    commitments: HashMap<u32, String>,
    threats: threats::ArenaThreatWorkspace,
    priority: Option<PrioritySnapshot>,
    cheap: cheap::Workspace,
    blocks: Option<Blocks>,
    orders: Option<orders::CampaignOrders>,
    report: orders::PlanningReport,
    recruitment_report: orders::RecruitmentReport,
    turn_plan_ready: bool,
    issued: Option<Command>,
    cooperation: cooperation::Workspace,
    escort: escort::Workspace,
}
/// Serializable continuation of a policy phase. Route caches are rebuilt after
/// restoration; decisions, unit order and recruitment budgets remain intact.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanningSnapshot {
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orders: Option<orders::CampaignOrders>,
    #[serde(default)]
    pub report: orders::PlanningReport,
    #[serde(default)]
    pub recruitment_report: orders::RecruitmentReport,
    #[serde(default)]
    pub turn_plan_ready: bool,
    #[serde(default)]
    pub issued: Option<Command>,
    #[serde(default)]
    pub cooperation: cooperation::Workspace,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Blocks>,
    pub policy: Option<Policy>,
    pub seed: u32,
    pub recruitment_remaining: usize,
    pub goals: Option<Vec<Objective>>,
    pub diplomacy: Diplomacy,
    pub initialized: bool,
    pub finished: bool,
    pub classic_objectives: Vec<orders::Target>,
    pub classic_ids: Vec<UnitId>,
    #[serde(default)]
    pub classic_revision: u32,
    pub tactical_remaining: Option<usize>,
    pub commands: Vec<Command>,
    pub attack_continuation: Option<(UnitId, UnitId, WeaponId)>,
    pub commitments: Vec<(UnitId, String)>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<PrioritySnapshot>,
}
impl PlanningWorkspace {
    /// Start a policy phase with its seed, recruitment budget and objectives.
    pub fn new(
        seed: u32,
        recruitment_limit: usize,
        goals: Option<Vec<Objective>>,
        diplomacy: Diplomacy,
    ) -> Self {
        Self {
            policy: None,
            blocks: None,
            orders: None,
            report: orders::PlanningReport::default(),
            recruitment_report: orders::RecruitmentReport::default(),
            turn_plan_ready: false,
            issued: None,
            cooperation: cooperation::Workspace::default(),
            escort: escort::Workspace::default(),
            seed,
            recruitment_remaining: recruitment_limit,
            goals,
            prepared_goals: None,
            diplomacy,
            initialized: false,
            finished: false,
            classic_objectives: Vec::new(),
            classic_ids: VecDeque::new(),
            classic_revision: 0,
            tactical_remaining: None,
            commands: VecDeque::new(),
            attack_continuation: None,
            routes: HashMap::default(),
            commitments: HashMap::default(),
            threats: threats::ArenaThreatWorkspace::default(),
            priority: None,
            cheap: cheap::Workspace::default(),
        }
    }
    /// Select independent blocks before starting a phase. Saved continuations retain them.
    pub fn with_blocks(mut self, blocks: Blocks) -> Self {
        assert!(
            !self.initialized && self.policy.is_none(),
            "cannot replace blocks during an active phase"
        );
        self.blocks = Some(blocks);
        self
    }
    pub(crate) fn with_optional_blocks(self, blocks: Option<Blocks>) -> Self {
        match blocks {
            Some(blocks) => self.with_blocks(blocks),
            None => self,
        }
    }
    pub(super) fn effective_blocks(&self, policy: Policy) -> Blocks {
        self.blocks
            .or_else(|| policy.blocks())
            .expect("live AI preset")
    }
    pub(crate) fn complete_turn_plan(
        &mut self,
        policy: Policy,
        blocks: Blocks,
        report: orders::PlanningReport,
        recruitment: orders::RecruitmentReport,
    ) {
        self.policy = Some(policy);
        self.blocks = Some(blocks);
        self.report = report;
        self.recruitment_report = recruitment;
        self.initialized = true;
        self.finished = true;
    }
    /// Whether the policy has exhausted its actions and queued continuation.
    pub fn is_finished(&self) -> bool {
        self.finished && self.commands.is_empty()
    }
    pub(crate) fn diagnostics(&self) -> Option<orders::TurnDiagnostics> {
        let orders = self.orders.as_ref()?;
        orders.tactics.campaign.then(|| orders::TurnDiagnostics {
            orders: orders.clone(),
            report: self.report.clone(),
            recruitment: self.recruitment_report.clone(),
        })
    }
    /// Save the exact policy continuation without its recomputable route cache.
    pub fn snapshot(&self) -> PlanningSnapshot {
        let mut commitments: Vec<_> = self
            .commitments
            .iter()
            .map(|(id, goal)| (UnitId(*id), goal.clone()))
            .collect();
        commitments.sort_by_key(|(id, _)| id.0);
        PlanningSnapshot {
            orders: self.orders.clone(),
            report: self.report.clone(),
            recruitment_report: self.recruitment_report.clone(),
            turn_plan_ready: self.turn_plan_ready,
            issued: self.issued.clone(),
            cooperation: self.cooperation.clone(),
            blocks: self.blocks,
            policy: self.policy,
            seed: self.seed,
            recruitment_remaining: self.recruitment_remaining,
            goals: self.goals.clone(),
            diplomacy: self.diplomacy,
            initialized: self.initialized,
            finished: self.finished,
            classic_objectives: self.classic_objectives.clone(),
            classic_ids: self.classic_ids.iter().map(|id| UnitId(*id)).collect(),
            classic_revision: self.classic_revision,
            tactical_remaining: self.tactical_remaining,
            commands: self.commands.iter().cloned().collect(),
            attack_continuation: self.attack_continuation.clone(),
            commitments,
            priority: self.priority.clone(),
        }
    }
    /// Restore a previously validated snapshot for the same battle and turn.
    pub fn from_snapshot(saved: PlanningSnapshot) -> Self {
        Self {
            orders: saved.orders,
            report: saved.report,
            recruitment_report: saved.recruitment_report,
            turn_plan_ready: saved.turn_plan_ready,
            issued: saved.issued,
            cooperation: saved.cooperation,
            escort: escort::Workspace::default(),
            policy: saved.policy,
            blocks: saved.blocks,
            seed: saved.seed,
            recruitment_remaining: saved.recruitment_remaining,
            goals: saved.goals,
            prepared_goals: None,
            diplomacy: saved.diplomacy,
            cheap: cheap::Workspace::default(),
            initialized: saved.initialized,
            finished: saved.finished,
            classic_objectives: saved.classic_objectives,
            classic_ids: saved.classic_ids.into_iter().map(|id| id.0).collect(),
            classic_revision: saved.classic_revision,
            tactical_remaining: saved.tactical_remaining,
            commands: saved.commands.into(),
            attack_continuation: saved.attack_continuation,
            routes: HashMap::default(),
            commitments: saved
                .commitments
                .into_iter()
                .map(|(id, goal)| (id.0, goal))
                .collect(),
            threats: threats::ArenaThreatWorkspace::default(),
            priority: saved.priority,
        }
    }
}

/// Choose the next command without changing the battle. Apply each returned
/// command once before requesting another, using the same workspace and policy
/// until EndTurn. A fresh turn requires a fresh workspace.
///
/// EndTurn signals that the policy has exhausted its actions; the caller decides
/// when to apply it. Catalog mismatches and invalid planned actions return typed
/// command errors without changing the battle.
pub fn choose_command(
    catalog: &Catalog,
    battle: &Battle,
    policy: Policy,
    workspace: &mut PlanningWorkspace,
) -> Result<Command, CommandError> {
    if policy.legacy().is_some() {
        return Err(CommandError::ReferenceOnlyPolicy);
    }
    battle
        .with_game(catalog, |game| {
            choose_game_command(catalog, game, policy, workspace)
        })
        .map_err(|_| CommandError::CatalogMismatch)?
}

pub(crate) fn choose_game_command(
    rules: &Rules,
    game: &Game<'_>,
    policy: Policy,
    workspace: &mut PlanningWorkspace,
) -> Result<Command, CommandError> {
    if policy.legacy().is_some() {
        return Err(CommandError::ReferenceOnlyPolicy);
    }
    let blocks = workspace.effective_blocks(policy);
    if !blocks.campaign.supports_campaign() && game.campaign.is_some() {
        return Err(CommandError::BattleOnlyPolicy);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _profile = profiling::Decision::start(policy, game.turn);
    workspace.policy.get_or_insert(policy);
    workspace.blocks.get_or_insert(blocks);
    if !std::ptr::eq(rules, game.rules) {
        return Err(CommandError::CatalogMismatch);
    }
    if game.winner.is_some() {
        return Ok(Command::EndTurn);
    }
    if !workspace.turn_plan_ready {
        let mut tactic = tactic::Tactic::new(blocks.recruiter, blocks.campaign, blocks.planner);
        let plan = tactic.run_turn(game, workspace)?;
        workspace.report = plan.planning;
        workspace.recruitment_report = plan.recruitment;
        workspace.commands = plan.commands.into();
        workspace.turn_plan_ready = true;
        workspace.initialized = true;
        workspace.finished = true;
    }
    let command = workspace.commands.pop_front().unwrap_or(Command::EndTurn);
    workspace.issued = Some(command.clone());
    Ok(command)
}

impl Game<'_> {
    #[allow(dead_code)] // Compact native adapter; other hosts consume one decision at a time.
    pub(crate) fn play_policy(
        &mut self,
        policy: Policy,
        seed: u32,
        limit: usize,
        goals: Option<&[Objective]>,
        diplomacy: Diplomacy,
    ) -> Result<(), CommandError> {
        self.play_blocks(policy, None, seed, limit, goals, diplomacy)
    }
    pub(crate) fn play_blocks(
        &mut self,
        policy: Policy,
        blocks: Option<Blocks>,
        seed: u32,
        limit: usize,
        goals: Option<&[Objective]>,
        diplomacy: Diplomacy,
    ) -> Result<(), CommandError> {
        fn is_command_endturn(value: &Command) -> bool {
            match value {
                Command::EndTurn => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let mut workspace =
            PlanningWorkspace::new(seed, limit, goals.map(<[Objective]>::to_vec), diplomacy)
                .with_optional_blocks(blocks);
        loop {
            let command = choose_game_command(self.rules, self, policy, &mut workspace)?;
            if is_command_endturn(&command) {
                break;
            }
            self.execute(command)?;
        }
        Ok(())
    }
}

/// Records the continuation of one planned unit action on an isolated battle.
/// This lets classic evaluate its attack after the selected move without ever
/// mutating the caller's battle. Intent changes commit through engine commands.
struct Recorder {
    commands: VecDeque<Command>,
    intents: HashMap<u32, (Option<bool>, Option<bool>, bool)>,
    error: Option<CommandError>,
    retreat_logs: HashSet<u32>,
}
impl Recorder {
    fn new(game: &context::BattleView<'_, '_>) -> Self {
        Self {
            commands: VecDeque::new(),
            error: None,
            retreat_logs: HashSet::new(),
            intents: game
                .units()
                .iter()
                .map(|u| (u.id, (u.retreating, u.rebuilding, u.attacked)))
                .collect(),
        }
    }
    fn flush_intents(&mut self, game: &context::BattleView<'_, '_>) {
        for u in game.units() {
            let intent = (u.retreating, u.rebuilding, u.attacked);
            let record_retreat = self.retreat_logs.remove(&u.id);
            if record_retreat || self.intents.get(&u.id).is_some_and(|old| *old != intent) {
                self.commands.push_back(Command::SetUnitIntent {
                    unit: UnitId(u.id),
                    retreating: u.retreating,
                    rebuilding: u.rebuilding,
                    attacked: u.attacked,
                    record_retreat,
                });
            }
            self.intents.insert(u.id, intent);
        }
    }
    fn mark_retreat(&mut self, unit: u32) {
        self.retreat_logs.insert(unit);
    }
    fn emit(&mut self, game: &mut context::BattleDraft<'_, '_>, command: Command) {
        self.flush_intents(game);
        if let Err(error) = game.execute(command.clone()) {
            self.error = Some(error);
            return;
        }
        self.commands.push_back(command);
        self.intents = game
            .units()
            .iter()
            .map(|u| (u.id, (u.retreating, u.rebuilding, u.attacked)))
            .collect();
    }
    fn move_unit(&mut self, game: &mut context::BattleDraft<'_, '_>, unit: u32, tile: usize) {
        self.emit(game, move_command(game, unit, tile));
    }
    fn attack(
        &mut self,
        game: &mut context::BattleDraft<'_, '_>,
        unit: u32,
        target: u32,
        weapon: usize,
    ) {
        self.emit(game, attack_command(game, unit, target, weapon));
    }
    fn advance(&mut self, game: &mut context::BattleDraft<'_, '_>, unit: u32, kind: usize) {
        self.emit(game, advance_command(game, unit, kind));
    }
}

fn intent_command(
    unit: &Unit,
    retreating: Option<bool>,
    rebuilding: Option<bool>,
    attacked: bool,
) -> Command {
    Command::SetUnitIntent {
        unit: UnitId(unit.id),
        retreating,
        rebuilding,
        attacked,
        record_retreat: false,
    }
}
fn move_command(game: &context::BattleView<'_, '_>, unit: u32, tile: usize) -> Command {
    Command::Move {
        unit: UnitId(unit),
        tile: TileId(game.board().ids[tile].clone()),
    }
}
fn attack_command(
    game: &context::BattleView<'_, '_>,
    unit: u32,
    target: u32,
    weapon: usize,
) -> Command {
    let kind = game.unit(unit).unwrap().kind;
    Command::Attack {
        unit: UnitId(unit),
        target: UnitId(target),
        weapon: WeaponId(game.rules().units[kind].attacks[weapon].id.clone()),
    }
}
fn recruit_command(
    game: &context::BattleView<'_, '_>,
    unit: u32,
    kind: usize,
    tile: usize,
) -> Command {
    Command::Recruit {
        unit: UnitId(unit),
        kind: UnitTypeId(game.rules().units[kind].id.clone()),
        tile: TileId(game.board().ids[tile].clone()),
    }
}
fn advance_command(game: &context::BattleView<'_, '_>, unit: u32, kind: usize) -> Command {
    Command::Advance {
        unit: UnitId(unit),
        kind: UnitTypeId(game.rules().units[kind].id.clone()),
    }
}

#[cfg(test)]
mod courier_tests {
    use super::*;

    #[test]
    fn required_courier_can_detour_around_stationary_allies() {
        fn is_command_setunitintent(value: &Command) -> bool {
            match value {
                Command::SetUnitIntent { .. } => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::EndTurn
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue => false,
            }
        }
        fn is_command_move(value: &Command) -> bool {
            match value {
                Command::Move {
                    unit: UnitId(5), ..
                } => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::EndTurn
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        let mut game = Game::start_campaign(
            &rules,
            "borrowed-tide",
            0,
            vec![],
            0,
            vec![],
            vec![],
            vec![],
        );
        game.units.retain(|u| [1, 2, 4, 5].contains(&u.id));
        for (id, tile) in [(1, "8,5"), (2, "7,3"), (4, "8,3"), (5, "8,4")] {
            let tile = game.site(tile);
            game.units.iter_mut().find(|u| u.id == id).unwrap().tile = tile;
        }
        // Keep allies stationary for this turn: the courier must take a detour.
        for ally in game.units.iter_mut().filter(|u| u.id != 5) {
            ally.moved = true;
            ally.attacked = true;
        }
        game.roster_changed();
        let target = game.site("9,3");
        // Block the other direct approach as well.
        game.units.push(Unit {
            id: 100,
            kind: rules.types["reefguard"],
            tile: game.site("9,4"),
            side: Side::Player,
            hp: rules.units[rules.types["reefguard"]].hp,
            moved: true,
            attacked: true,
            experience: None,
            name: None,
            role: None,
            retreating: None,
            rebuilding: None,
        });
        game.roster_changed();
        let courier = game.unit(5).unwrap();
        let static_route = game.routes(courier.kind, &[target]);
        assert!(
            game.reachable(courier)
                .iter()
                .all(|&(tile, _)| static_route[tile] >= static_route[courier.tile])
        );
        let mut workspace = PlanningWorkspace::new(1, 0, None, Diplomacy::Restrained);
        let command = loop {
            let command =
                choose_game_command(&rules, &game, Policy::Mixed, &mut workspace).unwrap();
            if is_command_setunitintent(&command) {
                game.execute(command).unwrap();
            } else {
                break command;
            }
        };
        assert!(is_command_move(&command), "{command:?}");
        game.execute(command).unwrap();
        assert_ne!(game.board.ids[game.unit(5).unwrap().tile], "8,4");
    }

    #[test]
    fn allies_clear_approaches_and_complete_the_escort_for_every_policy() {
        fn is_command_endturn(value: &Command) -> bool {
            match value {
                Command::EndTurn => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        for policy in Policy::ALL
            .into_iter()
            .filter(|p| p.supports_campaign() && p.legacy().is_none())
        {
            let mut game = Game::start_campaign(
                &rules,
                "borrowed-tide",
                0,
                vec![],
                0,
                vec![],
                vec![],
                vec![],
            );
            game.units.retain(|u| u.side == Side::Player);
            for (id, tile) in [(1, "8,5"), (2, "7,3"), (3, "9,4"), (4, "8,3"), (5, "8,4")] {
                let tile = game.site(tile);
                game.units.iter_mut().find(|u| u.id == id).unwrap().tile = tile;
            }
            for tile in ["6,2", "8,5"] {
                let tile = game.site(tile);
                game.owners[tile] = Some(Side::Player);
            }
            game.roster_changed();
            let mut workspace = PlanningWorkspace::new(1, 0, None, Diplomacy::Restrained);
            for _ in 0..40 {
                let command = choose_game_command(&rules, &game, policy, &mut workspace).unwrap();
                if is_command_endturn(&command) {
                    break;
                }
                game.execute(command).unwrap();
                if game.winner.is_some() {
                    break;
                }
            }
            assert_eq!(
                game.winner,
                Some(Side::Player),
                "{policy:?}: {:?}",
                game.public_units()
            );
            assert_eq!(game.board.ids[game.unit(5).unwrap().tile], "9,3");
        }
    }
}
