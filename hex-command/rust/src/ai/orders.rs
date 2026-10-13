//! Explicit, turn-local contracts between the three fixed AI blocks.
use crate::{data::*, logic::Game};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitSelector {
    Type(UnitTypeId),
    Family(Family),
    Role(UnitRole),
}
impl UnitSelector {
    pub(crate) fn accepts(&self, game: &Game<'_>, kind: usize) -> bool {
        let rule = &game.rules.units[kind];
        match self {
            Self::Type(id) => rule.id == id.0,
            Self::Family(family) => rule.family == *family,
            Self::Role(role) => rule.role == *role,
        }
    }
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecruitRequirement {
    EnsureUnits {
        selector: UnitSelector,
        count: usize,
    },
}
impl RecruitRequirement {
    pub(crate) fn deficit(&self, game: &Game<'_>) -> usize {
        match self {
            Self::EnsureUnits { selector, count } => count.saturating_sub(
                game.units
                    .iter()
                    .filter(|u| u.side == game.turn && selector.accepts(game, u.kind))
                    .count(),
            ),
        }
    }
    pub(crate) fn accepts(&self, game: &Game<'_>, kind: usize) -> bool {
        match self {
            Self::EnsureUnits { selector, count: _ } => selector.accepts(game, kind),
        }
    }
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentOrders {
    pub requirements: Vec<RecruitRequirement>,
    pub reserved: Vec<TileId>,
    pub rotate_pool: bool,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum BattleObjective {
    #[default]
    BattleToKill,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TacticalConstraint {
    Survive { actor: UnitId },
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ObjectivePriority {
    Normal,
    Important,
    Critical,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveOrder {
    pub id: String,
    pub priority: ObjectivePriority,
    pub objective: TacticalObjective,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TacticalOrders {
    pub battle: BattleObjective,
    pub constraints: Vec<TacticalConstraint>,
    pub objectives: Vec<ObjectiveOrder>,
    /// General combat/resource targets used by the existing heuristic scorers.
    pub targets: Vec<Objective>,
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
}
impl TacticalOrders {
    pub(crate) fn assigned_elsewhere(&self, id: u32, goal: &Objective) -> bool {
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
        game: &Game<'_>,
    ) -> rustc_hash::FxHashMap<u32, rustc_hash::FxHashSet<usize>> {
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
                    super::objectives::actor_matches(&goal.actor, actor.0)
                        && goal.tiles.contains(&destination.0)
                })
            })
            .collect();
        if missing.is_empty() {
            return game.actor_lanes(&self.targets);
        }
        let mut goals = self.targets.clone();
        for (actor, destination) in missing {
            let mut goal = super::objectives::objective(
                "passage",
                ObjectiveKind::Visit,
                vec![destination.0.clone()],
                24.0,
            );
            goal.actor = Some(format!("u{}", actor.0));
            goals.push(goal);
        }
        game.actor_lanes(&goals)
    }
    pub(crate) fn rebuild_plan(
        &self,
        game: &Game<'_>,
        id: u32,
        required: bool,
    ) -> (Vec<usize>, Option<bool>) {
        let Some(unit) = game.unit(id) else {
            return (vec![], None);
        };
        if !game.rules.units[unit.kind].leader || required {
            return (vec![], unit.rebuilding);
        }
        let keeps: Vec<_> = (0..game.terrain.len())
            .filter(|&tile| {
                game.terrain_is(tile, Terrain::Keep) && game.owners[tile] == Some(unit.side)
            })
            .collect();
        let options = game.recruitment_options(unit.side);
        if keeps.is_empty() || options.is_empty() {
            return (vec![], None);
        }
        let count = game.ordinary(unit).count();
        let rebuilding = if self.required_units.iter().any(|r| r.deficit(game) > 0) {
            true
        } else if count >= 5 {
            false
        } else if count <= 2 {
            true
        } else {
            unit.rebuilding.unwrap_or(true)
        };
        if rebuilding
            && (game.income(unit.side) > 0
                || options
                    .iter()
                    .any(|&kind| game.rules.units[kind].cost <= game.gold[unit.side.index()]))
        {
            (keeps, Some(rebuilding))
        } else {
            (vec![], Some(rebuilding))
        }
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
    pub(crate) fn rebuild_objectives(&self, game: &Game<'_>) -> Vec<Objective> {
        game.units
            .iter()
            .filter(|u| u.side == game.turn && game.rules.units[u.kind].leader)
            .filter_map(|unit| {
                let (keeps, _) = self.rebuild_plan(game, unit.id, self.moving_actor(unit.id));
                if keeps.is_empty() {
                    return None;
                }
                let mut goal = super::objectives::objective(
                    &format!("rebuild:u{}", unit.id),
                    ObjectiveKind::Capture,
                    keeps
                        .iter()
                        .map(|&tile| game.board.ids[tile].clone())
                        .collect(),
                    80.0,
                );
                goal.actor = Some(format!("u{}", unit.id));
                Some(goal)
            })
            .collect()
    }
    pub(crate) fn protects(&self, id: u32) -> bool {
        self.constraints.iter().any(|constraint| match constraint {
            TacticalConstraint::Survive { actor } => actor.0 == id,
        })
    }
    pub(crate) fn important(&self, game: &Game<'_>, unit: &Unit) -> bool {
        self.protects(unit.id)
            || game.rules.units[unit.kind].leader
            || unit.role == Some(Role::Courier)
    }
    pub(crate) fn arrives(&self, game: &Game<'_>, unit: &Unit, tile: usize) -> bool {
        self.terminal_arrivals
            .iter()
            .any(|(actor, destination)| actor.0 == unit.id && destination.0 == game.board.ids[tile])
    }
    pub(crate) fn terminal_attack(&self, target: &Unit) -> bool {
        self.terminal_targets.contains(&UnitId(target.id))
    }
    pub(crate) fn attack_allowed(
        &self,
        game: &Game<'_>,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> bool {
        let preview = || game.preview(unit, target, weapon, from);
        (!self.protects(unit.id) || preview().1 == 0)
            && (!self.protect_withdrawal
                || target.retreating != Some(true)
                    && (!self.mediated.contains(&UnitId(target.id)) || preview().0 < target.hp))
    }
    pub(crate) fn lanes(&self, game: &Game<'_>) -> rustc_hash::FxHashSet<usize> {
        self.withdrawal_tiles
            .iter()
            .map(|tile| game.site(&tile.0))
            .collect()
    }
    pub(crate) fn authored_destinations(
        &self,
        game: &Game<'_>,
        unit: &Unit,
        destinations: Vec<usize>,
    ) -> Vec<usize> {
        let goals: Vec<_> = if game.rules.units[unit.kind].leader && self.leader_post.is_some() {
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

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CampaignOrders {
    pub recruitment: RecruitmentOrders,
    pub tactics: TacticalOrders,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlanningStatus {
    Fulfilled,
    Deferred,
    Unmet,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
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
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DutyReport {
    pub id: String,
    pub actor: Option<UnitId>,
    pub status: PlanningStatus,
    pub reason: Option<PlanningReason>,
    pub guards: Vec<UnitId>,
    pub incoming: Option<i32>,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningReport {
    pub duties: Vec<DutyReport>,
    pub mandatory_recruits: Vec<UnitId>,
    pub candidates: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnDiagnostics {
    pub orders: CampaignOrders,
    pub report: PlanningReport,
}
