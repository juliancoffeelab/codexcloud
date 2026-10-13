//! CampaignBlock duties, required-actor movement, safety and planner escalation.
use super::{
    Diplomacy, PlanningWorkspace,
    objectives::{actor_matches, individual, objective},
};
use crate::{data::*, logic::Game};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampaignBlock {
    Objectives,
    Adaptive,
    ArenaOnly,
}
impl CampaignBlock {
    pub const fn supports_campaign(self) -> bool {
        match self {
            Self::Objectives | Self::Adaptive => true,
            Self::ArenaOnly => false,
        }
    }
    pub(super) fn restrained(self, game: &Game<'_>, workspace: &PlanningWorkspace) -> bool {
        match self {
            Self::Objectives | Self::ArenaOnly => workspace.diplomacy == Diplomacy::Restrained,
            Self::Adaptive => restrained(game, workspace),
        }
    }
    pub(super) fn planner(self, game: &Game<'_>, selected: super::Planner) -> super::Planner {
        fn adaptive(block: CampaignBlock) -> bool {
            match block {
                CampaignBlock::Adaptive => true,
                CampaignBlock::Objectives | CampaignBlock::ArenaOnly => false,
            }
        }
        if adaptive(self) && crisis(game) {
            super::Planner::Mixed
        } else {
            selected
        }
    }
    pub(super) fn queue_mode(self, game: &Game<'_>) -> super::planner::QueueMode {
        fn adaptive(block: CampaignBlock) -> bool {
            match block {
                CampaignBlock::Adaptive => true,
                CampaignBlock::Objectives | CampaignBlock::ArenaOnly => false,
            }
        }
        if game.campaign.is_some() && adaptive(self) {
            super::planner::QueueMode::CampaignBlock
        } else {
            super::planner::QueueMode::Arena
        }
    }
    pub(super) fn goals(self, game: &Game<'_>, provided: Option<&[Objective]>) -> Vec<Objective> {
        match self {
            Self::Adaptive => policy_goals(game, provided),
            Self::Objectives | Self::ArenaOnly => individual(game.policy_objectives(provided)),
        }
    }
}
impl Game<'_> {
    /// Mission tools are required independently of the combat recruitment style.
    pub(super) fn required_recruit(&self, side: Side) -> Option<Family> {
        fn specialist(goal: &Goal) -> Option<&str> {
            match goal {
                Goal::Seal { unit_type, .. } => unit_type.as_deref(),
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Control { .. }
                | Goal::Event { .. }
                | Goal::Hold { .. }
                | Goal::Vigil { .. }
                | Goal::Refuge { .. } => None,
            }
        }
        if side != Side::Player {
            return None;
        }
        let mission = self.mission.as_ref()?;
        let pool = self.recruitment_options(side);
        mission
            .goals
            .iter()
            .filter(|g| !self.goal_done(g))
            .find_map(|goal| {
                let family = self.rules.units[self.kind(specialist(goal)?)].family;
                (!self
                    .units
                    .iter()
                    .any(|u| u.side == side && self.rules.units[u.kind].family == family)
                    && pool
                        .iter()
                        .any(|&kind| self.rules.units[kind].family == family))
                .then_some(family)
            })
    }
    pub fn essential(&self, u: &Unit) -> bool {
        self.rules.units[u.kind].leader
            || u.role == Some(Role::Courier)
            || (u.side == Side::Player
                && self.mission.as_ref().is_some_and(|m| {
                    u.name
                        .as_ref()
                        .is_some_and(|n| m.required_companions.contains(n))
                }))
    }
    pub fn can_pursue(&self, u: &Unit, g: &Objective) -> bool {
        fn is_objective_kind_escort_escape_vigil(value: &ObjectiveKind) -> bool {
            match value {
                ObjectiveKind::Escort | ObjectiveKind::Escape | ObjectiveKind::Vigil => true,
                ObjectiveKind::Capture
                | ObjectiveKind::Defeat
                | ObjectiveKind::Visit
                | ObjectiveKind::Seal => false,
            }
        }

        (!is_objective_kind_escort_escape_vigil(&g.kind) || g.actor.is_some())
            && (g.actor.is_none() || actor_matches(&g.actor, u.id))
            && g.family
                .as_ref()
                .is_none_or(|f| self.rules.units[u.kind].family == *f)
    }

    /// Personal duties take precedence over incidental nearby resources.
    pub(super) fn assigned_duty(&self, unit: &Unit, goal: &Objective) -> bool {
        self.can_pursue(unit, goal) && (goal.actor.is_some() || goal.family.is_some())
    }
    pub fn reserved(&self, u: &Unit, goals: &[Objective]) -> HashSet<usize> {
        fn is_objective_kind_escort_escape_vigil(value: &ObjectiveKind) -> bool {
            match value {
                ObjectiveKind::Escort | ObjectiveKind::Escape | ObjectiveKind::Vigil => true,
                ObjectiveKind::Capture
                | ObjectiveKind::Defeat
                | ObjectiveKind::Visit
                | ObjectiveKind::Seal => false,
            }
        }

        goals
            .iter()
            .filter(|g| {
                (g.actor.is_some()
                    || g.family.is_some()
                    || is_objective_kind_escort_escape_vigil(&g.kind))
                    && !self.can_pursue(u, g)
            })
            .flat_map(|g| g.tiles.iter().map(|t| self.site(t)))
            .collect()
    }
    pub fn objectives(&self, side: Side) -> Vec<Objective> {
        fn is_trigger_visit(value: &Trigger) -> bool {
            match value {
                Trigger::Visit { .. } => true,
                Trigger::Round { .. } | Trigger::Control { .. } => false,
            }
        }
        fn is_goal_event(value: &Goal) -> bool {
            match value {
                Goal::Event { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Control { .. }
                | Goal::Seal { .. }
                | Goal::Hold { .. }
                | Goal::Vigil { .. }
                | Goal::Refuge { .. } => false,
            }
        }
        fn is_goal_control_hold_refuge_vigil(value: &Goal) -> bool {
            match value {
                Goal::Control { .. }
                | Goal::Hold { .. }
                | Goal::Refuge { .. }
                | Goal::Vigil { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Seal { .. }
                | Goal::Event { .. } => false,
            }
        }

        fn describe(
            game: &Game<'_>,
            goal: &Goal,
            side: Side,
        ) -> (ObjectiveKind, Option<String>, Option<Family>, bool, bool) {
            fn actor(game: &Game<'_>, side: Side, role: Role) -> Option<String> {
                game.units
                    .iter()
                    .find(|u| u.side == side && u.role == Some(role))
                    .map(|u| format!("u{}", u.id))
            }
            match goal {
                Goal::Escort { .. } => (
                    ObjectiveKind::Escort,
                    actor(game, side, Role::Courier),
                    None,
                    false,
                    false,
                ),
                Goal::Escape { .. } => (
                    ObjectiveKind::Escape,
                    actor(game, side, Role::Leader),
                    None,
                    false,
                    false,
                ),
                Goal::Vigil { companion, .. } => (
                    ObjectiveKind::Vigil,
                    game.units
                        .iter()
                        .find(|u| u.side == side && u.name.as_ref() == Some(companion))
                        .map(|u| format!("u{}", u.id)),
                    None,
                    false,
                    false,
                ),
                Goal::Seal { unit_type, .. } => (
                    ObjectiveKind::Seal,
                    None,
                    unit_type
                        .as_ref()
                        .map(|k| game.rules.units[game.kind(k)].family),
                    false,
                    false,
                ),
                Goal::Event { .. } => (ObjectiveKind::Visit, None, None, false, false),
                Goal::Defeat { .. } => (ObjectiveKind::Defeat, None, None, false, false),
                Goal::Control { .. } => (ObjectiveKind::Capture, None, None, false, false),
                Goal::Hold { .. } => (ObjectiveKind::Capture, None, None, true, false),
                Goal::Refuge { .. } => {
                    let escalated = game.campaign.as_ref().unwrap().escalated();
                    (
                        if escalated {
                            ObjectiveKind::Defeat
                        } else {
                            ObjectiveKind::Capture
                        },
                        None,
                        None,
                        true,
                        escalated,
                    )
                }
            }
        }
        fn discard_completed(goal: &Goal) -> bool {
            match goal {
                Goal::Seal { .. } | Goal::Event { .. } | Goal::Defeat { .. } => true,
                Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Vigil { .. }
                | Goal::Control { .. }
                | Goal::Hold { .. }
                | Goal::Refuge { .. } => false,
            }
        }

        let mut goals = vec![];
        if let Some(mission) = &self.mission {
            let state = self.campaign.as_ref().unwrap();
            if side == Side::Player {
                for (index, g) in mission.goals.iter().enumerate() {
                    if self.goal_done(g) && discard_completed(g) {
                        continue;
                    }
                    let mut tiles = self.active_goal_tiles(g);
                    let (kind, actor, family, hold, all_enemies) = describe(self, g, side);
                    fn defeat(kind: ObjectiveKind) -> bool {
                        match kind {
                            ObjectiveKind::Defeat => true,
                            ObjectiveKind::Capture
                            | ObjectiveKind::Visit
                            | ObjectiveKind::Seal
                            | ObjectiveKind::Escort
                            | ObjectiveKind::Escape
                            | ObjectiveKind::Vigil => false,
                        }
                    }
                    if defeat(kind) {
                        // An escalated refuge is no longer a location to hold.
                        // Defeat duties target opponents, never the former shelter.
                        tiles.clear();
                        tiles.extend(
                            self.units
                                .iter()
                                .filter(|u| {
                                    u.side != side
                                        && (all_enemies || self.rules.units[u.kind].leader)
                                })
                                .map(|u| self.board.ids[u.tile].clone()),
                        );
                    }
                    if !tiles.is_empty() {
                        let mut result = objective(&format!("required:{index}"), kind, tiles, 24.0);
                        result.actor = actor;
                        result.family = family;
                        result.hold = Some(hold);
                        goals.push(result);
                    }
                }
                for e in &mission.events {
                    if is_trigger_visit(&e.trigger)
                        && !state.marked(&format!("event:{}", e.id))
                        && !mission
                            .goals
                            .iter()
                            .any(|g| is_goal_event(g) && g.event() == Some(&e.id))
                    {
                        goals.push(objective(
                            &format!("optional:{}", e.id),
                            ObjectiveKind::Visit,
                            vec![match &e.trigger {
                                Trigger::Visit { tile } => tile.clone(),
                                Trigger::Round { .. } | Trigger::Control { .. } => {
                                    unreachable!("visit event classification")
                                }
                            }],
                            4.0,
                        ));
                    }
                }
            } else {
                if let (Some(post), Some(leader)) = (
                    &mission.enemy_leader_post,
                    self.units
                        .iter()
                        .find(|u| u.side == side && self.rules.units[u.kind].leader),
                ) {
                    let mut o = objective(
                        "authored-post",
                        ObjectiveKind::Capture,
                        vec![post.clone()],
                        80.0,
                    );
                    o.actor = Some(format!("u{}", leader.id));
                    goals.push(o);
                }
                for (i, g) in mission.goals.iter().enumerate() {
                    if is_goal_control_hold_refuge_vigil(g) {
                        let mut o = objective(
                            &format!("deny:{i}"),
                            ObjectiveKind::Capture,
                            self.active_goal_tiles(g),
                            24.0,
                        );
                        o.hold = Some(true);
                        goals.push(o);
                    }
                }
            }
        }
        for tile in 0..self.terrain.len() {
            if self.terrain_is(tile, Terrain::Village) && self.owners[tile] != Some(side) {
                goals.push(objective(
                    &format!("village:{}", self.board.ids[tile]),
                    ObjectiveKind::Capture,
                    vec![self.board.ids[tile].clone()],
                    8.0,
                ));
            }
        }
        if !goals.iter().any(|g| g.kind == ObjectiveKind::Defeat) {
            goals.push(objective(
                "opposing-leader",
                ObjectiveKind::Defeat,
                self.units
                    .iter()
                    .filter(|u| u.side != side && self.rules.units[u.kind].leader)
                    .map(|u| self.board.ids[u.tile].clone())
                    .collect(),
                12.0,
            ));
        }
        goals.retain(|g| !g.tiles.is_empty());
        goals
    }
    pub fn support_objective(&self, u: &Unit, g: &Objective) -> Objective {
        let mut result = g.clone();
        if self.can_pursue(u, g) {
            return result;
        }
        let reserved = self.reserved(u, std::slice::from_ref(g));
        let mut seen = HashSet::default();
        result.tiles = g
            .tiles
            .iter()
            .flat_map(|t| self.board.adjacent[self.site(t)].iter().copied())
            .filter(|&t| {
                seen.insert(t) && !reserved.contains(&t) && self.cost(u.kind, t) != u16::MAX
            })
            .map(|t| self.board.ids[t].clone())
            .collect();
        result.weight *= 0.35;
        result
    }
    /// Identify friendly blockers on a required actor's next movement corridor.
    /// Escorts clear the path rather than camping beside its destination forever.
    pub(super) fn actor_lanes(&self, goals: &[Objective]) -> HashMap<u32, HashSet<usize>> {
        let mut lanes: HashMap<u32, HashSet<usize>> = HashMap::default();
        for goal in goals.iter().filter(|g| actor_route(g)) {
            let Some(actor) = self
                .units
                .iter()
                .filter(|u| {
                    u.side == self.turn
                        && if goal.actor.is_some() {
                            actor_matches(&goal.actor, u.id)
                        } else {
                            self.can_pursue(u, goal)
                        }
                })
                .min_by_key(|u| {
                    let targets: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
                    self.routes(u.kind, &targets)[u.tile]
                })
            else {
                continue;
            };
            let targets: Vec<_> = goal.tiles.iter().map(|t| self.site(t)).collect();
            if targets.contains(&actor.tile) {
                continue;
            }
            let mut penalty = 2;
            let mut route = self.occupied_routes(actor, &targets, true);
            let available = if actor.moved {
                0
            } else {
                self.rules.units[actor.kind].r#move as u16
            };
            if route[actor.tile] > available {
                let short = self.occupied_routes_with_penalty(actor, &targets, Some(0));
                let arrival_cost = targets
                    .iter()
                    .map(|&tile| self.cost(actor.kind, tile))
                    .min()
                    .unwrap_or(0);
                if short[actor.tile] <= available.saturating_add(arrival_cost) {
                    route = short;
                    penalty = 0;
                }
            }
            if route[actor.tile] == u16::MAX {
                continue;
            }
            // Equal route values are valid on zero-cost terrain. Search admissible
            // edges with visited parents, rather than requiring strict descent.
            let mut parents = vec![usize::MAX; self.terrain.len()];
            parents[actor.tile] = actor.tile;
            let mut pending = std::collections::VecDeque::from([actor.tile]);
            let mut endpoint = None;
            while let Some(current) = pending.pop_front() {
                if targets.contains(&current) {
                    endpoint = Some(current);
                    break;
                }
                let mut next: Vec<_> =
                    self.board.adjacent[current]
                        .iter()
                        .copied()
                        .filter(|&tile| {
                            if parents[tile] != usize::MAX || route[tile] == u16::MAX {
                                return false;
                            }
                            let occupied = self
                                .at(tile)
                                .is_some_and(|u| u.id != actor.id && u.side == actor.side);
                            let entering = self
                                .cost(actor.kind, tile)
                                .saturating_add(if occupied { penalty } else { 0 });
                            route[tile].saturating_add(entering) <= route[current]
                        })
                        .collect();
                next.sort_by_key(|&tile| route[tile].saturating_add(self.cost(actor.kind, tile)));
                for tile in next {
                    parents[tile] = current;
                    pending.push_back(tile);
                }
            }
            let Some(mut current) = endpoint else {
                continue;
            };
            let mut path = vec![current];
            while current != actor.tile {
                current = parents[current];
                path.push(current);
            }
            path.reverse();
            let mut spent = 0;
            for &tile in path.iter().skip(1) {
                spent += self.cost(actor.kind, tile) as usize;
                if spent > self.rules.units[actor.kind].r#move {
                    break;
                }
                if let Some(blocker) = self
                    .at(tile)
                    .filter(|u| u.side == actor.side && u.id != actor.id)
                {
                    lanes
                        .entry(blocker.id)
                        .or_default()
                        .extend(path.iter().copied());
                }
            }
        }
        lanes
    }
    pub fn finishes_objective(&self, u: &Unit, tile: usize, goals: &[Objective]) -> bool {
        fn is_objective_kind_escort_escape(value: &ObjectiveKind) -> bool {
            match value {
                ObjectiveKind::Escort | ObjectiveKind::Escape => true,
                ObjectiveKind::Capture
                | ObjectiveKind::Defeat
                | ObjectiveKind::Visit
                | ObjectiveKind::Seal
                | ObjectiveKind::Vigil => false,
            }
        }
        fn is_goal_escort(value: &Goal) -> bool {
            match value {
                Goal::Escort { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escape { .. }
                | Goal::Control { .. }
                | Goal::Seal { .. }
                | Goal::Event { .. }
                | Goal::Hold { .. }
                | Goal::Vigil { .. }
                | Goal::Refuge { .. } => false,
            }
        }
        fn is_goal_escape(value: &Goal) -> bool {
            match value {
                Goal::Escape { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Control { .. }
                | Goal::Seal { .. }
                | Goal::Event { .. }
                | Goal::Hold { .. }
                | Goal::Vigil { .. }
                | Goal::Refuge { .. } => false,
            }
        }

        self.campaign.is_some()
            && u.side == Side::Player
            && goals.iter().any(|g| {
                actor_matches(&g.actor, u.id)
                    && is_objective_kind_escort_escape(&g.kind)
                    && g.tiles.contains(&self.board.ids[tile])
            })
            && self.mission.as_ref().unwrap().goals.iter().all(|g| {
                self.goal_done(g)
                    || ((is_goal_escort(g) && u.role == Some(Role::Courier)
                        || is_goal_escape(g) && u.role == Some(Role::Leader))
                        && g.tile() == Some(&self.board.ids[tile]))
            })
    }
    pub fn authored_destinations(&self, u: &Unit, dest: Vec<usize>) -> Vec<usize> {
        if u.side != Side::Enemy || self.mission.is_none() {
            return dest;
        }
        let mission = self.mission.as_ref().unwrap();
        let post = if self.rules.units[u.kind].leader {
            mission.enemy_leader_post.as_ref()
        } else {
            None
        };
        let goals = if let Some(post) = post {
            vec![self.site(post)]
        } else if mission.defend_objectives {
            self.territory()
        } else {
            vec![]
        };
        if goals.is_empty() {
            return dest;
        }
        let routes = self.routes(u.kind, &goals);
        let limit = dest
            .iter()
            .map(|&t| routes[t])
            .min()
            .unwrap_or(u16::MAX)
            .max(1);
        dest.into_iter().filter(|&t| routes[t] <= limit).collect()
    }
    pub fn territory(&self) -> Vec<usize> {
        fn is_goal_control(value: &Goal) -> bool {
            match value {
                Goal::Control { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Seal { .. }
                | Goal::Event { .. }
                | Goal::Hold { .. }
                | Goal::Vigil { .. }
                | Goal::Refuge { .. } => false,
            }
        }
        fn is_goal_hold(value: &Goal) -> bool {
            match value {
                Goal::Hold { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Control { .. }
                | Goal::Seal { .. }
                | Goal::Event { .. }
                | Goal::Vigil { .. }
                | Goal::Refuge { .. } => false,
            }
        }
        fn is_goal_refuge(value: &Goal) -> bool {
            match value {
                Goal::Refuge { .. } => true,
                Goal::Defeat { .. }
                | Goal::Escort { .. }
                | Goal::Escape { .. }
                | Goal::Control { .. }
                | Goal::Seal { .. }
                | Goal::Event { .. }
                | Goal::Hold { .. }
                | Goal::Vigil { .. } => false,
            }
        }

        self.mission.as_ref().map_or(vec![], |m| {
            m.goals
                .iter()
                .flat_map(|g| {
                    if is_goal_control(g) {
                        g.tiles().to_vec()
                    } else if is_goal_hold(g)
                        || (is_goal_refuge(g) && !self.campaign.as_ref().unwrap().escalated())
                    {
                        vec![g.tile().unwrap().clone()]
                    } else {
                        vec![]
                    }
                })
                .map(|t| self.site(&t))
                .collect()
        })
    }
    pub fn protects_withdrawal(&self, side: Side, restrained: bool) -> bool {
        restrained
            && side == Side::Player
            && self.mission.as_ref().is_some_and(|m| m.refuge.is_some())
            && !self.campaign.as_ref().unwrap().escalated()
    }
    pub fn diplomacy_allows(
        &self,
        u: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
        restrained: bool,
    ) -> bool {
        !self.protects_withdrawal(u.side, restrained)
            || (target.retreating != Some(true)
                && (!self.mediation_at_risk(target)
                    || self.preview(u, target, weapon, from).0 < target.hp))
    }
    pub fn withdrawal_lanes(&self, side: Side, restrained: bool) -> HashSet<usize> {
        let mut lanes = HashSet::default();
        if !self.protects_withdrawal(side, restrained) {
            return lanes;
        }
        for u in self
            .units
            .iter()
            .filter(|u| u.side != side && u.retreating == Some(true))
        {
            let mut homes: Vec<_> = (0..self.terrain.len())
                .filter(|&t| {
                    self.owners[t] == Some(u.side)
                        && (self.terrain_is(t, Terrain::Keep)
                            || self.terrain_is(t, Terrain::Village))
                })
                .collect();
            if homes.is_empty() {
                homes = (0..self.terrain.len())
                    .filter(|&t| self.board.positions[t].0 == self.cols - 1)
                    .collect();
            }
            let routes = self.routes(u.kind, &homes);
            for &t in &self.board.adjacent[u.tile] {
                if routes[t] < routes[u.tile] && self.cost(u.kind, t) != u16::MAX {
                    lanes.insert(t);
                }
            }
        }
        lanes
    }
    pub(super) fn leader_rebuild_plan(
        &self,
        id: u32,
        required: bool,
    ) -> (Vec<usize>, Option<bool>) {
        let Some(u) = self.unit(id) else {
            return (vec![], None);
        };
        if !self.rules.units[u.kind].leader || required {
            return (vec![], u.rebuilding);
        }
        let keeps: Vec<_> = (0..self.terrain.len())
            .filter(|&t| self.terrain_is(t, Terrain::Keep) && self.owners[t] == Some(u.side))
            .collect();
        let options = self.recruitment_options(u.side);
        if keeps.is_empty() || options.is_empty() {
            return (vec![], None);
        }
        let count = self.ordinary(&u).count();
        let rebuilding = if self.required_recruit(u.side).is_some() {
            true
        } else if count >= 5 {
            false
        } else if count <= 2 {
            true
        } else {
            u.rebuilding.unwrap_or(true)
        };
        if rebuilding
            && (self.income(u.side) > 0
                || options
                    .iter()
                    .any(|&k| self.rules.units[k].cost <= self.gold[u.side.index()]))
        {
            (keeps, Some(rebuilding))
        } else {
            (vec![], Some(rebuilding))
        }
    }
    pub(super) fn leader_rebuild(&mut self, id: u32, required: bool) -> Vec<usize> {
        let (keeps, intent) = self.leader_rebuild_plan(id, required);
        if let Some(index) = self.index(id) {
            self.units[index].rebuilding = intent;
        }
        keeps
    }
    pub(super) fn ordinary(&self, u: &Unit) -> impl Iterator<Item = &Unit> {
        let side = u.side;
        self.units.iter().filter(move |o| {
            o.side == side
                && !self.rules.units[o.kind].leader
                && o.role != Some(Role::Courier)
                && self.rules.units[o.kind].healing == 0
        })
    }
    pub fn proximity(&self, side: Side, tile: usize) -> u16 {
        self.units
            .iter()
            .filter(|o| o.side != side)
            .map(|o| self.board.distance(tile, o.tile))
            .min()
            .unwrap_or(20)
    }
    pub fn leader_may_advance(
        &self,
        u: &Unit,
        tile: usize,
        recruiting: bool,
        required: bool,
    ) -> bool {
        if tile == u.tile || required || !self.units.iter().any(|o| o.side != u.side) {
            return true;
        }
        let before = self.proximity(u.side, u.tile);
        let after = self.proximity(u.side, tile);
        if u.rebuilding == Some(true) && recruiting && before > 2 {
            return false;
        }
        if after >= before {
            return true;
        }
        let escorts = self.ordinary(u).filter(|o| {
            o.hp as f64 >= self.rules.units[o.kind].hp as f64 / 2.0
                && self.board.distance(tile, o.tile) <= 2
        });
        let (count, at_front) = escorts.fold((0, false), |(count, front), unit| {
            (
                count + 1,
                front || self.proximity(u.side, unit.tile) <= after,
            )
        });
        count >= 2 && at_front
    }
}
pub(super) fn policy_goals(game: &Game<'_>, provided: Option<&[Objective]>) -> Vec<Objective> {
    fn is_goal_escort_escape(value: &Goal) -> bool {
        match value {
            Goal::Escort { .. } | Goal::Escape { .. } => true,
            Goal::Defeat { .. }
            | Goal::Control { .. }
            | Goal::Seal { .. }
            | Goal::Event { .. }
            | Goal::Hold { .. }
            | Goal::Vigil { .. }
            | Goal::Refuge { .. } => false,
        }
    }

    let mut goals = game.policy_objectives(provided);
    let escort_finish = provided.is_none()
        && game.turn == Side::Player
        && game.mission.as_ref().is_some_and(|mission| {
            (mission.refuge.is_none() || game.campaign.as_ref().is_some_and(|c| c.escalated()))
                && mission
                    .goals
                    .iter()
                    .any(|g| !game.goal_done(g) && is_goal_escort_escape(g))
                && mission
                    .goals
                    .iter()
                    .all(|g| game.goal_done(g) || is_goal_escort_escape(g))
        });
    if escort_finish {
        for goal in goals.iter_mut().filter(|g| g.id == "opposing-leader") {
            goal.weight = 48.0;
        }
    }
    individual(goals)
}
pub(super) fn heavy_campaign(game: &Game<'_>) -> bool {
    fn is_goal_defeat(value: &Goal) -> bool {
        match value {
            Goal::Defeat { .. } => true,
            Goal::Escort { .. }
            | Goal::Escape { .. }
            | Goal::Control { .. }
            | Goal::Seal { .. }
            | Goal::Event { .. }
            | Goal::Hold { .. }
            | Goal::Vigil { .. }
            | Goal::Refuge { .. } => false,
        }
    }
    fn is_goal_seal(value: &Goal) -> bool {
        match value {
            Goal::Seal { .. } => true,
            Goal::Defeat { .. }
            | Goal::Escort { .. }
            | Goal::Escape { .. }
            | Goal::Control { .. }
            | Goal::Event { .. }
            | Goal::Hold { .. }
            | Goal::Vigil { .. }
            | Goal::Refuge { .. } => false,
        }
    }

    game.campaign.as_ref().is_some_and(|state| {
        game.rules.campaigns[&state.id]
            .missions
            .iter()
            .any(|mission| {
                mission.goals.iter().any(|g| is_goal_defeat(g))
                    && mission
                        .goals
                        .iter()
                        .any(|g| is_goal_seal(g) && g.unit_type().is_none())
            })
    })
}
pub(super) fn crisis(game: &Game<'_>) -> bool {
    if heavy_campaign(game) {
        return game.turn == Side::Player;
    }
    game.turn == Side::Player
        && game.campaign.is_some()
        && game.units.iter().any(|u| {
            u.side == Side::Player
                && game.essential(u)
                && u.hp * 2 <= game.rules.units[u.kind].hp
                && game.proximity(u.side, u.tile) <= 5
        })
}

pub(super) fn restrained(game: &Game<'_>, workspace: &PlanningWorkspace) -> bool {
    workspace.diplomacy == super::Diplomacy::Restrained
        || !heavy_campaign(game)
            && game.turn == Side::Player
            && game.mission.as_ref().is_some_and(|m| m.refuge.is_some())
            && game.campaign.as_ref().is_some_and(|c| !c.escalated())
}
pub(super) fn terminal_attack(game: &Game<'_>, unit: &Unit, target: &Unit) -> bool {
    fn kill_goal(game: &Game<'_>, goal: &Goal) -> Option<bool> {
        match goal {
            Goal::Defeat { .. } => Some(false),
            Goal::Refuge { .. } => game.campaign.as_ref().unwrap().escalated().then_some(true),
            Goal::Escort { .. }
            | Goal::Escape { .. }
            | Goal::Control { .. }
            | Goal::Seal { .. }
            | Goal::Event { .. }
            | Goal::Hold { .. }
            | Goal::Vigil { .. } => None,
        }
    }
    if !game.rules.units[target.kind].leader {
        return false;
    }
    if game.campaign.is_none() {
        return true;
    }
    unit.side == Side::Player
        && game.mission.as_ref().is_some_and(|mission| {
            mission
                .goals
                .iter()
                .all(|g| game.goal_done(g) || kill_goal(game, g).is_some())
                && game
                    .units
                    .iter()
                    .filter(|u| {
                        u.side != unit.side
                            && (game.rules.units[u.kind].leader
                                || mission
                                    .goals
                                    .iter()
                                    .any(|g| kill_goal(game, g) == Some(true)))
                    })
                    .all(|u| u.id == target.id)
        })
}

/// Whether an actor must be routed through current occupancy and allies must yield.
pub(super) fn actor_route(goal: &Objective) -> bool {
    fn required_arrival(goal: &Objective) -> bool {
        match goal.kind {
            ObjectiveKind::Escort | ObjectiveKind::Escape | ObjectiveKind::Vigil => true,
            ObjectiveKind::Capture => goal.actor.is_some(),
            ObjectiveKind::Seal => goal.family.is_some(),
            ObjectiveKind::Defeat | ObjectiveKind::Visit => false,
        }
    }
    required_arrival(goal)
}
