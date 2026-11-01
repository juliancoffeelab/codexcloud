//! Policies choose typed commands from a read-only battle. A workspace retains
//! turn-local commitments and the continuation of a classic unit's action.
#![deny(clippy::wildcard_enum_match_arm)]
use crate::reuse::Vec;
#[cfg(test)]
use crate::reuse::vec;
pub mod campaigner;
pub mod context;
mod presets;
pub(crate) mod tactic;
use planner::{
    assignments as cooperation, candidates as cheap, escort, queued as priority, search as tactics,
};
mod ids;
#[cfg(target_arch = "wasm32")]
mod keys;
#[cfg(target_arch = "wasm32")]
mod sites;
#[cfg(not(target_arch = "wasm32"))]
mod sites {
    pub type Sites = smallvec::SmallVec<[super::orders::SiteKey; 8]>;
}
pub mod objectives;
pub mod orders;
pub mod planner;
#[cfg(not(target_arch = "wasm32"))]
pub mod profiling;
pub mod recruiter;
mod roles;
use crate::reuse::Map as HashMap;
use crate::{
    data::*,
    logic::{Action as Command, CommandError, Game},
};
pub use campaigner::CampaignerKind as CampaignBlock;
use context::forecast as threats;
pub use planner::PlannerKind as Planner;
pub use presets::Blocks;
use priority::UnitQueue;
pub use recruiter::RecruiterKind as Recruiter;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
pub use tactics::Random;

#[cfg(target_arch = "wasm32")]
pub(crate) fn prepare_storage(tiles: usize, weapons: usize) {
    let _registration = crate::reuse::registration();
    sites::prepare(tiles);
    keys::prepare(tiles);
    crate::reuse::prepare::<orders::SiteKey>(4, tiles + 1);
    crate::reuse::prepare::<orders::TacticalConstraint>(4, tiles + 1);
    crate::reuse::prepare::<(UnitId, ids::TileKey)>(4, tiles + 1);
    crate::reuse::prepare::<(UnitId, Vec<ids::TileKey>)>(4, tiles + 1);
    crate::reuse::prepare::<(UnitId, Option<bool>)>(4, tiles + 1);
    crate::reuse::prepare::<(usize, u32, usize)>(2, tiles + 1);
    crate::reuse::prepare::<usize>(72, tiles + 1);
    crate::reuse::prepare_large::<usize>(tiles * tiles);
    crate::reuse::prepare::<String>(4, 8);
    crate::reuse::prepare::<ids::TileKey>(32, 16);
    crate::reuse::prepare_large::<ids::TileKey>(tiles + 1);
    crate::reuse::prepare::<orders::Target>(8, tiles + 1);
    crate::reuse::prepare::<orders::ObjectiveOrder>(4, tiles + 1);
    crate::reuse::prepare::<f64>(6, tiles + 1);
    crate::reuse::prepare::<(UnitId, Vec<UnitId>)>(2, tiles + 1);
    crate::reuse::prepare_maps::<u32, i32>(tiles, 4);
    drop(PlanningWorkspace::new(0, None, None, Diplomacy::Restrained));
    drop(orders::TacticalOrders::default());
    drop(orders::RecruitmentReport::default());
    crate::reuse::prepare::<(u32, (Option<bool>, Option<bool>, bool))>(4, tiles + 1);
    crate::reuse::prepare_maps::<u32, (Vec<usize>, Option<bool>)>(tiles, 8);
    crate::reuse::prepare::<Command>(96, 8);
    crate::reuse::prepare_large::<Command>(tiles * 4 + 8);
    crate::reuse::prepare::<u32>(64, tiles + 1);
    crate::reuse::prepare::<UnitId>(16, tiles + 1);
    crate::reuse::prepare_maps::<u32, orders::GoalKey>(tiles, 4);
    crate::reuse::prepare::<orders::DutyReport>(4, tiles * 3 + 8);
    crate::reuse::prepare::<orders::RecruitmentShortfall>(4, tiles + 1);
    crate::reuse::prepare::<orders::RecruitRequirement>(4, tiles + 1);
    crate::reuse::prepare::<(u32, ids::UnitTypeId)>(4, tiles + 1);
    crate::reuse::prepare::<(usize, u32, f64)>(2, tiles + 1);
    crate::reuse::prepare::<(f64, bool, bool)>(2, tiles + 1);
    crate::reuse::prepare::<(usize, f64, f64)>(2, tiles * 4 + 1);
    crate::reuse::prepare::<orders::AuthorGoal>(4, tiles + 1);
    threats::prepare_storage(tiles, weapons);
    recruiter::prepare_storage(tiles, weapons);
    cheap::prepare_storage(tiles);
    crate::reuse::prepare_registered(tiles);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn prepare_storage(_tiles: usize, _weapons: usize) {}

/// Native diagnostics reuse the planner's conservative, independent-enemy
/// forecast without executing a plan or changing the battle.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn audit_leader_exposure(
    game: &Game<'_>,
    leader: &Unit,
    tiles: impl IntoIterator<Item = usize>,
) -> std::vec::Vec<(usize, i32)> {
    let view = context::BattleView::new(game);
    let mut forecast = threats::Threats::new(&view, leader.side);
    tiles
        .into_iter()
        .map(|tile| (tile, forecast.at(leader, tile, None)))
        .collect()
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
/// Tactical and recruitment policy, preserving each policy's stable choices.
pub enum Policy {
    #[default]
    Classic,
    Rush,
    Balanced,
    Rotated,
    Situational,
    Predictor,
}
impl Policy {
    /// Supported presets in their stable authored order.
    pub const ALL: [Self; 6] = [
        Self::Classic,
        Self::Rush,
        Self::Balanced,
        Self::Rotated,
        Self::Situational,
        Self::Predictor,
    ];
    /// Stable JSON/CLI policy identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Rush => "rush",
            Self::Balanced => "balanced",
            Self::Rotated => "rotated",
            Self::Situational => "situational",
            Self::Predictor => "predictor",
        }
    }
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(clap::ValueEnum))]
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
    goals: Option<std::vec::Vec<Objective>>,
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
    attack_continuation: Option<(UnitId, UnitId, usize)>,
    commitments: HashMap<u32, orders::GoalKey>,
    threats: threats::ArenaThreatWorkspace,
    priority: Option<UnitQueue>,
    cheap: cheap::Workspace,
    search: tactics::Scratch,
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
        goals: Option<std::vec::Vec<Objective>>,
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
            commands: Vec::<Command>::with_capacity(8).into(),
            attack_continuation: None,
            commitments: HashMap::default(),
            threats: threats::ArenaThreatWorkspace::default(),
            priority: None,
            cheap: cheap::Workspace::default(),
            search: tactics::Scratch::default(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn finished(&self) -> bool {
        self.finished
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
        debug_assert!(workspace.commands.is_empty());
        crate::reuse::recycle_owned(std::vec::Vec::from(std::mem::take(&mut workspace.commands)));
        workspace.commands = Vec::<Command>::with_capacity(8).into();
        let mut tactic = tactic::Tactic::new(blocks.recruiter, blocks.campaign, blocks.planner);
        let plan = tactic.run_turn(game, workspace)?;
        workspace.report = plan.planning;
        crate::reuse::recycle_owned(std::vec::Vec::from(std::mem::take(&mut workspace.commands)));
        workspace.commands = plan.commands.into();
        workspace.turn_plan_ready = true;
    }
    let command = workspace.commands.pop_front().unwrap_or(Command::EndTurn);
    if command.is_attack() {
        workspace.turn_plan_ready = false;
    }
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
        loop {
            let plan = {
                #[cfg(not(target_arch = "wasm32"))]
                let _profile = profiling::Decision::start(policy, self.turn);
                tactic.run_turn(self, &mut workspace)?
            };
            for command in plan.commands {
                self.execute(command)?;
            }
            if workspace.finished || self.winner.is_some() {
                break;
            }
        }
        Ok(())
    }
}

/// Records the continuation of one planned unit action on an isolated battle.
/// This lets classic evaluate its attack after the selected move without ever
/// mutating the caller's battle. Intent changes commit through engine commands.
struct Recorder {
    commands: Vec<Command>,
    intents: Vec<(u32, (Option<bool>, Option<bool>, bool))>,
    error: Option<CommandError>,
    retreat_log: Option<u32>,
}
impl Recorder {
    fn new(game: &context::BattleView<'_, '_>) -> Self {
        Self {
            commands: Vec::<Command>::new(),
            error: None,
            retreat_log: None,
            intents: game
                .units()
                .iter()
                .map(|u| (u.id, (u.retreating, u.rebuilding, u.attacked)))
                .collect(),
        }
    }
    fn flush_intents(&mut self, game: &context::BattleView<'_, '_>) {
        for (index, u) in game.units().iter().enumerate() {
            let intent = (u.retreating, u.rebuilding, u.attacked);
            let record_retreat = self.retreat_log == Some(u.id);
            if record_retreat {
                self.retreat_log = None;
            }
            let old = self
                .intents
                .get(index)
                .filter(|entry| entry.0 == u.id)
                .or_else(|| self.intents.iter().find(|entry| entry.0 == u.id));
            if record_retreat || old.is_some_and(|entry| entry.1 != intent) {
                self.commands.push(Command::SetUnitIntent {
                    unit: UnitId(u.id),
                    retreating: u.retreating,
                    rebuilding: u.rebuilding,
                    attacked: u.attacked,
                    record_retreat,
                });
            }
        }
        self.snapshot(game);
    }
    fn snapshot(&mut self, game: &context::BattleView<'_, '_>) {
        self.intents.clear();
        self.intents.extend(
            game.units()
                .iter()
                .map(|u| (u.id, (u.retreating, u.rebuilding, u.attacked))),
        );
    }
    fn mark_retreat(&mut self, unit: u32) {
        debug_assert!(self.retreat_log.is_none_or(|id| id == unit));
        self.retreat_log = Some(unit);
    }
    fn emit(&mut self, game: &mut context::BattleDraft<'_, '_>, command: Command) {
        self.flush_intents(game);
        if let Err(error) = game.execute(command.clone()) {
            self.error = Some(error);
            return;
        }
        self.commands.push(command);
        self.snapshot(game);
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
fn move_command(_game: &context::BattleView<'_, '_>, unit: u32, tile: usize) -> Command {
    Command::Move {
        unit: UnitId(unit),
        tile,
    }
}
fn attack_command(
    _game: &context::BattleView<'_, '_>,
    unit: u32,
    target: u32,
    weapon: usize,
) -> Command {
    Command::Attack {
        unit: UnitId(unit),
        target: UnitId(target),
        weapon,
    }
}
fn recruit_command(
    _game: &context::BattleView<'_, '_>,
    unit: u32,
    kind: usize,
    tile: usize,
) -> Command {
    Command::Recruit {
        unit: UnitId(unit),
        kind,
        tile,
    }
}
fn advance_command(_game: &context::BattleView<'_, '_>, unit: u32, kind: usize) -> Command {
    Command::Advance {
        unit: UnitId(unit),
        kind,
    }
}

#[cfg(test)]
mod courier_tests {
    use super::*;
    use crate::reuse::vec;

    #[test]
    fn required_courier_can_detour_around_stationary_allies() {
        fn is_command_setunitintent(value: &Command) -> bool {
            match value {
                Command::SetUnitIntent { .. } => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::EndTurn => false,
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
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        let mut game = Game::start_campaign(
            &rules,
            "borrowed-tide",
            0,
            std::vec![],
            0,
            std::vec![],
            std::vec![],
            std::vec![],
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
            let command = choose_game_command(&game, Policy::Balanced, &mut workspace).unwrap();
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
    fn allies_clear_approaches_and_complete_the_escort() {
        fn is_command_endturn(value: &Command) -> bool {
            match value {
                Command::EndTurn => true,
                Command::Move { .. }
                | Command::Attack { .. }
                | Command::Recruit { .. }
                | Command::Advance { .. }
                | Command::SetUnitIntent { .. } => false,
            }
        }

        let rules = Rules::load().unwrap();
        for policy in [Policy::Classic, Policy::Balanced] {
            let mut game = Game::start_campaign(
                &rules,
                "borrowed-tide",
                0,
                std::vec![],
                0,
                std::vec![],
                std::vec![],
                std::vec![],
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

impl Drop for PlanningWorkspace {
    fn drop(&mut self) {
        crate::reuse::recycle_owned(std::vec::Vec::from(std::mem::take(&mut self.commands)));
        crate::reuse::recycle_owned(std::vec::Vec::from(std::mem::take(&mut self.classic_ids)));
    }
}
