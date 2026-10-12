//! Policies choose typed commands from a read-only battle. A workspace retains
//! turn-local commitments and the continuation of a classic unit's action.
use self::objectives::actor_matches;
mod classic;
pub mod objectives;
mod recruitment;
mod roles;
mod snapshot;
pub(crate) mod tactics;
pub(crate) mod threats;
use crate::{
    data::*,
    logic::{Battle, Command, CommandError, Game},
};
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
}
impl Policy {
    /// Complete supported policies in their stable authored order.
    pub const ALL: [Self; 3] = [Self::Classic, Self::Rush, Self::Mixed];
    /// Stable JSON/CLI policy identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Rush => "rush",
            Self::Mixed => "mixed",
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
    diplomacy: Diplomacy,
    initialized: bool,
    finished: bool,
    classic_objectives: Vec<Objective>,
    classic_ids: VecDeque<u32>,
    tactical_remaining: Option<usize>,
    commands: VecDeque<Command>,
    attack_continuation: Option<(UnitId, UnitId, WeaponId)>,
    routes: HashMap<(usize, Vec<usize>), Rc<Vec<u16>>>,
    commitments: HashMap<u32, String>,
}
/// Serializable continuation of a policy phase. Route caches are rebuilt after
/// restoration; decisions, unit order and recruitment budgets remain intact.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanningSnapshot {
    pub policy: Option<Policy>,
    pub seed: u32,
    pub recruitment_remaining: usize,
    pub goals: Option<Vec<Objective>>,
    pub diplomacy: Diplomacy,
    pub initialized: bool,
    pub finished: bool,
    pub classic_objectives: Vec<Objective>,
    pub classic_ids: Vec<UnitId>,
    pub tactical_remaining: Option<usize>,
    pub commands: Vec<Command>,
    pub attack_continuation: Option<(UnitId, UnitId, WeaponId)>,
    pub commitments: Vec<(UnitId, String)>,
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
            seed,
            recruitment_remaining: recruitment_limit,
            goals,
            diplomacy,
            initialized: false,
            finished: false,
            classic_objectives: Vec::new(),
            classic_ids: VecDeque::new(),
            tactical_remaining: None,
            commands: VecDeque::new(),
            attack_continuation: None,
            routes: HashMap::default(),
            commitments: HashMap::default(),
        }
    }
    /// Whether the policy has exhausted its actions and queued continuation.
    pub fn is_finished(&self) -> bool {
        self.finished && self.commands.is_empty()
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
            policy: self.policy,
            seed: self.seed,
            recruitment_remaining: self.recruitment_remaining,
            goals: self.goals.clone(),
            diplomacy: self.diplomacy,
            initialized: self.initialized,
            finished: self.finished,
            classic_objectives: self.classic_objectives.clone(),
            classic_ids: self.classic_ids.iter().map(|id| UnitId(*id)).collect(),
            tactical_remaining: self.tactical_remaining,
            commands: self.commands.iter().cloned().collect(),
            attack_continuation: self.attack_continuation.clone(),
            commitments,
        }
    }
    /// Restore a previously validated snapshot for the same battle and turn.
    pub fn from_snapshot(saved: PlanningSnapshot) -> Self {
        Self {
            policy: saved.policy,
            seed: saved.seed,
            recruitment_remaining: saved.recruitment_remaining,
            goals: saved.goals,
            diplomacy: saved.diplomacy,
            initialized: saved.initialized,
            finished: saved.finished,
            classic_objectives: saved.classic_objectives,
            classic_ids: saved.classic_ids.into_iter().map(|id| id.0).collect(),
            tactical_remaining: saved.tactical_remaining,
            commands: saved.commands.into(),
            attack_continuation: saved.attack_continuation,
            routes: HashMap::default(),
            commitments: saved
                .commitments
                .into_iter()
                .map(|(id, goal)| (id.0, goal))
                .collect(),
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
    workspace.policy.get_or_insert(policy);
    if !std::ptr::eq(rules, game.rules) {
        return Err(CommandError::CatalogMismatch);
    }
    loop {
        if let Some(command) = workspace.commands.pop_front() {
            return Ok(command);
        }
        if workspace.finished || game.winner.is_some() {
            workspace.finished = true;
            return Ok(Command::EndTurn);
        }
        if let Some((unit, target, weapon)) = workspace.attack_continuation.take() {
            let actor = game.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
            if actor.retreating.is_some() {
                workspace.commands.push_back(intent_command(
                    actor,
                    None,
                    actor.rebuilding,
                    actor.attacked,
                ));
            }
            workspace.commands.push_back(Command::Attack {
                unit,
                target,
                weapon,
            });
            continue;
        }
        let restrained = workspace.diplomacy == Diplomacy::Restrained;
        if !workspace.initialized {
            workspace.initialized = true;
            for unit in game.units.iter().filter(|u| {
                u.side == game.turn && u.retreating == Some(true) && u.hp == rules.units[u.kind].hp
            }) {
                workspace.commands.push_back(intent_command(
                    unit,
                    None,
                    unit.rebuilding,
                    unit.attacked,
                ));
            }
            if !workspace.commands.is_empty() {
                continue;
            }
        }
        if workspace.recruitment_remaining > 0 {
            workspace.recruitment_remaining -= 1;
            if let Some(command) = game.choose_recruit(
                policy,
                workspace.seed,
                workspace.goals.as_deref(),
                restrained,
            ) {
                return Ok(command);
            }
        } else if policy == Policy::Classic {
            if workspace.tactical_remaining.is_none() {
                let explicit = workspace.goals.is_some()
                    || (game.turn == Side::Player && game.campaign.is_some());
                let goals = if explicit {
                    workspace.goals.as_deref()
                } else {
                    Some(&[][..])
                };
                workspace.classic_objectives = game.policy_objectives(goals);
                workspace.classic_ids = game.classic_order(&workspace.classic_objectives).into();
                workspace.tactical_remaining = Some(workspace.classic_ids.len());
                for unit in game
                    .units
                    .iter()
                    .filter(|u| u.side == game.turn && rules.units[u.kind].leader)
                {
                    let required = workspace.classic_objectives.iter().any(|g| {
                        g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, unit.id)
                    });
                    let (_, intent) = game.leader_rebuild_plan(unit.id, required);
                    if unit.rebuilding != intent {
                        workspace.commands.push_back(intent_command(
                            unit,
                            unit.retreating,
                            intent,
                            unit.attacked,
                        ));
                    }
                }
                if !workspace.commands.is_empty() {
                    continue;
                }
            }
            if let Some(id) = workspace.classic_ids.pop_front() {
                let mut virtual_game = game.fork();
                let mut recorder = Recorder::new(game);
                virtual_game.classic_choose(
                    game,
                    id,
                    &workspace.classic_objectives,
                    restrained,
                    &mut recorder,
                );
                recorder.flush_intents(&virtual_game);
                if let Some(error) = recorder.error {
                    return Err(error);
                }
                workspace.commands.extend(recorder.commands);
            } else {
                workspace.finished = true;
            }
        } else {
            let remaining = workspace.tactical_remaining.get_or_insert_with(|| {
                game.units.iter().filter(|u| u.side == game.turn).count() * 3
            });
            if *remaining == 0 {
                workspace.finished = true;
                continue;
            }
            *remaining -= 1;
            let mut intents = Vec::new();
            let plan = game.best_plan(
                policy,
                workspace.seed,
                workspace.goals.as_deref(),
                restrained,
                &mut workspace.routes,
                &mut workspace.commitments,
                &mut intents,
            );
            for (id, intent) in &intents {
                let actor = game.unit(*id).unwrap();
                workspace.commands.push_back(intent_command(
                    actor,
                    actor.retreating,
                    *intent,
                    actor.attacked,
                ));
            }
            if let Some(plan) = plan {
                let actor = game.unit(plan.unit).unwrap();
                let rebuilding = intents
                    .iter()
                    .find(|(id, _)| *id == plan.unit)
                    .map_or(actor.rebuilding, |(_, intent)| *intent);
                if let Some(kind) = plan.advance {
                    workspace
                        .commands
                        .push_back(advance_command(game, plan.unit, kind));
                } else {
                    if actor.tile != plan.tile {
                        workspace
                            .commands
                            .push_back(move_command(game, plan.unit, plan.tile));
                        let retreating = plan.withdrawing.then_some(true);
                        if actor.retreating != retreating {
                            workspace.commands.push_back(intent_command(
                                actor,
                                retreating,
                                rebuilding,
                                actor.attacked,
                            ));
                        }
                    }
                    if let (Some(target), Some(weapon)) = (plan.target, plan.weapon) {
                        workspace.attack_continuation = Some((
                            UnitId(plan.unit),
                            UnitId(target),
                            WeaponId(rules.units[actor.kind].attacks[weapon].id.clone()),
                        ));
                    }
                }
            } else {
                workspace.finished = true;
            }
        }
    }
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
        let mut workspace =
            PlanningWorkspace::new(seed, limit, goals.map(<[Objective]>::to_vec), diplomacy);
        loop {
            let command = choose_game_command(self.rules, self, policy, &mut workspace)?;
            if matches!(command, Command::EndTurn) {
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
    fn new(game: &Game<'_>) -> Self {
        Self {
            commands: VecDeque::new(),
            error: None,
            retreat_logs: HashSet::new(),
            intents: game
                .units
                .iter()
                .map(|u| (u.id, (u.retreating, u.rebuilding, u.attacked)))
                .collect(),
        }
    }
    fn flush_intents(&mut self, game: &Game<'_>) {
        for u in &game.units {
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
    fn emit(&mut self, game: &mut Game<'_>, command: Command) {
        self.flush_intents(game);
        if let Err(error) = game.execute(command.clone()) {
            self.error = Some(error);
            return;
        }
        self.commands.push_back(command);
        self.intents = game
            .units
            .iter()
            .map(|u| (u.id, (u.retreating, u.rebuilding, u.attacked)))
            .collect();
    }
    fn move_unit(&mut self, game: &mut Game<'_>, unit: u32, tile: usize) {
        self.emit(game, move_command(game, unit, tile));
    }
    fn attack(&mut self, game: &mut Game<'_>, unit: u32, target: u32, weapon: usize) {
        self.emit(game, attack_command(game, unit, target, weapon));
    }
    fn advance(&mut self, game: &mut Game<'_>, unit: u32, kind: usize) {
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
fn move_command(game: &Game<'_>, unit: u32, tile: usize) -> Command {
    Command::Move {
        unit: UnitId(unit),
        tile: TileId(game.board.ids[tile].clone()),
    }
}
fn attack_command(game: &Game<'_>, unit: u32, target: u32, weapon: usize) -> Command {
    let kind = game.unit(unit).unwrap().kind;
    Command::Attack {
        unit: UnitId(unit),
        target: UnitId(target),
        weapon: WeaponId(game.rules.units[kind].attacks[weapon].id.clone()),
    }
}
fn recruit_command(game: &Game<'_>, unit: u32, kind: usize, tile: usize) -> Command {
    Command::Recruit {
        unit: UnitId(unit),
        kind: UnitTypeId(game.rules.units[kind].id.clone()),
        tile: TileId(game.board.ids[tile].clone()),
    }
}
fn advance_command(game: &Game<'_>, unit: u32, kind: usize) -> Command {
    Command::Advance {
        unit: UnitId(unit),
        kind: UnitTypeId(game.rules.units[kind].id.clone()),
    }
}

#[cfg(test)]
mod courier_tests {
    use super::*;

    #[test]
    fn required_courier_can_detour_around_stationary_allies() {
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
            if matches!(command, Command::SetUnitIntent { .. }) {
                game.execute(command).unwrap();
            } else {
                break command;
            }
        };
        assert!(
            matches!(
                command,
                Command::Move {
                    unit: UnitId(5),
                    ..
                }
            ),
            "{command:?}"
        );
        game.execute(command).unwrap();
        assert_ne!(game.board.ids[game.unit(5).unwrap().tile], "8,4");
    }

    #[test]
    fn allies_clear_approaches_and_complete_the_escort_for_every_policy() {
        let rules = Rules::load().unwrap();
        for policy in Policy::ALL {
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
                if matches!(command, Command::EndTurn) {
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
