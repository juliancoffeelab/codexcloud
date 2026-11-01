//! Explicit, turn-local contracts between the three fixed AI blocks.
use crate::ai::ids::{TileId, UnitTypeId};
use crate::logic::Action as Command;
use crate::reuse::Vec;
#[cfg(test)]
use crate::reuse::vec;
use crate::{ai::context::BattleView, data::*};
use serde::{Deserialize, Serialize};

pub(crate) fn text<A: smallvec::Array<Item = u8>>(
    args: std::fmt::Arguments<'_>,
) -> smallstr::SmallString<A> {
    use std::fmt::Write;
    let mut value = smallstr::SmallString::new();
    value.write_fmt(args).expect("formatting a planner key");
    value
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type GoalKey = smallstr::SmallString<[u8; 32]>;
#[cfg(target_arch = "wasm32")]
pub(crate) use super::keys::GoalKey;
pub(crate) type SiteKey = smallstr::SmallString<[u8; 16]>;
#[derive(Clone, Debug)]
pub(crate) struct AuthorGoal {
    pub id: GoalKey,
    pub kind: ObjectiveKind,
    pub tiles: super::sites::Sites,
    pub weight: f64,
    pub actor: Option<SiteKey>,
    pub family: Option<Family>,
    pub hold: Option<bool>,
}
impl From<&crate::data::Objective> for AuthorGoal {
    fn from(value: &crate::data::Objective) -> Self {
        Self {
            id: value.id.as_str().into(),
            kind: value.kind,
            tiles: value.tiles.iter().map(|s| s.as_str().into()).collect(),
            weight: value.weight,
            actor: value.actor.as_deref().map(Into::into),
            family: value.family,
            hold: value.hold,
        }
    }
}
pub(crate) fn authored(
    id: &str,
    kind: ObjectiveKind,
    tiles: impl IntoIterator<Item = impl AsRef<str>>,
    weight: f64,
) -> AuthorGoal {
    AuthorGoal {
        id: id.into(),
        kind,
        tiles: tiles.into_iter().map(|s| s.as_ref().into()).collect(),
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}
pub(crate) fn key(args: std::fmt::Arguments<'_>) -> GoalKey {
    #[cfg(not(target_arch = "wasm32"))]
    {
        text(args)
    }
    #[cfg(target_arch = "wasm32")]
    {
        GoalKey::formatted(args)
    }
}
/// Generic heuristic hints supplement typed duties without exposing mission kinds.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetKind {
    Capture,
    Defeat,
    Visit,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    #[cfg_attr(feature = "wasm", tsify(type = "string"))]
    pub id: GoalKey,
    pub kind: TargetKind,
    #[cfg_attr(feature = "wasm", tsify(type = "string[]"))]
    pub tiles: super::sites::Sites,
    pub weight: f64,
    #[cfg_attr(feature = "wasm", tsify(type = "string | null"))]
    pub actor: Option<smallstr::SmallString<[u8; 16]>>,
    pub family: Option<Family>,
    pub hold: Option<bool>,
}
impl From<AuthorGoal> for Target {
    fn from(value: AuthorGoal) -> Self {
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
impl From<crate::data::Objective> for Target {
    fn from(value: crate::data::Objective) -> Self {
        AuthorGoal::from(&value).into()
    }
}
pub(in crate::ai) fn target(
    id: &str,
    kind: TargetKind,
    tiles: impl IntoIterator<Item = impl AsRef<str>>,
    weight: f64,
) -> Target {
    Target {
        id: id.into(),
        kind,
        tiles: tiles.into_iter().map(|t| t.as_ref().into()).collect(),
        weight,
        actor: None,
        family: None,
        hold: None,
    }
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnitSelector {
    Type(UnitTypeId),
    Family(Family),
    Role(UnitRole),
}
impl UnitSelector {
    pub(crate) fn accepts(&self, game: &BattleView<'_, '_>, kind: usize) -> bool {
        let rule = &game.rules().units[kind];
        match self {
            Self::Type(id) => rule.id.as_ref() == id.0.as_str(),
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
    pub(crate) fn deficit(&self, game: &BattleView<'_, '_>) -> usize {
        match self {
            Self::EnsureUnits { selector, count } => count.saturating_sub(
                game.units()
                    .iter()
                    .filter(|u| u.side == game.turn() && selector.accepts(game, u.kind))
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
    #[cfg_attr(feature = "wasm", tsify(type = "string"))]
    pub id: GoalKey,
    pub priority: ObjectivePriority,
    pub objective: TacticalObjective,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TacticalOrders {
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
    pub(in crate::ai) fn promotion(&self, game: &BattleView<'_, '_>, unit: &Unit) -> Option<usize> {
        let (_, kind) = self.promotions.iter().find(|(id, _)| id.0 == unit.id)?;
        game.promotion_options(unit)
            .into_iter()
            .find(|&option| game.rules().units[option].id.as_ref() == kind.0.as_str())
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
            .any(|order| goal.id == order.id && other(order, id))
    }
    pub(crate) fn clearing(
        &self,
        game: &BattleView<'_, '_>,
    ) -> crate::reuse::Map<u32, crate::reuse::Set<usize>> {
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
        let missing: crate::reuse::Local<_> = self
            .objectives
            .iter()
            .filter_map(passage)
            .filter(|(actor, destination)| {
                !self.targets.iter().any(|goal| {
                    super::objectives::actor_matches(&goal.actor, actor.0)
                        && goal.tiles.iter().any(|t| t == &destination.0)
                })
            })
            .collect();
        if missing.is_empty() {
            return game.actor_lanes(&self.targets);
        }
        let mut goals = self.targets.clone();
        for (actor, destination) in missing {
            let mut goal = authored(
                "passage",
                ObjectiveKind::Visit,
                std::iter::once(destination.0.as_str()),
                24.0,
            );
            goal.actor = Some(text(format_args!("u{}", actor.0)));
            goals.push(goal.into());
        }
        game.actor_lanes(&goals)
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
                goal.id.starts_with("rebuild:") && super::objectives::actor_matches(&goal.actor, id)
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
                        unit.side == game.turn()
                            && super::objectives::actor_matches(&goal.actor, unit.id)
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
    pub(crate) fn important(&self, game: &BattleView<'_, '_>, unit: &Unit) -> bool {
        self.protects(unit.id)
            || game.rules().units[unit.kind].leader
            || unit.role == Some(Role::Courier)
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
        if self.protect_withdrawal && target.retreating == Some(true) {
            return false;
        }
        let protected = self.protects(unit.id);
        let mediated = self.protect_withdrawal && self.mediated.contains(&UnitId(target.id));
        if !protected && !mediated {
            return true;
        }
        let forecast = game.exchange(unit, target, weapon, from);
        (!protected || forecast.damage[1].round() as i32 == 0)
            && (!mediated || (forecast.damage[0].round() as i32) < target.hp)
    }
    pub(crate) fn lanes(&self, game: &BattleView<'_, '_>) -> crate::reuse::Set<usize> {
        self.withdrawal_tiles
            .iter()
            .map(|tile| game.site(&tile.0))
            .collect()
    }
    pub(crate) fn authored_destinations(
        &self,
        game: &BattleView<'_, '_>,
        unit: &Unit,
        destinations: &mut Vec<usize>,
    ) {
        let goals: Vec<_> = if game.rules().units[unit.kind].leader && self.leader_post.is_some() {
            self.leader_post.iter().map(|t| game.site(&t.0)).collect()
        } else {
            self.defensive_tiles
                .iter()
                .map(|t| game.site(&t.0))
                .collect()
        };
        if goals.is_empty() {
            return;
        }
        let routes = game.routes(unit.kind, &goals);
        let limit = destinations
            .iter()
            .map(|&t| routes[t])
            .min()
            .unwrap_or(u16::MAX)
            .max(1);
        destinations.retain(|&t| routes[t] <= limit);
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
    #[cfg_attr(feature = "wasm", tsify(type = "string"))]
    pub id: GoalKey,
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
    pub recruitment: RecruitmentReport,
    pub orders: CampaignOrders,
    pub report: PlanningReport,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentReport {
    pub recruited: Vec<UnitId>,
    pub mandatory: Vec<UnitId>,
    pub requirements: Vec<RecruitmentShortfall>,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecruitmentShortfall {
    pub selector: UnitSelector,
    pub missing: usize,
    pub reason: PlanningReason,
}
pub(crate) struct TurnPlan {
    pub commands: Vec<Command>,
    pub recruitment: RecruitmentReport,
    pub planning: PlanningReport,
}
