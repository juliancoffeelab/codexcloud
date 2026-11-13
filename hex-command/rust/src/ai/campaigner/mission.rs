//! Authored missions are interpreted here and emitted as explicit turn orders.
use super::CampaignerKind as CampaignBlock;
use crate::ai::context::BattleView;
use crate::ai::orders::UnitSelector;
use crate::ai::{
    Diplomacy,
    orders::*,
    policies::objectives::{actor_matches, objective},
};
use crate::data::*;
use crate::sort::SharedSort;
use hex_command_battle::{
    CampaignGroup, CampaignIntent, CampaignObjective, CampaignView as AuthoredView, PhaseForecasts,
};
use rustc_hash::FxHashSet as HashSet;
#[derive(Clone, Copy)]
pub(crate) struct CampaignInput<'a> {
    pub seed: u32,
    pub goals: Option<&'a [Objective]>,
    pub diplomacy: Diplomacy,
}

/// Mission decision inputs. The authored read model cannot obtain live execution.
pub(crate) struct MissionPolicy<'a, 'catalog> {
    pub(crate) battle: &'a BattleView<'a, 'catalog>,
    authored: AuthoredView<'a, 'catalog>,
    phases: PhaseForecasts<'a, 'catalog>,
    objectives: Vec<CampaignObjective<'a>>,
    groups: Vec<CampaignGroup<'a>>,
    requirements_disclosed: bool,
}
impl<'a, 'catalog> MissionPolicy<'a, 'catalog> {
    pub(crate) fn new(
        authored: AuthoredView<'a, 'catalog>,
        phases: PhaseForecasts<'a, 'catalog>,
    ) -> Self {
        let battle = authored.battle();
        let objectives = authored.planning_objectives();
        let groups = authored.groups();
        let requirements_disclosed = authored.requirements_disclosed();
        Self {
            battle,
            authored,
            phases,
            objectives,
            groups,
            requirements_disclosed,
        }
    }
    /// Scheduling phase is not allegiance: an autonomous ally shares the
    /// player's campaign duties even when it acts during the AI phase.
    fn player_team(&self, phase: TurnPhase) -> bool {
        self.faction_relation("player", self.faction_for_phase(phase)) == Relation::Allied
    }
    fn player_unit(&self, unit: &Unit) -> bool {
        self.faction_relation("player", self.unit_faction(unit)) == Relation::Allied
    }
    fn script_unit_matches(&self, selector: &UnitMatch, unit: &Unit) -> bool {
        self.authored.script_unit_matches(selector, unit)
    }
    fn mediation_at_risk(&self, unit: &Unit) -> bool {
        self.authored.mediation_at_risk(unit)
    }
}
impl<'a, 'catalog> std::ops::Deref for MissionPolicy<'a, 'catalog> {
    type Target = BattleView<'a, 'catalog>;
    fn deref(&self) -> &Self::Target {
        self.battle
    }
}

impl CampaignBlock {
    pub const fn supports_campaign(self) -> bool {
        match self {
            Self::Mission => true,
            Self::Arena => false,
        }
    }
    pub(in crate::ai) fn restrained(
        self,
        game: &MissionPolicy<'_, '_>,
        input: CampaignInput<'_>,
    ) -> bool {
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
        let campaign = game.authored.in_campaign();
        let mut authored_targets = match self {
            Self::Mission => self.goals(game, input.goals),
            Self::Arena => individual(game.policy_objectives(input.goals)),
        };
        let mut scoped = Vec::new();
        let mut inactive: Vec<_> = game
            .units()
            .iter()
            .filter(|unit| game.belongs_to_phase(unit, game.turn()) && !game.may_act(unit))
            .map(|unit| UnitId(unit.id))
            .collect();
        if game.turn() == TurnPhase::Ai && !game.groups.is_empty() {
            let ordinary = std::mem::take(&mut authored_targets);
            for unit in game
                .units()
                .iter()
                .filter(|unit| game.belongs_to_phase(unit, game.turn()))
            {
                let group = game
                    .name(unit.ai_group)
                    .and_then(|id| game.groups.iter().find(|group| group.id == id));
                if let Some(group) = group {
                    if !group.active {
                        inactive.push(UnitId(unit.id));
                        continue;
                    }
                    if let Some((area, attack_radius)) = &group.hold {
                        scoped.push(ScopedOrder {
                            actor: UnitId(unit.id),
                            area: area.iter().cloned().map(TileId).collect(),
                            attack_radius: *attack_radius,
                        });
                        let mut target = objective(
                            &format!("group:{}:{}", group.id, unit.id),
                            ObjectiveKind::Visit,
                            area.to_vec(),
                            24.0,
                        );
                        target.actor = Some(format!("u{}", unit.id));
                        authored_targets.push(target);
                        continue;
                    }
                    let tiles = game
                        .units()
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
            deadline_round: game.authored.deadline_round(),
            progress: game
                .objectives
                .iter()
                .map(|o| ObjectiveProgress {
                    id: format!(
                        "{}:{}",
                        if o.optional { "optional" } else { "required" },
                        o.id
                    ),
                    current: o.progress_current,
                    total: o.progress_total,
                    satisfied: o.satisfied,
                    hold_turns_remaining: o.hold_turns_remaining,
                })
                .collect(),
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
        tactics.pending_arrivals = game.authored.pending_arrivals();
        for unit in game
            .units()
            .iter()
            .filter(|unit| game.belongs_to_phase(unit, game.turn()))
        {
            let mut sites = vec![];
            for objective in &game.objectives {
                if objective.optional {
                    continue;
                }
                let Some(selector) = objective.selector else {
                    continue;
                };
                let relevant = match objective.intent {
                    CampaignIntent::Escort => {
                        unit.role.is_some_and(Role::is_escort)
                            && game.script_unit_matches(selector, unit)
                    }
                    CampaignIntent::Escape => {
                        unit.is_leader() && game.script_unit_matches(selector, unit)
                    }
                    CampaignIntent::Hold => selector
                        .character
                        .as_ref()
                        .is_some_and(|id| game.name(unit.character) == Some(id.as_str())),
                    CampaignIntent::GoTo => game.script_unit_matches(selector, unit),
                    CampaignIntent::Approach
                    | CampaignIntent::Own
                    | CampaignIntent::Capture
                    | CampaignIntent::Defeat => false,
                };
                if relevant {
                    sites.extend(objective.tiles.iter().cloned().map(TileId));
                }
            }
            if !sites.is_empty() {
                tactics.transition_sensitive.push((UnitId(unit.id), sites));
            }
        }
        for unit in game
            .units()
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
        if game.player_team(game.turn()) {
            for objective in &game.objectives {
                if objective.intent != CampaignIntent::Hold || objective.satisfied {
                    continue;
                }
                let selector = objective.selector.expect("hold selector");
                let kinds: Vec<_> = selector
                    .unit_type
                    .iter()
                    .chain(selector.unit_types.iter())
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
        } else if game.authored.in_campaign() {
            tactics.leader_post = game
                .authored
                .enemy_leader_post()
                .map(|tile| TileId(tile.into()));
            tactics.movement_tiles = game
                .territory()
                .into_iter()
                .map(|tile| TileId(game.board().ids[tile].clone()))
                .collect();
            if game.authored.defend_objectives() {
                tactics.defensive_tiles = game
                    .territory()
                    .into_iter()
                    .map(|tile| TileId(game.board().ids[tile].clone()))
                    .collect();
            }
        }
        for unit in game
            .units()
            .iter()
            .filter(|u| game.belongs_to_phase(u, game.turn()))
        {
            let leader = unit.is_leader();
            let needs_protection = campaign
                && game.player_team(game.turn())
                && unit.character.is_some()
                && game.essential(unit)
                && (!leader
                    || unit.hp * 2 <= game.rules().units[unit.kind].hp
                    || crate::ai::policies::threat::Threats::protected(
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
        if let Some(unit) = game
            .units()
            .iter()
            .find(|u| game.belongs_to_phase(u, game.turn()))
        {
            tactics.terminal_targets = game
                .units()
                .iter()
                .filter(|enemy| {
                    game.hostile_to_phase(enemy, game.turn()) && terminal_attack(game, unit, enemy)
                })
                .map(|enemy| UnitId(enemy.id))
                .collect();
        }
        if tactics.protect_withdrawal {
            tactics.mediated = game
                .units()
                .iter()
                .filter(|u| game.hostile_to_phase(u, game.turn()) && game.mediation_at_risk(u))
                .map(|u| UnitId(u.id))
                .collect();
        }
        for target in &authored_targets {
            if !actor_route(target) || target.id.starts_with("rebuild:") {
                continue;
            }
            let Some(actor) = game
                .units()
                .iter()
                .filter(|u| {
                    game.belongs_to_phase(u, game.turn())
                        && crate::ai::policies::objectives::can_pursue(
                            &game.battle,
                            u,
                            &Target::from(target.clone()),
                        )
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
                && game.player_team(game.turn())
                && actor.character.is_some()
                && !tactics.protects(actor.id)
            {
                tactics
                    .constraints
                    .push(TacticalConstraint::Survive { actor: actor_id });
            }
            let priority = if game.player_team(game.turn()) {
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
            let travelling = game.player_team(game.turn())
                && actor.is_leader()
                && !occupy
                && !authored_targets.iter().any(|goal| {
                    goal.id.starts_with("required:") && goal.kind == ObjectiveKind::Defeat
                });
            if travelling {
                // Travel is a company duty, even before the leader is threatened.
                for role in UnitRole::ALL {
                    let guards = game
                        .units()
                        .iter()
                        .filter(|unit| {
                            game.belongs_to_phase(unit, game.turn())
                                && !unit.is_leader()
                                && !unit.role.is_some_and(Role::is_escort)
                                && game.rules().units[unit.kind].role == role
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
            } else if game.player_team(game.turn())
                && crate::ai::policies::threat::Threats::unshielded(game.phases, actor) > 0
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
        let mut lanes: Vec<_> =
            crate::ai::policies::objectives::actor_lanes(&game.battle, &tactics.targets)
                .into_values()
                .flatten()
                .map(|tile| TileId(game.board().ids[tile].clone()))
                .collect();
        lanes.order_by_key(|tile| game.board().positions[game.site(&tile.0)]);
        lanes.dedup();
        recruitment.reserved.extend(lanes);
        tactics.required_units = recruitment.requirements.clone();
        super::patrol::assign(&game.battle, input.seed, &mut tactics, |unit| {
            game.authored.search_guidance(unit)
        });
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
    pub(crate) fn scripted_policy_objectives(&self, side: TurnPhase) -> Vec<Objective> {
        let mut goals = vec![];
        for objective in &self.objectives {
            let kind = match objective.intent {
                CampaignIntent::Hold => ObjectiveKind::Vigil,
                CampaignIntent::Escort => ObjectiveKind::Escort,
                CampaignIntent::GoTo | CampaignIntent::Approach => ObjectiveKind::Visit,
                CampaignIntent::Escape => ObjectiveKind::Escape,
                CampaignIntent::Own | CampaignIntent::Capture => ObjectiveKind::Capture,
                CampaignIntent::Defeat => ObjectiveKind::Defeat,
            };
            let hold = match objective.intent {
                CampaignIntent::Hold | CampaignIntent::Own | CampaignIntent::Capture => true,
                CampaignIntent::Escort
                | CampaignIntent::GoTo
                | CampaignIntent::Escape
                | CampaignIntent::Approach
                | CampaignIntent::Defeat => false,
            };
            let selector = match objective.intent {
                CampaignIntent::Own | CampaignIntent::Capture | CampaignIntent::Defeat => None,
                CampaignIntent::Hold
                | CampaignIntent::Escort
                | CampaignIntent::GoTo
                | CampaignIntent::Escape
                | CampaignIntent::Approach => objective.selector,
            };
            let tiles = objective
                .tiles
                .iter()
                .filter(|t| {
                    objective.intent != CampaignIntent::Capture
                        || self.owners()[self.site(t)].as_deref() != Some("player")
                })
                .cloned()
                .collect::<Vec<_>>();
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
                    format!("optional:{}", objective.id)
                } else {
                    format!("required:{}", objective.id)
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
        // A valid strategic requirement may currently have no known location
        // (for example an unseen opposing leader). It is not a movement target.
        goals.retain(|goal| !goal.tiles.is_empty());
        goals.extend(self.rebuild_objectives(&goals));
        goals
    }
    pub(in crate::ai) fn rebuild_objectives(&self, goals: &[Objective]) -> Vec<Objective> {
        let mut rebuilding = vec![];
        let leaders: Vec<_> = self
            .units()
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
                    targets
                        .iter()
                        .map(|&t| self.board().ids[t].clone())
                        .collect(),
                    80.0,
                );
                o.actor = Some(format!("u{id}"));
                o.hold = Some(true);
                rebuilding.push(o);
            }
        }
        rebuilding
    }
    pub(in crate::ai) fn required_recruit(&self, side: TurnPhase) -> Option<Family> {
        if !self.player_team(side) {
            return None;
        }
        self.objectives
            .iter()
            .filter(|objective| !objective.satisfied && objective.intent == CampaignIntent::Hold)
            .find_map(|objective| {
                let selector = objective.selector.expect("hold selector");
                if self
                    .units()
                    .iter()
                    .any(|unit| unit.side == side && self.script_unit_matches(selector, unit))
                {
                    return None;
                }
                selector
                    .unit_type
                    .as_ref()
                    .or(selector.unit_types.first())
                    .map(|kind| self.rules().units[self.kind(kind)].family)
            })
    }

    pub fn essential(&self, u: &Unit) -> bool {
        u.is_leader()
            || u.role.is_some_and(Role::is_escort)
            || (self.player_unit(u)
                && self.name(u.character).is_some_and(|id| {
                    self.authored
                        .required_companions()
                        .iter()
                        .any(|required| required == id)
                }))
    }

    pub fn objectives(&self, side: TurnPhase) -> Vec<Objective> {
        let mut goals = if self.player_team(side) {
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
        if !self.authored.in_campaign() || !self.player_unit(unit) {
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
        arrival
            && self.requirements_disclosed
            && self.objectives.iter().all(|objective| {
                if objective.optional || objective.satisfied {
                    return true;
                }
                let Some(selector) = objective.selector else {
                    return false;
                };
                let actor = match objective.intent {
                    CampaignIntent::Escort => unit.role.is_some_and(Role::is_escort),
                    CampaignIntent::Escape => unit.is_leader(),
                    CampaignIntent::Hold
                    | CampaignIntent::GoTo
                    | CampaignIntent::Approach
                    | CampaignIntent::Own
                    | CampaignIntent::Capture
                    | CampaignIntent::Defeat => false,
                };
                actor
                    && self.script_unit_matches(selector, unit)
                    && objective.tiles.contains(&self.board().ids[tile])
            })
    }
    pub fn territory(&self) -> Vec<usize> {
        self.objectives
            .iter()
            .filter(|objective| match objective.intent {
                CampaignIntent::Capture | CampaignIntent::Own | CampaignIntent::Hold => true,
                CampaignIntent::Escort
                | CampaignIntent::GoTo
                | CampaignIntent::Escape
                | CampaignIntent::Approach
                | CampaignIntent::Defeat => false,
            })
            .flat_map(|objective| objective.tiles.iter())
            .map(|tile| self.site(tile))
            .collect()
    }

    pub fn protects_withdrawal(&self, side: TurnPhase, diplomatic: bool) -> bool {
        diplomatic && self.player_team(side) && self.authored.in_campaign()
    }
    pub fn withdrawal_lanes(&self, side: TurnPhase, restrained: bool) -> HashSet<usize> {
        let mut lanes = HashSet::default();
        if !self.protects_withdrawal(side, restrained) {
            return lanes;
        }
        for u in self
            .units()
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
        } else if self
            .units()
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
    let is_arrival = |objective: &CampaignObjective<'_>| match objective.intent {
        CampaignIntent::Escort | CampaignIntent::Escape => true,
        CampaignIntent::Hold
        | CampaignIntent::GoTo
        | CampaignIntent::Approach
        | CampaignIntent::Own
        | CampaignIntent::Capture
        | CampaignIntent::Defeat => false,
    };
    let escort_finish =
        provided.is_none()
            && game.player_team(game.turn())
            && game.requirements_disclosed
            && game.objectives.iter().any(|objective| {
                !objective.optional && !objective.satisfied && is_arrival(objective)
            })
            && game.objectives.iter().all(|objective| {
                objective.optional || objective.satisfied || is_arrival(objective)
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
pub(in crate::ai) fn terminal_attack(
    game: &MissionPolicy<'_, '_>,
    unit: &Unit,
    target: &Unit,
) -> bool {
    if !target.is_leader() {
        return false;
    }
    if !game.authored.in_campaign() {
        return true;
    }
    if !game.player_unit(unit) {
        return false;
    }
    game.requirements_disclosed
        && game.objectives.iter().all(|objective| {
            if objective.optional || objective.satisfied {
                return true;
            }
            objective.intent == CampaignIntent::Defeat
                && objective.selector.is_some_and(|selector| {
                    game.units()
                        .iter()
                        .filter(|unit| game.script_unit_matches(selector, unit))
                        .all(|unit| unit.id == target.id)
                })
        })
        && game
            .units()
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
