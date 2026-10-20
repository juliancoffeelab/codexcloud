//! Authored missions are interpreted here and emitted as explicit turn orders.
use super::CampaignerKind as CampaignBlock;
use crate::ai::context::BattleView;
use crate::ai::{
    Diplomacy, PlanningWorkspace,
    objectives::{actor_matches, objective},
    orders::*,
};
use crate::{data::*, logic::Game};
use rustc_hash::FxHashSet as HashSet;
impl CampaignBlock {
    pub const fn supports_campaign(self) -> bool {
        match self {
            Self::Mission => true,
            Self::Arena => false,
        }
    }
    pub(in crate::ai) fn restrained(self, game: &Game<'_>, workspace: &PlanningWorkspace) -> bool {
        match self {
            Self::Arena => workspace.diplomacy == Diplomacy::Restrained,
            Self::Mission => restrained(game, workspace),
        }
    }
    pub(in crate::ai) fn goals(
        self,
        game: &Game<'_>,
        provided: Option<&[Objective]>,
    ) -> Vec<Objective> {
        match self {
            Self::Mission => policy_goals(game, provided),
            Self::Arena => individual(game.policy_objectives(provided)),
        }
    }

    pub(in crate::ai) fn campaign_orders(
        self,
        game: &Game<'_>,
        workspace: &PlanningWorkspace,
    ) -> CampaignOrders {
        let restrained = self.restrained(game, workspace);
        let campaign = game.campaign.is_some();
        let authored_targets = match self {
            Self::Mission => self.goals(game, workspace.goals.as_deref()),
            Self::Arena => individual(game.policy_objectives(workspace.goals.as_deref())),
        };
        let mut tactics = TacticalOrders {
            targets: authored_targets.iter().cloned().map(Target::from).collect(),
            restrained,
            campaign,
            protect_withdrawal: game.protects_withdrawal(game.turn, restrained),
            withdrawal_tiles: game
                .withdrawal_lanes(game.turn, restrained)
                .into_iter()
                .map(|tile| TileId(game.board.ids[tile].clone()))
                .collect(),
            ..TacticalOrders::default()
        };
        if let Some(mission) = &game.mission {
            tactics.pending_arrivals = game.turn == Side::Player
                && mission.waves.iter().any(|wave| {
                    game.round >= wave.round
                        && !game
                            .campaign
                            .as_ref()
                            .unwrap()
                            .marked(&format!("wave:{}", wave.id))
                });
            for unit in game.units.iter().filter(|unit| unit.side == game.turn) {
                let mut sites = Vec::new();
                for goal in &mission.goals {
                    let eligible = match goal {
                        Goal::Escort { .. } => unit.role == Some(Role::Courier),
                        Goal::Escape { .. } => unit.role == Some(Role::Leader),
                        Goal::Vigil { .. }
                        | Goal::Hold {
                            companion: Some(_), ..
                        } => unit.name.as_ref() == goal.companion(),
                        Goal::Seal { .. } => goal.unit_type().is_none_or(|kind| {
                            game.rules.units[game.kind(kind)].family
                                == game.rules.units[unit.kind].family
                        }),
                        Goal::Defeat { .. }
                        | Goal::Control { .. }
                        | Goal::Event { .. }
                        | Goal::Hold {
                            companion: None, ..
                        }
                        | Goal::Refuge { .. } => false,
                    };
                    if eligible {
                        sites.extend(
                            goal.tile()
                                .into_iter()
                                .chain(goal.tiles())
                                .map(|tile| TileId(tile.clone())),
                        );
                    }
                }
                if !sites.is_empty() {
                    tactics.transition_sensitive.push((UnitId(unit.id), sites));
                }
            }
        }
        for unit in game
            .units
            .iter()
            .filter(|unit| unit.side == game.turn && game.rules.units[unit.kind].leader)
        {
            let escaping = tactics.targets.iter().any(|target| {
                authored_targets.iter().any(|authored| {
                    authored.id == target.id && authored.kind == ObjectiveKind::Escape
                }) && actor_matches(&target.actor, unit.id)
            });
            let (_, intent) = game.leader_rebuild_plan(unit.id, escaping);
            tactics.leader_intents.push((UnitId(unit.id), intent));
        }
        let mut recruitment = RecruitmentOrders::default();
        recruitment.rotate_pool = campaign || game.pools.is_some();
        if game.turn == Side::Player {
            if let Some(mission) = &game.mission {
                for goal in &mission.goals {
                    let specialist = match goal {
                        Goal::Seal { unit_type, .. } => {
                            unit_type.as_ref().filter(|_| !game.goal_done(goal))
                        }
                        Goal::Defeat { .. }
                        | Goal::Escort { .. }
                        | Goal::Escape { .. }
                        | Goal::Control { .. }
                        | Goal::Event { .. }
                        | Goal::Hold { .. }
                        | Goal::Vigil { .. }
                        | Goal::Refuge { .. } => None,
                    };
                    if let Some(kind) = specialist {
                        let selector =
                            UnitSelector::Family(game.rules.units[game.kind(kind)].family);
                        if !recruitment
                            .requirements
                            .iter()
                            .any(|requirement| match requirement {
                                RecruitRequirement::EnsureUnits {
                                    selector: existing,
                                    count: _,
                                } => *existing == selector,
                            })
                        {
                            recruitment
                                .requirements
                                .push(RecruitRequirement::EnsureUnits { selector, count: 1 });
                        }
                    }
                }
            }
        } else if let Some(mission) = &game.mission {
            tactics.leader_post = mission.enemy_leader_post.clone().map(TileId);
            tactics.movement_tiles = game
                .territory()
                .into_iter()
                .map(|tile| TileId(game.board.ids[tile].clone()))
                .collect();
            if mission.defend_objectives {
                tactics.defensive_tiles = game
                    .territory()
                    .into_iter()
                    .map(|tile| TileId(game.board.ids[tile].clone()))
                    .collect();
            }
        }
        for unit in game.units.iter().filter(|u| u.side == game.turn) {
            let leader = game.rules.units[unit.kind].leader;
            let needs_protection = campaign
                && game.turn == Side::Player
                && unit.name.is_some()
                && game.essential(unit)
                && (!leader
                    || unit.hp * 2 <= game.rules.units[unit.kind].hp
                    || crate::ai::threats::Threats::protected(
                        &BattleView::new(game),
                        game.turn,
                        &tactics,
                    )
                    .at(unit, unit.tile, None)
                        >= unit.hp);
            if needs_protection {
                tactics.constraints.push(TacticalConstraint::Survive {
                    actor: UnitId(unit.id),
                });
            }
        }
        if let Some(unit) = game.units.iter().find(|u| u.side == game.turn) {
            tactics.terminal_targets = game
                .units
                .iter()
                .filter(|enemy| enemy.side != game.turn && terminal_attack(game, unit, enemy))
                .map(|enemy| UnitId(enemy.id))
                .collect();
        }
        if tactics.protect_withdrawal {
            tactics.mediated = game
                .units
                .iter()
                .filter(|u| u.side != game.turn && game.mediation_at_risk(u))
                .map(|u| UnitId(u.id))
                .collect();
        }
        for target in &authored_targets {
            if !actor_route(target) || target.id.starts_with("rebuild:") {
                continue;
            }
            let Some(actor) = game
                .units
                .iter()
                .filter(|u| {
                    u.side == game.turn
                        && BattleView::new(game).can_pursue(u, &Target::from(target.clone()))
                })
                .min_by_key(|u| {
                    target
                        .tiles
                        .iter()
                        .map(|tile| game.board.distance(u.tile, game.site(tile)))
                        .min()
                        .unwrap_or(u16::MAX)
                })
            else {
                continue;
            };
            let Some(destination) = target
                .tiles
                .iter()
                .min_by_key(|tile| game.board.distance(actor.tile, game.site(tile)))
            else {
                continue;
            };
            let actor_id = UnitId(actor.id);
            if campaign
                && game.turn == Side::Player
                && actor.name.is_some()
                && !tactics.protects(actor.id)
            {
                tactics
                    .constraints
                    .push(TacticalConstraint::Survive { actor: actor_id });
            }
            let priority = if game.turn == Side::Player {
                ObjectivePriority::Critical
            } else {
                ObjectivePriority::Normal
            };
            let occupy = target.hold == Some(true) || target.kind == ObjectiveKind::Vigil;
            tactics.objectives.push(ObjectiveOrder {
                id: target.id.clone(),
                priority,
                objective: if occupy {
                    TacticalObjective::Occupy {
                        actor: actor_id,
                        location: TileId(destination.clone()),
                    }
                } else {
                    TacticalObjective::GoTo {
                        actor: actor_id,
                        destination: TileId(destination.clone()),
                    }
                },
            });
            tactics.objectives.push(ObjectiveOrder {
                id: format!("{}:passage", target.id),
                priority,
                objective: TacticalObjective::ClearPassage {
                    actor: actor_id,
                    destination: TileId(destination.clone()),
                },
            });
            recruitment.reserved.push(TileId(destination.clone()));
            if game.turn == Side::Player
                && crate::ai::threats::Threats::unshielded(&BattleView::new(game), actor) > 0
            {
                tactics.objectives.push(ObjectiveOrder {
                    id: format!("{}:escort", target.id),
                    priority: ObjectivePriority::Important,
                    objective: TacticalObjective::Escort {
                        actor: actor_id,
                        guards: 2,
                        selector: UnitSelector::Role(UnitRole::Warrior),
                    },
                });
            }
            for destination in &target.tiles {
                if game.finishes_objective(actor, game.site(destination), &authored_targets) {
                    tactics
                        .terminal_arrivals
                        .push((actor_id, TileId(destination.clone())));
                }
            }
        }
        recruitment
            .reserved
            .extend(tactics.withdrawal_tiles.iter().cloned());
        let mut lanes: Vec<_> = BattleView::new(game)
            .actor_lanes(&tactics.targets)
            .into_values()
            .flatten()
            .map(|tile| TileId(game.board.ids[tile].clone()))
            .collect();
        lanes.sort_by(|a, b| a.0.cmp(&b.0));
        lanes.dedup();
        recruitment.reserved.extend(lanes);
        tactics.required_units = recruitment.requirements.clone();
        tactics
            .objectives
            .sort_by(|a, b| b.priority.cmp(&a.priority).then(a.id.cmp(&b.id)));
        CampaignOrders {
            recruitment,
            tactics,
        }
    }
}
impl Game<'_> {
    pub(in crate::ai) fn policy_objectives(&self, goals: Option<&[Objective]>) -> Vec<Objective> {
        let mut goals = goals.map_or_else(|| self.objectives(self.turn), |g| g.to_vec());
        goals.extend(self.rebuild_objectives(&goals));
        goals
    }
    pub(in crate::ai) fn rebuild_objectives(&self, goals: &[Objective]) -> Vec<Objective> {
        let mut rebuilding = vec![];
        let leaders: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.side == self.turn && self.rules.units[u.kind].leader)
            .map(|u| u.id)
            .collect();
        for id in leaders {
            let required = goals
                .iter()
                .any(|g| g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, id));
            let (targets, _) = self.leader_rebuild_plan(id, required);
            if !targets.is_empty() {
                let mut o = crate::ai::objectives::objective(
                    &format!("rebuild:u{id}"),
                    ObjectiveKind::Capture,
                    targets.iter().map(|&t| self.board.ids[t].clone()).collect(),
                    80.0,
                );
                o.actor = Some(format!("u{id}"));
                o.hold = Some(true);
                rebuilding.push(o);
            }
        }
        rebuilding
    }
    /// Mission tools are required independently of the combat recruitment style.
    pub(in crate::ai) fn required_recruit(&self, side: Side) -> Option<Family> {
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
                Goal::Hold { companion, .. } => (
                    ObjectiveKind::Capture,
                    companion.as_ref().and_then(|name| {
                        game.units
                            .iter()
                            .find(|u| u.side == side && u.name.as_ref() == Some(name))
                            .map(|u| format!("u{}", u.id))
                    }),
                    None,
                    true,
                    false,
                ),
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
    pub(in crate::ai) fn leader_rebuild_plan(
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
        let count = BattleView::new(self).ordinary(&u).count();
        let rebuilding = if self.required_recruit(u.side).is_some() {
            true
        } else if count >= 5 {
            false
        } else if count <= 2 {
            true
        } else {
            u.rebuilding.unwrap_or(true)
        };
        let targets = if !rebuilding {
            vec![]
        } else if self.income(u.side) > 0
            || options
                .iter()
                .any(|&k| self.rules.units[k].cost <= self.gold[u.side.index()])
        {
            keeps
        } else {
            // Recover village income before returning to an unaffordable keep.
            (0..self.terrain.len())
                .filter(|&tile| {
                    self.terrain_is(tile, Terrain::Village) && self.owners[tile] != Some(u.side)
                })
                .collect()
        };
        (targets, Some(rebuilding))
    }
}
pub(in crate::ai) fn policy_goals(
    game: &Game<'_>,
    provided: Option<&[Objective]>,
) -> Vec<Objective> {
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
pub(in crate::ai) fn heavy_campaign(game: &Game<'_>) -> bool {
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
pub(in crate::ai) fn restrained(game: &Game<'_>, workspace: &PlanningWorkspace) -> bool {
    workspace.diplomacy == crate::ai::Diplomacy::Restrained
        || !heavy_campaign(game)
            && game.turn == Side::Player
            && game.mission.as_ref().is_some_and(|m| m.refuge.is_some())
            && game.campaign.as_ref().is_some_and(|c| !c.escalated())
}
pub(in crate::ai) fn terminal_attack(game: &Game<'_>, unit: &Unit, target: &Unit) -> bool {
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

pub(super) struct Mission;
impl super::Campaigner for Mission {
    fn objectives(
        &mut self,
        view: crate::ai::context::CampaignView<'_, '_>,
    ) -> crate::ai::orders::CampaignOrders {
        view.orders(super::CampaignerKind::Mission)
    }
}

fn individual(goals: Vec<Objective>) -> Vec<Objective> {
    goals
        .into_iter()
        .flat_map(|g| {
            if g.kind == ObjectiveKind::Capture {
                g.tiles
                    .iter()
                    .map(|t| {
                        let mut o = g.clone();
                        o.id = format!("{}:{t}", g.id);
                        o.tiles = vec![t.clone()];
                        o
                    })
                    .collect()
            } else {
                vec![g]
            }
        })
        .collect()
}

fn actor_route(goal: &Objective) -> bool {
    fn required_arrival(goal: &Objective) -> bool {
        match goal.kind {
            ObjectiveKind::Escort | ObjectiveKind::Escape | ObjectiveKind::Vigil => true,
            ObjectiveKind::Capture => goal.actor.is_some(),
            ObjectiveKind::Seal => goal.family.is_some(),
            ObjectiveKind::Visit => goal.actor.is_some(),
            ObjectiveKind::Defeat => false,
        }
    }
    required_arrival(goal)
}
