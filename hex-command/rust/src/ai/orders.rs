//! Explicit, turn-local contracts between the three fixed AI blocks.
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};

/// Generic heuristic hints supplement typed duties without exposing mission kinds.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Capture,
    Defeat,
    Visit,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub kind: TargetKind,
    pub tiles: Vec<Hex>,
    pub weight: f64,
    pub actor: Option<String>,
    pub family: Option<Family>,
    pub hold: Option<bool>,
}
impl From<crate::data::Objective> for Target {
    fn from(value: crate::data::Objective) -> Self {
        let kind = match value.kind {
            ObjectiveKind::Capture => TargetKind::Capture,
            ObjectiveKind::Defeat => TargetKind::Defeat,
            ObjectiveKind::Visit
            | ObjectiveKind::Escort
            | ObjectiveKind::Escape
            | ObjectiveKind::Vigil
            | ObjectiveKind::Seal => TargetKind::Visit,
        };
        Self {
            id: value.id,
            kind,
            tiles: value.tiles,
            weight: value.weight,
            actor: value.actor,
            family: value.family,
            hold: value.hold,
        }
    }
}
pub(in crate::ai) fn target(id: &str, kind: TargetKind, tiles: Vec<Hex>, weight: f64) -> Target {
    Target {
        id: id.into(),
        kind,
        tiles,
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitSelector {
    Type(UnitTypeId),
    AnyOf(Vec<UnitTypeId>),
    Family(Family),
    Role(UnitRole),
}
impl UnitSelector {
    pub(crate) fn accepts(&self, game: &BattleView<'_, '_>, kind: usize) -> bool {
        let rule = &game.rules().units[kind];
        match self {
            Self::Type(id) => rule.id == id.0,
            Self::AnyOf(ids) => ids.iter().any(|id| rule.id == id.0),
            Self::Family(family) => rule.family == *family,
            Self::Role(role) => rule.role == *role,
        }
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecruitRequirement {
    EnsureUnits {
        selector: UnitSelector,
        count: usize,
    },
}
impl RecruitRequirement {
    pub(crate) fn deficit(&self, game: &BattleView<'_, '_>) -> usize {
        match self {
            Self::EnsureUnits { selector, count } => count.saturating_sub(
                game.units()
                    .iter()
                    .filter(|u| {
                        game.belongs_to_faction(u, game.observer_faction())
                            && selector.accepts(game, u.kind)
                    })
                    .count(),
            ),
        }
    }
    pub(crate) fn accepts(&self, game: &BattleView<'_, '_>, kind: usize) -> bool {
        match self {
            Self::EnsureUnits { selector, count: _ } => selector.accepts(game, kind),
        }
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentOrders {
    pub requirements: Vec<RecruitRequirement>,
    pub reserved: Vec<TileId>,
    pub rotate_pool: bool,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum BattleObjective {
    #[default]
    BattleToKill,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TacticalConstraint {
    /// A soft preference evaluated against objective value and full exposure.
    Survive { actor: UnitId },
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ObjectivePriority {
    Normal,
    Important,
    Critical,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TacticalObjective {
    GoTo {
        actor: UnitId,
        destination: TileId,
    },
    Occupy {
        actor: UnitId,
        location: TileId,
    },
    Escort {
        actor: UnitId,
        guards: usize,
        selector: UnitSelector,
    },
    ClearPassage {
        actor: UnitId,
        destination: TileId,
    },
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveOrder {
    pub id: String,
    pub priority: ObjectivePriority,
    pub objective: TacticalObjective,
}

/// Visible mission progress, separate from planner-owned duty assignments.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveProgress {
    pub id: String,
    pub current: u32,
    pub total: u32,
    pub satisfied: bool,
    pub hold_turns_remaining: Option<u32>,
}

pub use hex_command_battle::model::ScopedOrder;

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TacticalOrders {
    /// First round triggering an unconditional scripted campaign defeat.
    pub deadline_round: Option<u32>,
    pub progress: Vec<ObjectiveProgress>,
    pub scoped: Vec<ScopedOrder>,
    /// Actors withheld by Campaigner, independent of unit archetypes or economy.
    pub inactive: Vec<UnitId>,
    /// Promotion branches selected by Recruiter; Planner owns their timing.
    pub promotions: Vec<(UnitId, UnitTypeId)>,
    pub battle: BattleObjective,
    pub constraints: Vec<TacticalConstraint>,
    pub objectives: Vec<ObjectiveOrder>,
    /// General combat/resource targets used by the existing heuristic scorers.
    pub targets: Vec<Target>,
    pub restrained: bool,
    pub protect_withdrawal: bool,
    pub mediated: Vec<UnitId>,
    pub withdrawal_tiles: Vec<TileId>,
    pub terminal_targets: Vec<UnitId>,
    pub terminal_arrivals: Vec<(UnitId, TileId)>,
    pub defensive_tiles: Vec<TileId>,
    pub movement_tiles: Vec<TileId>,
    pub leader_post: Option<TileId>,
    pub campaign: bool,
    pub required_units: Vec<RecruitRequirement>,
    /// Explicit mechanical transition dependencies supplied by Campaigner.
    pub transition_sensitive: Vec<(UnitId, Vec<TileId>)>,
    pub pending_arrivals: bool,
    pub leader_intents: Vec<(UnitId, Option<bool>)>,
}
impl TacticalOrders {
    pub(crate) fn promotion(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Option<usize> {
        let (_, kind) = self.promotions.iter().find(|(id, _)| id.0 == unit.id)?;
        game.promotion_options(unit)
            .into_iter()
            .find(|&option| game.rules().units[option].id == kind.0)
    }

    pub(crate) fn assigned_elsewhere(&self, id: u32, goal: &Target) -> bool {
        fn other(order: &ObjectiveOrder, id: u32) -> bool {
            match order.objective {
                TacticalObjective::GoTo {
                    actor,
                    destination: _,
                }
                | TacticalObjective::Occupy { actor, location: _ } => actor.0 != id,
                TacticalObjective::Escort { .. } | TacticalObjective::ClearPassage { .. } => false,
            }
        }
        self.objectives
            .iter()
            .any(|order| order.id == goal.id && other(order, id))
    }
    pub(crate) fn clearing(
        &self,
        game: &BattleView<'_, '_>,
    ) -> rustc_hash::FxHashMap<u32, rustc_hash::FxHashSet<usize>> {
        crate::ai::policies::objectives::actor_lanes(game, &self.passage_targets())
    }
    /// Complete the explicit passage contract without choosing planner heuristics.
    pub(crate) fn passage_targets(&self) -> std::borrow::Cow<'_, [Target]> {
        fn passage(order: &ObjectiveOrder) -> Option<(UnitId, &TileId)> {
            match &order.objective {
                TacticalObjective::ClearPassage { actor, destination } => {
                    Some((*actor, destination))
                }
                TacticalObjective::GoTo { .. }
                | TacticalObjective::Occupy { .. }
                | TacticalObjective::Escort { .. } => None,
            }
        }
        let missing: Vec<_> = self
            .objectives
            .iter()
            .filter_map(passage)
            .filter(|(actor, destination)| {
                !self.targets.iter().any(|goal| {
                    crate::ai::policies::objectives::actor_matches(&goal.actor, actor.0)
                        && goal.tiles.contains(&destination.0)
                })
            })
            .collect();
        if missing.is_empty() {
            return std::borrow::Cow::Borrowed(&self.targets);
        }
        let mut goals = self.targets.clone();
        for (actor, destination) in missing {
            let mut goal = crate::ai::policies::objectives::objective(
                "passage",
                ObjectiveKind::Visit,
                vec![destination.0.clone()],
                24.0,
            );
            goal.actor = Some(format!("u{}", actor.0));
            goals.push(goal.into());
        }
        std::borrow::Cow::Owned(goals)
    }
    pub(crate) fn rebuild_plan(
        &self,
        game: &BattleView<'_, '_>,
        id: u32,
        _required: bool,
    ) -> (Vec<usize>, Option<bool>) {
        let intent = self
            .leader_intents
            .iter()
            .find(|(actor, _)| actor.0 == id)
            .and_then(|(_, intent)| *intent);
        let targets = self
            .targets
            .iter()
            .filter(|goal| {
                goal.id.starts_with("rebuild:")
                    && crate::ai::policies::objectives::actor_matches(&goal.actor, id)
            })
            .flat_map(|goal| goal.tiles.iter().map(|tile| game.site(tile)))
            .collect();
        (targets, intent)
    }
    pub(crate) fn moving_actor(&self, id: u32) -> bool {
        self.objectives.iter().any(|order| {
            order.priority == ObjectivePriority::Critical
                && match order.objective {
                    TacticalObjective::GoTo {
                        actor,
                        destination: _,
                    }
                    | TacticalObjective::Occupy { actor, location: _ } => actor.0 == id,
                    TacticalObjective::Escort { .. } | TacticalObjective::ClearPassage { .. } => {
                        false
                    }
                }
        })
    }
    pub(crate) fn rebuild_objectives(&self, game: &BattleView<'_, '_>) -> Vec<Target> {
        self.targets
            .iter()
            .filter(|goal| {
                goal.id.starts_with("rebuild:")
                    && game.units().iter().any(|unit| {
                        game.belongs_to_faction(unit, game.observer_faction())
                            && crate::ai::policies::objectives::actor_matches(&goal.actor, unit.id)
                    })
            })
            .cloned()
            .collect()
    }
    pub(crate) fn protects(&self, id: u32) -> bool {
        self.constraints.iter().any(|constraint| match constraint {
            TacticalConstraint::Survive { actor } => actor.0 == id,
        })
    }
    pub(crate) fn important(&self, _game: &BattleView<'_, '_>, unit: &Unit) -> bool {
        self.protects(unit.id) || unit.is_leader() || unit.role.is_some_and(Role::is_escort)
    }
    pub(crate) fn arrives(&self, game: &BattleView<'_, '_>, unit: &Unit, tile: usize) -> bool {
        self.terminal_arrivals.iter().any(|(actor, destination)| {
            actor.0 == unit.id && destination.0 == game.board().ids[tile]
        })
    }
    pub(crate) fn terminal_attack(&self, target: &Unit) -> bool {
        self.terminal_targets.contains(&UnitId(target.id))
    }
    pub(crate) fn attack_allowed(
        &self,
        game: &BattleView<'_, '_>,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> bool {
        self.scoped
            .iter()
            .filter(|order| order.actor.0 == unit.id)
            .all(|order| order.allows_attack(game, from, target.tile))
            && (!self.protect_withdrawal
                || target.retreating != Some(true)
                    && (!self.mediated.contains(&UnitId(target.id))
                        || game.preview(unit, target, weapon, from).0 < target.hp))
    }
    pub(crate) fn lanes(&self, game: &BattleView<'_, '_>) -> rustc_hash::FxHashSet<usize> {
        self.withdrawal_tiles
            .iter()
            .map(|tile| game.site(&tile.0))
            .collect()
    }
    pub(crate) fn authored_destinations(
        &self,
        game: &BattleView<'_, '_>,
        unit: &Unit,
        destinations: Vec<usize>,
    ) -> Vec<usize> {
        let goals: Vec<_> = if unit.is_leader() && self.leader_post.is_some() {
            self.leader_post.iter().map(|t| game.site(&t.0)).collect()
        } else {
            self.defensive_tiles
                .iter()
                .map(|t| game.site(&t.0))
                .collect()
        };
        if goals.is_empty() {
            return destinations;
        }
        let routes = game.routes(unit.kind, &goals);
        let limit = destinations
            .iter()
            .map(|&t| routes[t])
            .min()
            .unwrap_or(u16::MAX)
            .max(1);
        destinations
            .into_iter()
            .filter(|&t| routes[t] <= limit)
            .collect()
    }
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CampaignOrders {
    pub recruitment: RecruitmentOrders,
    pub tactics: TacticalOrders,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlanningStatus {
    Fulfilled,
    Deferred,
    Unmet,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanningReason {
    InactiveActor,
    ActorLost,
    InsufficientGuards,
    RecruitmentUnavailable,
    BlockedAccess,
    ConflictingDuty,
    UnsafeFallback,
    SearchBudget,
    NotReached,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DutyReport {
    pub id: String,
    pub actor: Option<UnitId>,
    pub status: PlanningStatus,
    pub reason: Option<PlanningReason>,
    pub guards: Vec<UnitId>,
    pub incoming: Option<i32>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningReport {
    pub duties: Vec<DutyReport>,
    pub mandatory_recruits: Vec<UnitId>,
    pub candidates: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnDiagnostics {
    pub recruitment: RecruitmentReport,
    pub orders: CampaignOrders,
    pub report: PlanningReport,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentReport {
    pub recruited: Vec<UnitId>,
    pub mandatory: Vec<UnitId>,
    pub requirements: Vec<RecruitmentShortfall>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecruitmentShortfall {
    pub selector: UnitSelector,
    pub missing: usize,
    pub reason: PlanningReason,
}
pub struct TurnPlan {
    pub commands: Vec<hex_command_battle::ActionProposal>,
    pub recruitment: RecruitmentReport,
    pub planning: PlanningReport,
}
