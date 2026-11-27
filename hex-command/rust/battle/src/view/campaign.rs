//! Campaigner observations and their faction-specific authored objective projection.
use super::{
    BattleView, CampaignGroup, CampaignIntent, CampaignObjective, CampaignObservation,
    SearchGuidance,
};
use crate::data::*;
use crate::logic::event_conditions::{ConditionTree, ScriptTree};
use crate::logic::{
    conditions::LiveContext,
    event_queries::{EventFacts, reference_site},
};

impl<'a, 'catalog> std::ops::Deref for CampaignObservation<'a, 'catalog> {
    type Target = BattleView<'a, 'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.view
    }
}

impl<'a, 'catalog> CampaignObservation<'a, 'catalog> {
    /// A gameplay rule for strategic searching, not a sighting. Guide patrols
    /// loosely toward the nearest hostile unit in pathfinding terms without
    /// disclosing its roster or exact position.
    pub fn search_guidance(&self, origin: &Unit) -> Option<SearchGuidance> {
        let faction = self.game.unit_faction(origin);
        let mut goals: Vec<usize> = self
            .game
            .position
            .units
            .iter()
            .filter(|unit| {
                self.game
                    .faction_relation(faction, self.game.unit_faction(unit))
                    == Relation::Hostile
            })
            .map(|unit| unit.tile)
            .collect();
        if goals.is_empty() {
            return None;
        }
        goals.sort_unstable();
        goals.dedup();
        let routes = self.view.routes(origin, &goals);
        let origin_cost = routes[origin.tile];
        (origin_cost != u16::MAX && origin_cost > 0).then_some(SearchGuidance {
            routes,
            origin_cost,
        })
    }

    #[inline]
    pub fn in_campaign(&self) -> bool {
        self.game.story.campaign.is_some()
    }

    #[inline]
    pub fn required_companions(&self) -> &[String] {
        self.game
            .story
            .mission
            .as_ref()
            .map_or(&[], |m| m.required_companions.as_slice())
    }
    #[inline]
    pub fn enemy_leader_post(&self) -> Option<Hex> {
        if self
            .view
            .faction_relation("player", self.view.observer_faction())
            != Relation::Hostile
        {
            return None;
        }
        self.game
            .story
            .mission
            .as_ref()
            .and_then(|m| m.enemy_leader_post.as_ref())
            .map(|target| self.game.runtime.board.ids[reference_site(&self.game.board, target)])
    }
    #[inline]
    pub fn defend_objectives(&self) -> bool {
        if self
            .view
            .faction_relation("player", self.view.observer_faction())
            != Relation::Hostile
        {
            return false;
        }
        self.game
            .story
            .mission
            .as_ref()
            .is_some_and(|m| m.defend_objectives)
    }
    pub fn groups(self) -> Vec<CampaignGroup<'a>> {
        let game = self.game;
        if game.turn != TurnPhase::Ai {
            return vec![];
        }
        game.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.enemy_groups)
            .map(|group| {
                let hold = match &group.order {
                    Some(GroupOrder::Hold {
                        area,
                        attack_radius,
                        when,
                        observer,
                    }) if when.as_ref().is_none_or(|condition| {
                        LiveContext::new(game, observer.as_deref(), None)
                            .event_condition_met(condition)
                    }) =>
                    {
                        Some((
                            area.iter()
                                .map(|name| {
                                    game.board.ids[reference_site(&game.board, name)].clone()
                                })
                                .collect(),
                            *attack_radius,
                        ))
                    }
                    _ => None,
                };
                CampaignGroup {
                    id: &group.id,
                    active: LiveContext::new(game, None, None).group_active(group),
                    hold,
                }
            })
            .collect()
    }
    #[inline]
    pub fn script_unit_matches(&self, select: &UnitMatch, unit: &Unit) -> bool {
        EventFacts::for_faction(self.game, self.view.observer_faction())
            .script_unit_matches(select, unit)
    }

    #[inline]
    pub fn battle(&self) -> &'a BattleView<'a, 'catalog> {
        self.view
    }
    #[inline]
    pub fn mediation_at_risk(&self, target: &Unit) -> bool {
        EventFacts::new(self.game).mediation_at_risk(target)
    }

    /// Unconditional timed defeats are explicit mission limits. Conditional
    /// events remain unknown rather than guessing whether their guards will hold.
    pub fn deadline_round(&self) -> Option<u32> {
        let game = self.game;
        if game.story.campaign.is_none() {
            return None;
        }
        game.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .filter(|event| {
                event
                    .actions
                    .iter()
                    .any(|a| matches!(a, EventAction::Defeat(_)))
            })
            .filter_map(|event| match &event.when {
                When::TurnAtLeast(n) => Some(*n),
                When::Turn(TurnSelection::Number(n)) => Some(*n),
                _ => None,
            })
            .min()
    }
    pub fn planning_objectives(self) -> Vec<CampaignObjective<'a>> {
        let game = self.game;
        let faction = self.view.observer_faction();
        let Some(state) = game.story.scripts.as_deref() else {
            return vec![];
        };
        let facts = EventFacts::new(game);
        game.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .filter_map(|event| {
                let objective = event.objective.as_ref()?;
                if objective.faction != faction {
                    return None;
                }
                let id = event.objective_id();
                if !state.revealed.iter().any(|known| known == id)
                    || !EventFacts::new(game).objective_available(event)
                {
                    return None;
                }
                let satisfied = facts.objective_satisfied(state, id);
                let (intent, selector, tiles) = match &objective.intent {
                    EventIntent::Hold(v) => (
                        CampaignIntent::Hold,
                        Some(&v.units),
                        vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                    ),
                    EventIntent::Escort(v) => (
                        CampaignIntent::Escort,
                        Some(&v.units),
                        vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                    ),
                    EventIntent::GoTo(v) => (
                        CampaignIntent::GoTo,
                        Some(&v.units),
                        vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                    ),
                    EventIntent::Escape(v) => (
                        CampaignIntent::Escape,
                        Some(&v.units),
                        vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                    ),
                    EventIntent::Approach(v) => {
                        let tiles = EventFacts::new(game).approach_tiles(
                            v,
                            faction,
                            game.units
                                .iter()
                                .filter(|unit| game.unit_visible_to(faction, unit, None)),
                        );
                        // Unknown target locations do not remove a disclosed objective.
                        (CampaignIntent::Approach, Some(&v.units), tiles)
                    }
                    EventIntent::Own(v) => (
                        CampaignIntent::Own,
                        None,
                        vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                    ),
                    EventIntent::Capture(v) => (
                        CampaignIntent::Capture,
                        None,
                        v.iter()
                            .map(|t| {
                                game.position.runtime.board.ids[reference_site(&game.board, t)]
                            })
                            .collect(),
                    ),
                    EventIntent::Defeat(v) => (
                        CampaignIntent::Defeat,
                        Some(v),
                        game.runtime
                            .units
                            .iter()
                            .filter(|unit| {
                                EventFacts::for_faction(game, &objective.faction)
                                    .script_unit_matches(v, unit)
                            })
                            .map(|unit| game.position.runtime.board.ids[unit.tile].clone())
                            .collect(),
                    ),
                };
                let tiles = if intent == CampaignIntent::Defeat {
                    tiles
                        .into_iter()
                        .filter(|tile| self.view.at(self.view.site(tile)).is_some())
                        .collect()
                } else {
                    tiles
                };
                let progress = EventFacts::new(game).objective_progress(event);
                let hold_turns_remaining = match &objective.intent {
                    EventIntent::Hold(v) => Some(v.turns.saturating_sub(progress.current)),
                    EventIntent::Own(v) => v.turns.map(|n| n.saturating_sub(progress.current)),
                    _ => None,
                };
                Some(CampaignObjective {
                    id,
                    optional: objective.optional,
                    satisfied,
                    intent,
                    selector,
                    tiles,
                    progress_current: progress.current,
                    progress_total: progress.total,
                    hold_turns_remaining,
                })
            })
            .collect()
    }
    /// Aggregate readiness blocks terminal shortcuts without revealing hidden requirements.
    pub fn requirements_disclosed(&self) -> bool {
        let game = self.game;
        let faction = self.view.observer_faction();
        let Some(state) = game.story.scripts.as_deref() else {
            return true;
        };
        let facts = EventFacts::new(game);
        game.story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .all(|event| {
                event.objective.as_ref().is_none_or(|objective| {
                    objective.faction != faction
                        || objective.optional
                        || facts.objective_satisfied(state, event.objective_id())
                        || (state.revealed.iter().any(|id| id == event.objective_id())
                            && objective.available_when.as_ref().is_none_or(|when| {
                                EventFacts::for_faction(game, faction).when_matches(
                                    when,
                                    Some(state),
                                    None,
                                    false,
                                    None,
                                )
                            }))
                })
            })
    }
    pub fn pending_arrivals(&self) -> bool {
        let game = self.game;
        if game.position.runtime.turn != TurnPhase::Player {
            return false;
        }
        let Some(state) = game.story.scripts.as_deref() else {
            return false;
        };
        state.pending.iter().any(|pending| {
            game.faction(&pending.faction)
                .is_some_and(|f| f.phase == TurnPhase::Ai)
        }) || game
            .story
            .mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.scripted_events)
            .any(|event| {
                let due = match &event.when {
                    When::Turn(TurnSelection::Faction { number, faction }) => {
                        *number <= game.position.runtime.round
                            && game
                                .faction(faction)
                                .is_some_and(|f| f.phase == TurnPhase::Ai)
                    }
                    _ => false,
                };
                due && state.active.iter().any(|id| id == &event.id)
                    && event
                        .actions_recursive()
                        .iter()
                        .any(|a| matches!(a, EventAction::Spawn(_)))
            })
    }
}
