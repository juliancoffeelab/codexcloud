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
use crate::{
    data::*,
    logic::{Command, CommandError, Game},
};
pub use campaigner::CampaignerKind as CampaignBlock;
use context::forecast as threats;
pub use planner::PlannerKind as Planner;
pub use presets::Blocks;
use priority::UnitQueue;
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
}
impl Policy {
    /// Complete supported policies in their stable authored order.
    pub const ALL: [Self; 5] = [
        Self::Classic,
        Self::Rush,
        Self::Mixed,
        Self::Priority,
        Self::Battler,
    ];
    /// Arena-only policies are excluded from campaign schedules and saves.
    pub const fn supports_campaign(self) -> bool {
        match self {
            Self::Classic | Self::Rush | Self::Mixed | Self::Priority => true,
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
/// and discard it when the turn ends.
pub struct PlanningWorkspace {
    seed: u32,
    recruitment_remaining: Option<usize>,
    goals: Option<Vec<Objective>>,
    prepared_goals: Option<Vec<orders::Target>>,
    diplomacy: Diplomacy,
    #[cfg(any(feature = "wasm", test))]
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
    priority: Option<UnitQueue>,
    cheap: cheap::Workspace,
    #[cfg(any(feature = "wasm", test))]
    blocks: Option<Blocks>,
    orders: Option<orders::CampaignOrders>,
    report: orders::PlanningReport,
    #[cfg(any(feature = "wasm", test))]
    turn_plan_ready: bool,
    cooperation: cooperation::Workspace,
    escort: escort::Workspace,
}
impl PlanningWorkspace {
    /// Start a policy phase with its seed, recruitment budget and objectives.
    pub fn new(
        seed: u32,
        recruitment_limit: impl Into<Option<usize>>,
        goals: Option<Vec<Objective>>,
        diplomacy: Diplomacy,
    ) -> Self {
        Self {
            #[cfg(any(feature = "wasm", test))]
            blocks: None,
            orders: None,
            report: orders::PlanningReport::default(),
            #[cfg(any(feature = "wasm", test))]
            turn_plan_ready: false,
            cooperation: cooperation::Workspace::default(),
            escort: escort::Workspace::default(),
            seed,
            recruitment_remaining: recruitment_limit.into(),
            goals,
            prepared_goals: None,
            diplomacy,
            #[cfg(any(feature = "wasm", test))]
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
    /// Select independent blocks before starting a phase.
    #[cfg(any(feature = "wasm", test))]
    pub fn with_blocks(mut self, blocks: Blocks) -> Self {
        assert!(
            !self.initialized,
            "cannot replace blocks during an active phase"
        );
        self.blocks = Some(blocks);
        self
    }
    #[cfg(any(feature = "wasm", test))]
    pub(super) fn effective_blocks(&self, policy: Policy) -> Blocks {
        self.blocks.unwrap_or_else(|| policy.blocks())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn diagnostics(
        &self,
        report: orders::PlanningReport,
        recruitment: orders::RecruitmentReport,
    ) -> Option<orders::TurnDiagnostics> {
        let orders = self.orders.as_ref()?;
        orders.tactics.campaign.then(|| orders::TurnDiagnostics {
            orders: orders.clone(),
            report,
            recruitment,
        })
    }
}

#[cfg(any(feature = "wasm", test))]
pub(crate) fn choose_game_command(
    game: &Game<'_>,
    policy: Policy,
    workspace: &mut PlanningWorkspace,
) -> Result<Command, CommandError> {
    let blocks = workspace.effective_blocks(policy);
    if !blocks.campaign.supports_campaign() && game.campaign.is_some() {
        return Err(CommandError::BattleOnlyPolicy);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _profile = profiling::Decision::start(policy, game.turn);
    workspace.initialized = true;
    workspace.blocks.get_or_insert(blocks);
    if game.winner.is_some() {
        return Ok(Command::EndTurn);
    }
    if !workspace.turn_plan_ready {
        let mut tactic = tactic::Tactic::new(blocks.recruiter, blocks.campaign, blocks.planner);
        let plan = tactic.run_turn(game, workspace)?;
        workspace.report = plan.planning;
        workspace.commands = plan.commands.into();
        workspace.turn_plan_ready = true;
        workspace.finished = true;
    }
    let command = workspace.commands.pop_front().unwrap_or(Command::EndTurn);
    Ok(command)
}

#[cfg(not(target_arch = "wasm32"))]
impl Game<'_> {
    pub(crate) fn play_blocks(
        &mut self,
        policy: Policy,
        blocks: Option<Blocks>,
        seed: u32,
        limit: usize,
        goals: Option<&[Objective]>,
        diplomacy: Diplomacy,
    ) -> Result<(), CommandError> {
        let blocks = blocks.unwrap_or_else(|| policy.blocks());
        if !blocks.campaign.supports_campaign() && self.campaign.is_some() {
            return Err(CommandError::BattleOnlyPolicy);
        }
        if self.winner.is_some() {
            return Ok(());
        }
        let mut workspace =
            PlanningWorkspace::new(seed, limit, goals.map(<[Objective]>::to_vec), diplomacy);
        let mut tactic = tactic::Tactic::new(blocks.recruiter, blocks.campaign, blocks.planner);
        let plan = {
            let _profile = profiling::Decision::start(policy, self.turn);
            tactic.run_turn(self, &mut workspace)?
        };
        for command in plan.commands {
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
            let command = choose_game_command(&game, Policy::Mixed, &mut workspace).unwrap();
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
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        for policy in Policy::ALL.into_iter().filter(|p| p.supports_campaign()) {
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
                let command = choose_game_command(&game, policy, &mut workspace).unwrap();
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
