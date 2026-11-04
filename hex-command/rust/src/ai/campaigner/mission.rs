//! Authored missions are interpreted here and emitted as explicit turn orders.
use super::CampaignerKind as CampaignBlock;
use crate::ai::context::BattleView;
use crate::ai::orders::UnitSelector;
use crate::ai::{
    Diplomacy,
    policies::objectives::{actor_matches, objective},
    orders::*,
};
use crate::sort::SharedSort;
use crate::data::*;
use hex_command_battle::{CampaignView as AuthoredView, PhaseForecasts};
use rustc_hash::FxHashSet as HashSet;
#[derive(Clone, Copy)]
pub(crate) struct CampaignInput<'a> {
    pub goals: Option<&'a [Objective]>,
    pub diplomacy: Diplomacy,
}

/// Mission decision inputs. The authored read model cannot obtain live execution.
pub(crate) struct MissionPolicy<'a, 'catalog> {
    pub(crate) battle: &'a BattleView<'a, 'catalog>,
    authored: AuthoredView<'a, 'catalog>,
    phases: PhaseForecasts<'a, 'catalog>,
}
impl<'a, 'catalog> MissionPolicy<'a, 'catalog> {
    pub(crate) fn new(authored: AuthoredView<'a, 'catalog>, phases: PhaseForecasts<'a, 'catalog>) -> Self {
        let battle = authored.battle();
        Self { battle, authored, phases }
    }
    fn script_unit_matches(&self, selector: &UnitMatch, unit: &Unit) -> bool { self.authored.script_unit_matches(selector, unit) }
    fn script_condition(&self, condition: &When) -> bool { self.authored.script_condition(condition) }
    fn event_condition_met(&self, condition: &Condition) -> bool { self.authored.event_condition_met(condition) }
    fn mediation_at_risk(&self, unit: &Unit) -> bool { self.authored.mediation_at_risk(unit) }
    fn group_active(&self, group: &EnemyGroup) -> bool { self.authored.group_active(group) }
    fn approach_tiles(&self, selection: &ProximitySelection) -> Vec<String> { self.authored.approach_tiles(selection) }
}
impl<'a, 'catalog> std::ops::Deref for MissionPolicy<'a, 'catalog> {
    type Target = BattleView<'a, 'catalog>;
    fn deref(&self) -> &Self::Target { self.battle }
}

fn is_spawn(action: &EventAction) -> bool {
    if let EventAction::Spawn(_) = action {
        true
    } else {
        false
    }
}
// Read event-owned duties through the same cheap completion facts as upstream.
fn authored_goal_done(game: &MissionPolicy<'_, '_>, event: &ScriptEvent) -> bool {
    let Some(objective) = &event.objective else {
        return true;
    };
    if objective.optional {
        return true;
    }
    let Some(state) = game.authored.scripts().as_deref() else {
        return false;
    };
    if !state.active.contains(&event.id) {
        return true;
    }
    match &objective.intent {
        EventIntent::Capture(tiles) => tiles
            .iter()
            .all(|tile| game.owners()[game.site(tile)].as_deref() == Some("player")),
        EventIntent::Escort(value) => game.units().iter().any(|unit| {
            unit.side == Side::Player
                && unit.role.is_some_and(Role::is_escort)
                && game.script_unit_matches(&value.units, unit)
                && game.board().ids[unit.tile] == value.tile
        }),
        EventIntent::Escape(value) => game.units().iter().any(|unit| {
            unit.side == Side::Player
                && unit.is_leader()
                && game.script_unit_matches(&value.units, unit)
                && game.board().ids[unit.tile] == value.tile
        }),
        _ => false,
    }
}
impl CampaignBlock {
    pub const fn supports_campaign(self) -> bool {
        match self {
            Self::Mission => true,
            Self::Arena => false,
        }
    }
    pub(in crate::ai) fn restrained(self, game: &MissionPolicy<'_, '_>, input: CampaignInput<'_>) -> bool {
        match self {
            Self::Arena => input.diplomacy == Diplomacy::Diplomatic,
            Self::Mission => restrained(game, input),
        }
    }
    pub(in crate::ai) fn goals(
        self,
        game: &MissionPolicy<'_, '_>,
        provided: Option<&[Objective]>,
    ) -> Vec<Objective> {
        match self {
            Self::Mission => policy_goals(game, provided),
            Self::Arena => individual(game.policy_objectives(provided)),
        }
    }

    pub(in crate::ai) fn campaign_orders(
        self,
        game: &MissionPolicy<'_, '_>,
        input: CampaignInput<'_>,
    ) -> CampaignOrders {
        let restrained = self.restrained(game, input);
        let campaign = game.authored.campaign().is_some();
        let mut authored_targets = match self {
            Self::Mission => self.goals(game, input.goals),
            Self::Arena => individual(game.policy_objectives(input.goals)),
        };
        let mut scoped = Vec::new();
        let mut inactive: Vec<_> = game.units()
            .iter()
            .filter(|unit| game.belongs_to_phase(unit, game.turn()) && !game.may_act(unit))
            .map(|unit| UnitId(unit.id))
            .collect();
        if game.turn() == Side::Enemy
            && game.authored.mission()
                .as_ref()
                .is_some_and(|m| !m.enemy_groups.is_empty())
        {
            let mission = game.authored.mission().as_ref().unwrap();
            let ordinary = std::mem::take(&mut authored_targets);
            for unit in game.units()
                .iter()
                .filter(|unit| game.belongs_to_phase(unit, game.turn()))
            {
                let group = unit
                    .ai_group
                    .as_ref()
                    .and_then(|id| mission.enemy_groups.iter().find(|group| group.id == *id));
                if let Some(group) = group {
                    if !game.group_active(group) {
                        inactive.push(UnitId(unit.id));
                        continue;
                    }
                    if let Some(order) = &group.order {
                        match order {
                            GroupOrder::Hold {
                                area,
                                attack_radius,
                                when,
                            } if when
                                .as_ref()
                                .is_none_or(|condition| game.event_condition_met(condition)) =>
                            {
                                scoped.push(ScopedOrder {
                                    actor: UnitId(unit.id),
                                    area: area.iter().cloned().map(TileId).collect(),
                                    attack_radius: *attack_radius,
                                });
                                let mut target = objective(
                                    &format!("group:{}:{}", group.id, unit.id),
                                    ObjectiveKind::Visit,
                                    area.clone(),
                                    24.0,
                                );
                                target.actor = Some(format!("u{}", unit.id));
                                authored_targets.push(target);
                                continue;
                            }
                            GroupOrder::Hold { .. } => {}
                        }
                    }
                    let tiles = game.units()
                        .iter()
                        .filter(|foe| game.hostile(unit, foe))
                        .map(|foe| game.board().ids[foe.tile].clone())
                        .collect();
                    let mut target = objective(
                        &format!("group:{}:{}", group.id, unit.id),
                        ObjectiveKind::Defeat,
                        tiles,
                        24.0,
                    );
                    target.actor = Some(format!("u{}", unit.id));
                    authored_targets.push(target);
                } else {
                    for target in &ordinary {
                        if target.actor.is_none() || actor_matches(&target.actor, unit.id) {
                            let mut target = target.clone();
                            target.id = format!("{}:actor:{}", target.id, unit.id);
                            target.actor = Some(format!("u{}", unit.id));
                            authored_targets.push(target);
                        }
                    }
                }
            }
        }
        let mut tactics = TacticalOrders {
            scoped,
            inactive,
            targets: authored_targets.iter().cloned().map(Target::from).collect(),
            restrained,
            campaign,
            protect_withdrawal: game.protects_withdrawal(game.turn(), restrained),
            withdrawal_tiles: game
                .withdrawal_lanes(game.turn(), restrained)
                .into_iter()
                .map(|tile| TileId(game.board().ids[tile].clone()))
                .collect(),
            ..TacticalOrders::default()
        };
        if let Some(mission) = &game.authored.mission() {
            // Preserve the upstream player-side, due-enemy-wave forecast horizon.
            // The event system supplies committed pending requests and scheduled waves.
            if game.turn() == Side::Player {
                if let Some(state) = game.authored.scripts().as_deref() {
                    tactics.pending_arrivals = state.pending.iter().any(|pending| {
                        game.faction(&pending.faction)
                            .is_some_and(|f| f.phase == Side::Enemy)
                    }) || mission.scripted_events.iter().any(|event| {
                        let due = match &event.when {
                            When::Turn(TurnSelection::Faction { number, faction }) => {
                                *number <= game.round()
                                    && game
                                        .faction(faction)
                                        .is_some_and(|f| f.phase == Side::Enemy)
                            }
                            _ => false,
                        };
                        due && state.active.contains(&event.id)
                            && event
                                .actions_recursive()
                                .iter()
                                .any(|action| is_spawn(action))
                    });
                }
            }
            // Read authored intent, never policy-generated rebuild/group goals.
            for unit in game.units()
                .iter()
                .filter(|u| game.belongs_to_phase(u, game.turn()))
            {
                let mut sites = vec![];
                for event in &mission.scripted_events {
                    let Some(objective) = &event.objective else {
                        continue;
                    };
                    if objective.optional {
                        continue;
                    }
                    let site = match &objective.intent {
                        EventIntent::Escort(value)
                            if unit.role.is_some_and(Role::is_escort)
                                && game.script_unit_matches(&value.units, unit) =>
                        {
                            Some(&value.tile)
                        }
                        EventIntent::Escape(value)
                            if unit.is_leader() && game.script_unit_matches(&value.units, unit) =>
                        {
                            Some(&value.tile)
                        }
                        EventIntent::Hold(value)
                            if value
                                .units
                                .character
                                .as_ref()
                                .is_some_and(|id| unit.character.as_ref() == Some(id)) =>
                        {
                            Some(&value.tile)
                        }
                        EventIntent::GoTo(value)
                            if game.script_unit_matches(&value.units, unit) =>
                        {
                            Some(&value.tile)
                        }
                        _ => None,
                    };
                    if let Some(site) = site {
                        sites.push(TileId(site.clone()));
                    }
                }
                if !sites.is_empty() {
                    tactics.transition_sensitive.push((UnitId(unit.id), sites));
                }
            }
        }
        for unit in game.units()
            .iter()
            .filter(|unit| game.belongs_to_phase(unit, game.turn()) && unit.is_leader())
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
        recruitment.rotate_pool = campaign || game.has_pools();
        if game.turn() == Side::Player {
            if let Some(mission) = &game.authored.mission() {
                for event in &mission.scripted_events {
                    let Some(objective) = &event.objective else {
                        continue;
                    };
                    let EventIntent::Hold(hold) = &objective.intent else {
                        continue;
                    };
                    if !game.authored.scripts()
                        .as_deref()
                        .is_some_and(|s| s.active.contains(&event.id))
                        || objective
                            .visible_when
                            .as_ref()
                            .is_some_and(|c| !game.script_condition(c))
                    {
                        continue;
                    }
                    let kinds: Vec<_> = hold
                        .units
                        .unit_type
                        .iter()
                        .chain(hold.units.unit_types.iter())
                        .cloned()
                        .map(UnitTypeId)
                        .collect();
                    if !kinds.is_empty() {
                        recruitment
                            .requirements
                            .push(RecruitRequirement::EnsureUnits {
                                selector: UnitSelector::AnyOf(kinds),
                                count: 1,
                            });
                    }
                }
            }
        } else if let Some(mission) = &game.authored.mission() {
            tactics.leader_post = mission.enemy_leader_post.clone().map(TileId);
            tactics.movement_tiles = game
                .territory()
                .into_iter()
                .map(|tile| TileId(game.board().ids[tile].clone()))
                .collect();
            if mission.defend_objectives {
                tactics.defensive_tiles = game
                    .territory()
                    .into_iter()
                    .map(|tile| TileId(game.board().ids[tile].clone()))
                    .collect();
            }
        }
        for unit in game.units()
            .iter()
            .filter(|u| game.belongs_to_phase(u, game.turn()))
        {
            let leader = unit.is_leader();
            let needs_protection = campaign
                && game.turn() == Side::Player
                && unit.character.is_some()
                && game.essential(unit)
                && (!leader
                    || unit.hp * 2 <= game.authored.catalogue().units[unit.kind].hp
                    || crate::ai::policies::threat::Threats::protected(
                        &game.battle,
                        game.phases,
                        game.turn(),
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
        if let Some(unit) = game.units()
            .iter()
            .find(|u| game.belongs_to_phase(u, game.turn()))
        {
            tactics.terminal_targets = game.units()
                .iter()
                .filter(|enemy| {
                    game.hostile_to_phase(enemy, game.turn()) && terminal_attack(game, unit, enemy)
                })
                .map(|enemy| UnitId(enemy.id))
                .collect();
        }
        if tactics.protect_withdrawal {
            tactics.mediated = game.units()
                .iter()
                .filter(|u| game.hostile_to_phase(u, game.turn()) && game.mediation_at_risk(u))
                .map(|u| UnitId(u.id))
                .collect();
        }
        for target in &authored_targets {
            if !actor_route(target) || target.id.starts_with("rebuild:") {
                continue;
            }
            let Some(actor) = game.units()
                .iter()
                .filter(|u| {
                    game.belongs_to_phase(u, game.turn())
                        && crate::ai::policies::objectives::can_pursue(&game.battle, u, &Target::from(target.clone()))
                })
                .min_by_key(|u| {
                    target
                        .tiles
                        .iter()
                        .map(|tile| game.board().distance(u.tile, game.site(tile)))
                        .min()
                        .unwrap_or(u16::MAX)
                })
            else {
                continue;
            };
            let Some(destination) = target
                .tiles
                .iter()
                .min_by_key(|tile| game.board().distance(actor.tile, game.site(tile)))
            else {
                continue;
            };
            let actor_id = UnitId(actor.id);
            if campaign
                && game.turn() == Side::Player
                && actor.character.is_some()
                && !tactics.protects(actor.id)
            {
                tactics
                    .constraints
                    .push(TacticalConstraint::Survive { actor: actor_id });
            }
            let priority = if game.turn() == Side::Player {
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
            let travelling = game.turn() == Side::Player
                && actor.is_leader()
                && !occupy
                && !authored_targets.iter().any(|goal| {
                    goal.id.starts_with("required:") && goal.kind == ObjectiveKind::Defeat
                });
            if travelling {
                // Travel is a company duty, even before the leader is threatened.
                for role in UnitRole::ALL {
                    let guards = game.units()
                        .iter()
                        .filter(|unit| {
                            game.belongs_to_phase(unit, game.turn())
                                && !unit.is_leader()
                                && !unit.role.is_some_and(Role::is_escort)
                                && game.authored.catalogue().units[unit.kind].role == role
                        })
                        .count();
                    if guards > 0 {
                        tactics.objectives.push(ObjectiveOrder {
                            id: format!("{}:{role:?}:escort", target.id),
                            priority: ObjectivePriority::Important,
                            objective: TacticalObjective::Escort {
                                actor: actor_id,
                                guards,
                                selector: UnitSelector::Role(role),
                            },
                        });
                    }
                }
            } else if game.turn() == Side::Player
                && crate::ai::policies::threat::Threats::unshielded(&game.battle, game.phases, actor) > 0
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
        let mut lanes: Vec<_> = crate::ai::policies::objectives::actor_lanes(&game.battle, &tactics.targets)
            .into_values()
            .flatten()
            .map(|tile| TileId(game.board().ids[tile].clone()))
            .collect();
        lanes.order_by_key(|tile| game.board().positions[game.site(&tile.0)]);
        lanes.dedup();
        recruitment.reserved.extend(lanes);
        tactics.required_units = recruitment.requirements.clone();
        tactics
            .objectives
            .order_by(|a, b| b.priority.cmp(&a.priority));
        CampaignOrders {
            recruitment,
            tactics,
        }
    }
}
impl MissionPolicy<'_, '_> {
    pub(crate) fn scripted_policy_objectives(&self, side: Side) -> Vec<Objective> {
        let Some(state) = self.authored.scripts().as_deref() else {
            return vec![];
        };
        let mut goals = vec![];
        for event in &self.authored.mission().as_ref().unwrap().scripted_events {
            let Some(objective) = &event.objective else {
                continue;
            };
            if !state.active.contains(&event.id)
                || objective
                    .visible_when
                    .as_ref()
                    .is_some_and(|c| !self.script_condition(c))
            {
                continue;
            }
            let (kind, tiles, selector, hold) = match &objective.intent {
                EventIntent::Hold(v) => (
                    ObjectiveKind::Vigil,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    true,
                ),
                EventIntent::Escort(v) => (
                    ObjectiveKind::Escort,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    false,
                ),
                EventIntent::GoTo(v) => (
                    ObjectiveKind::Visit,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    false,
                ),
                EventIntent::Escape(v) => (
                    ObjectiveKind::Escape,
                    vec![v.tile.clone()],
                    Some(&v.units),
                    false,
                ),
                EventIntent::Approach(v) => (
                    ObjectiveKind::Visit,
                    self.approach_tiles(v),
                    Some(&v.units),
                    false,
                ),
                EventIntent::Own(v) => (ObjectiveKind::Capture, vec![v.tile.clone()], None, true),
                EventIntent::Capture(v) => (
                    ObjectiveKind::Capture,
                    v.iter()
                        .filter(|t| self.owners()[self.site(t)].as_deref() != Some("player"))
                        .cloned()
                        .collect(),
                    None,
                    true,
                ),
                EventIntent::Defeat(v) => (
                    ObjectiveKind::Defeat,
                    self.units()
                        .iter()
                        .filter(|u| self.script_unit_matches(v, u))
                        .map(|u| self.board().ids[u.tile].clone())
                        .collect(),
                    None,
                    false,
                ),
            };
            if tiles.is_empty() {
                continue;
            }
            let actor = selector
                .filter(|s| {
                    s.character.is_some()
                        || s.role.is_some()
                        || s.unit_type.is_some()
                        || !s.unit_types.is_empty()
                })
                .and_then(|s| {
                    self.units()
                        .iter()
                        .filter(|u| {
                            self.belongs_to_phase(u, side) && self.script_unit_matches(s, u)
                        })
                        .min_by_key(|u| {
                            tiles
                                .iter()
                                .map(|t| self.board().distance(u.tile, self.site(t)))
                                .min()
                                .unwrap_or(0)
                        })
                });
            if selector.is_some_and(|s| {
                s.character.is_some()
                    || s.role.is_some()
                    || s.unit_type.is_some()
                    || !s.unit_types.is_empty()
            }) && actor.is_none()
            {
                continue;
            }
            goals.push(Objective {
                id: if objective.optional {
                    format!("optional:{}", event.objective_id())
                } else {
                    format!("required:{}", event.objective_id())
                },
                kind,
                tiles,
                weight: if objective.optional { 4.0 } else { 24.0 },
                actor: actor.map(|u| format!("u{}", u.id)),
                family: None,
                hold: Some(hold),
            });
        }
        goals
    }

    pub(in crate::ai) fn policy_objectives(&self, goals: Option<&[Objective]>) -> Vec<Objective> {
        let mut goals = goals.map_or_else(|| self.objectives(self.turn()), |g| g.to_vec());
        goals.extend(self.rebuild_objectives(&goals));
        goals
    }
    pub(in crate::ai) fn rebuild_objectives(&self, goals: &[Objective]) -> Vec<Objective> {
        let mut rebuilding = vec![];
        let leaders: Vec<_> = self.units()
            .iter()
            .filter(|u| self.belongs_to_phase(u, self.turn()) && u.is_leader())
            .map(|u| u.id)
            .collect();
        for id in leaders {
            let required = goals
                .iter()
                .any(|g| g.kind == ObjectiveKind::Escape && actor_matches(&g.actor, id));
            let (targets, _) = self.leader_rebuild_plan(id, required);
            if !targets.is_empty() {
                let mut o = crate::ai::policies::objectives::objective(
                    &format!("rebuild:u{id}"),
                    ObjectiveKind::Capture,
                    targets.iter().map(|&t| self.board().ids[t].clone()).collect(),
                    80.0,
                );
                o.actor = Some(format!("u{id}"));
                o.hold = Some(true);
                rebuilding.push(o);
            }
        }
        rebuilding
    }
    pub(in crate::ai) fn required_recruit(&self, side: Side) -> Option<Family> {
        if side != Side::Player {
            return None;
        }
        let mission = self.authored.mission().as_ref()?;
        let state = self.authored.scripts().as_deref()?;
        mission
            .scripted_events
            .iter()
            .filter(|e| state.active.contains(&e.id))
            .filter_map(|e| e.objective.as_ref())
            .filter(|o| {
                o.visible_when
                    .as_ref()
                    .is_none_or(|w| self.script_condition(w))
            })
            .find_map(|o| match &o.intent {
                EventIntent::Hold(v)
                    if !self.units()
                        .iter()
                        .any(|u| u.side == side && self.script_unit_matches(&v.units, u)) =>
                {
                    v.units
                        .unit_type
                        .as_ref()
                        .or(v.units.unit_types.first())
                        .map(|k| self.authored.catalogue().units[self.kind(k)].family)
                }
                _ => None,
            })
    }
    pub fn essential(&self, u: &Unit) -> bool {
        u.is_leader()
            || u.role.is_some_and(Role::is_escort)
            || (u.side == Side::Player
                && self.authored.mission().as_ref().is_some_and(|m| {
                    u.character
                        .as_ref()
                        .is_some_and(|n| m.required_companions.contains(n))
                }))
    }
    pub fn objectives(&self, side: Side) -> Vec<Objective> {
        let mut goals = if side == Side::Player {
            self.scripted_policy_objectives(side)
        } else {
            vec![]
        };
        for tile in 0..self.terrain().len() {
            if self.role_is(tile, TerrainRole::Village) && self.captures(tile, side) {
                goals.push(objective(
                    &format!("village:{}", self.board().ids[tile]),
                    ObjectiveKind::Capture,
                    vec![self.board().ids[tile].clone()],
                    8.0,
                ));
            }
        }
        if !goals.iter().any(|g| g.kind == ObjectiveKind::Defeat) {
            goals.push(objective(
                "opposing-leader",
                ObjectiveKind::Defeat,
                self.units()
                    .iter()
                    .filter(|u| self.hostile_to_phase(u, side) && u.is_leader())
                    .map(|u| self.board().ids[u.tile].clone())
                    .collect(),
                12.0,
            ));
        }
        goals.retain(|g| !g.tiles.is_empty());
        goals
    }
    pub fn finishes_objective(&self, unit: &Unit, tile: usize, goals: &[Objective]) -> bool {
        if self.authored.campaign().is_none() || unit.side != Side::Player {
            return false;
        }
        let arrival = goals.iter().any(|goal| {
            let arrival_kind = match goal.kind {
                ObjectiveKind::Escort | ObjectiveKind::Escape => true,
                _ => false,
            };
            actor_matches(&goal.actor, unit.id)
                && arrival_kind
                && goal.tiles.contains(&self.board().ids[tile])
        });
        let Some(mission) = self.authored.mission().as_ref() else {
            return false;
        };
        arrival
            && mission.scripted_events.iter().all(|event| {
                if authored_goal_done(self, event) {
                    return true;
                }
                let objective = event.objective.as_ref().unwrap();
                match &objective.intent {
                    EventIntent::Escort(value) => {
                        unit.role.is_some_and(Role::is_escort)
                            && self.script_unit_matches(&value.units, unit)
                            && value.tile == self.board().ids[tile]
                    }
                    EventIntent::Escape(value) => {
                        unit.is_leader()
                            && self.script_unit_matches(&value.units, unit)
                            && value.tile == self.board().ids[tile]
                    }
                    _ => false,
                }
            })
    }
    pub fn territory(&self) -> Vec<usize> {
        self.authored.mission().as_ref().map_or_else(Vec::new, |m| {
            m.scripted_events
                .iter()
                .filter_map(|e| e.objective.as_ref())
                .flat_map(|o| match &o.intent {
                    EventIntent::Capture(v) => v.clone(),
                    EventIntent::Own(v) => vec![v.tile.clone()],
                    EventIntent::Hold(v) => vec![v.tile.clone()],
                    _ => vec![],
                })
                .map(|t| self.site(&t))
                .collect()
        })
    }
    pub fn protects_withdrawal(&self, side: Side, diplomatic: bool) -> bool {
        diplomatic && side == Side::Player && self.authored.campaign().is_some()
    }
    pub fn withdrawal_lanes(&self, side: Side, restrained: bool) -> HashSet<usize> {
        let mut lanes = HashSet::default();
        if !self.protects_withdrawal(side, restrained) {
            return lanes;
        }
        for u in self.units()
            .iter()
            .filter(|u| self.hostile_to_phase(u, side) && u.retreating == Some(true))
        {
            let mut homes: Vec<_> = (0..self.terrain().len())
                .filter(|&t| {
                    self.owns(t, u.side)
                        && (self.is_keep(t) || self.role_is(t, TerrainRole::Village))
                })
                .collect();
            if homes.is_empty() {
                homes = (0..self.terrain().len())
                    .filter(|&t| self.board().positions[t].0 == self.cols() - 1)
                    .collect();
            }
            let routes = self.routes(u.kind, &homes);
            for &t in &self.board().adjacent[u.tile] {
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
        if !u.is_leader() || required {
            return (vec![], u.rebuilding);
        }
        let keeps: Vec<_> = (0..self.terrain().len())
            .filter(|&t| self.is_keep(t) && self.owns(t, u.side))
            .collect();
        let options = self.recruitment_options_for(u);
        if keeps.is_empty() || options.is_empty() {
            return (vec![], None);
        }
        let count = crate::ai::policies::objectives::ordinary(&self.battle, &u).count();
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
                .any(|&k| self.recruit_cost(u, k) <= self.gold_for_unit(u))
        {
            keeps
        } else if self.units()
            .iter()
            .filter(|unit| self.allied(u, unit) && !unit.is_leader())
            .count()
            < 2
        {
            // Only a nearly wiped-out army sends its leader to recover income.
            (0..self.terrain().len())
                .filter(|&tile| {
                    self.role_is(tile, TerrainRole::Village) && self.captures(tile, u.side)
                })
                .collect()
        } else {
            vec![]
        };
        (targets, Some(rebuilding))
    }
}
pub(in crate::ai) fn policy_goals(
    game: &MissionPolicy<'_, '_>,
    provided: Option<&[Objective]>,
) -> Vec<Objective> {
    let mut goals = game.policy_objectives(provided);
    let is_arrival = |event: &ScriptEvent| {
        event
            .objective
            .as_ref()
            .is_some_and(|objective| match &objective.intent {
                EventIntent::Escort(_) | EventIntent::Escape(_) => true,
                EventIntent::Hold(_)
                | EventIntent::Own(_)
                | EventIntent::GoTo(_)
                | EventIntent::Approach(_)
                | EventIntent::Capture(_)
                | EventIntent::Defeat(_) => false,
            })
    };
    let escort_finish = provided.is_none()
        && game.turn() == Side::Player
        && game.authored.mission().as_ref().is_some_and(|mission| {
            mission
                .scripted_events
                .iter()
                .any(|event| !authored_goal_done(game, event) && is_arrival(event))
                && mission
                    .scripted_events
                    .iter()
                    .all(|event| authored_goal_done(game, event) || is_arrival(event))
        });
    if escort_finish {
        for goal in goals.iter_mut().filter(|goal| goal.id == "opposing-leader") {
            goal.weight = 48.0;
        }
    }
    individual(goals)
}
pub(in crate::ai) fn restrained(_game: &MissionPolicy<'_, '_>, input: CampaignInput<'_>) -> bool {
    input.diplomacy == crate::ai::Diplomacy::Diplomatic
}
pub(in crate::ai) fn terminal_attack(game: &MissionPolicy<'_, '_>, unit: &Unit, target: &Unit) -> bool {
    if !target.is_leader() {
        return false;
    }
    if game.authored.campaign().is_none() {
        return true;
    }
    if unit.side != Side::Player {
        return false;
    }
    let Some(mission) = game.authored.mission().as_ref() else {
        return false;
    };
    mission.scripted_events.iter().all(|event| {
        if authored_goal_done(game, event) {
            return true;
        }
        if let EventIntent::Defeat(selector) = &event.objective.as_ref().unwrap().intent {
            game.units()
                .iter()
                .filter(|u| game.script_unit_matches(selector, u))
                .all(|u| u.id == target.id)
        } else {
            false
        }
    }) && game.units()
        .iter()
        .filter(|u| game.hostile(unit, u) && u.is_leader())
        .all(|u| u.id == target.id)
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
