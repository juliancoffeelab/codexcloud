//! Event predicates can see explicit battle facts and explicit current progress only.
use super::{
    Game, battle::Board, conditions::LiveContext, event_state::EventProgress,
    events::DeathOccurrence,
};
use crate::data::*;
pub(super) struct EventFacts<'a, 'catalog> {
    game: &'a Game<'catalog>,
    observer: Option<&'a str>,
    rules: &'catalog Rules,
    board: &'a Board,
    units: &'a [Unit],
    names: &'a Names,
    factions: &'a [Faction],
    owners: &'a [Option<String>],
    round: u32,
    turn: TurnPhase,
    flags: Option<&'a FlagHistory>,
    traversal: Option<&'a (u32, Vec<usize>)>,
    event_departure: Option<&'a Unit>,
    event_opening: bool,
    visibility: LiveContext<'a, 'catalog>,
}
impl<'a, 'catalog> EventFacts<'a, 'catalog> {
    #[inline]
    pub(super) fn new(game: &'a Game<'catalog>) -> Self {
        Self::for_context(game, None)
    }
    pub(super) fn for_faction(game: &'a Game<'catalog>, observer: &'a str) -> Self {
        Self::for_context(game, Some(observer))
    }
    pub(super) fn for_context(game: &'a Game<'catalog>, observer: Option<&'a str>) -> Self {
        Self {
            game,
            observer,
            rules: game.rules,
            board: &game.board,
            units: &game.units,
            names: &game.runtime.names,
            factions: &game.factions,
            owners: &game.owners,
            round: game.round,
            turn: game.turn,
            flags: game.story.campaign.as_deref().map(|s| &s.flags),
            traversal: game.traversal.as_ref(),
            event_departure: game.story.event_departure.as_ref(),
            event_opening: game.story.event_opening,
            visibility: LiveContext::new(game, observer, None),
        }
    }
    pub(super) fn for_event(game: &'a Game<'catalog>, event: &'a ScriptEvent) -> Self {
        Self::for_context(game, event.condition_observer())
    }
    pub(super) fn round(&self) -> u32 {
        self.round
    }
    pub(super) fn turn(&self) -> TurnPhase {
        self.turn
    }
    pub(super) fn owned_by(&self, tile: &TileRef, faction: &str) -> bool {
        self.owners[self.site(tile)].as_deref() == Some(faction)
    }
    fn site(&self, id: &TileRef) -> usize {
        self.board.resolve(id).expect("validated event location")
    }
    fn faction(&self, id: &str) -> Option<&Faction> {
        self.factions.iter().find(|f| f.id == id)
    }
    #[inline]
    fn script_unit_matches(&self, select: &UnitMatch, unit: &Unit) -> bool {
        unit_matches_for_faction(
            self.rules,
            self.factions,
            self.names,
            select,
            unit,
            self.observer,
        )
    }
    fn pending_unit_matches(
        &self,
        s: &UnitMatch,
        group: &PendingEventSpawn,
        u: &SpawnMember,
    ) -> bool {
        s.retreating.is_none_or(|v| !v)
            && s.character
                .as_ref()
                .is_none_or(|id| u.character.as_ref() == Some(id))
            && s.role.is_none_or(|r| u.role == Some(r))
            && s.faction.as_ref().is_none_or(|id| id == &group.faction)
            && s.unit_type.as_ref().is_none_or(|id| id == &u.unit_type)
            && (s.unit_types.is_empty() || s.unit_types.contains(&u.unit_type))
            && s.ai_group
                .as_ref()
                .is_none_or(|id| group.ai_group.as_ref() == Some(id))
            && s.side.is_none_or(|side| {
                if side == ConditionSide::Observer {
                    group.faction == self.observer.expect("validated observer selector")
                } else {
                    faction_matches_side(self.factions, &group.faction, side)
                }
            })
    }
    fn script_positions<'u>(&'u self, unit: &'u Unit, through: bool) -> &'u [usize] {
        if through {
            if let Some((actor, path)) = &self.traversal {
                if *actor == unit.id {
                    return path;
                }
            }
        }
        std::slice::from_ref(&unit.tile)
    }
    pub(super) fn stands(&self, select: &UnitMatch, tile: &TileRef, through: bool) -> bool {
        let target = self.site(tile);
        self.units.iter().any(|u| {
            self.script_unit_matches(select, u)
                && self.script_positions(u, through).contains(&target)
        })
    }
    /// One current-satisfaction rule for victory, inspection and permitted planning.
    pub(super) fn objective_satisfied(&self, state: &EventProgress, id: &str) -> bool {
        if state.failed.iter().any(|value| value == id) {
            return false;
        }
        let objective = state
            .objective(id)
            .expect("validated objective reference")
            .objective
            .as_ref()
            .unwrap();
        objective.satisfied_when.as_ref().map_or_else(
            || state.completed.iter().any(|value| value == id),
            |when| {
                EventFacts::for_faction(self.game, &objective.faction).when_matches(
                    when,
                    Some(state),
                    None,
                    false,
                    None,
                )
            },
        )
    }
    pub(super) fn when_matches(
        &self,
        when: &When,
        state: Option<&EventProgress>,
        event: Option<usize>,
        tick: bool,
        death: Option<&DeathOccurrence>,
    ) -> bool {
        match when {
            When::All(values) => values
                .iter()
                .all(|value| self.when_matches(value, state, event, tick, death)),
            When::Any(values) => values
                .iter()
                .any(|value| self.when_matches(value, state, event, tick, death)),
            When::Not(c) => !self.when_matches(c, state, event, tick, death),
            When::Stand(v) => self.stands(&v.units, &v.tile, v.through),
            When::Proximity(v) => {
                let targets: Vec<_> = if let Some(tile) = &v.tile {
                    vec![self.site(tile)]
                } else {
                    self.units
                        .iter()
                        .filter(|u| self.script_unit_matches(v.target.as_ref().unwrap(), u))
                        .map(|u| u.tile)
                        .collect()
                };
                self.units
                    .iter()
                    .filter(|u| self.script_unit_matches(&v.units, u))
                    .any(|u| {
                        self.script_positions(u, v.through).iter().any(|&p| {
                            targets
                                .iter()
                                .any(|&t| self.board.distance(p, t) <= v.radius)
                        })
                    })
            }
            When::Killed(v) => death.is_some_and(|d| {
                self.script_unit_matches(&v.units, &d.unit)
                    && v.initiated_by.as_ref().is_none_or(|f| f == &d.initiated_by)
                    && v.killed_by
                        .as_ref()
                        .is_none_or(|f| Some(f) == d.killed_by.as_ref())
                    && v.defending.is_none_or(|value| value == d.defending)
            }),
            When::Left(v) => self.event_departure.as_ref().is_some_and(|u| {
                u.tile == self.site(&v.tile) && self.script_unit_matches(&v.units, u)
            }),
            When::Turn(turn) => {
                tick && self.round == turn.number()
                    && match turn {
                        TurnSelection::Number(_) => self.turn == TurnPhase::Player,
                        TurnSelection::Faction { faction, .. } => {
                            self.faction(faction).is_some_and(|f| f.phase == self.turn)
                        }
                    }
            }
            When::TurnAtLeast(turn) => self.round >= *turn,
            When::ChapterStarted(value) => self.event_opening == *value,
            When::Always(value) => *value,
            When::ObjectiveSatisfied(id) => {
                state.is_some_and(|state| self.objective_satisfied(state, id))
            }
            When::Flag(id) => self.flags.is_some_and(|f| flag_is_set(f, id)),
            When::HasUnits(v) => self.units.iter().any(|u| self.script_unit_matches(v, u)),
            When::NoUnits(v) => {
                !self.units.iter().any(|u| self.script_unit_matches(v, u))
                    && !state
                        .into_iter()
                        .flat_map(|state| &state.pending)
                        .any(|g| g.units.iter().any(|u| self.pending_unit_matches(v, g, u)))
            }
            When::VisibleUnits(v) => self
                .units
                .iter()
                .any(|u| self.script_unit_matches(v, u) && self.visibility.unit_visible(u)),
            When::Owner(v) => self.owners[self.site(&v.tile)].as_deref() == Some(&v.faction),
            When::Own(v) if v.turns.is_none() => {
                self.owners[self.site(&v.tile)].as_deref() == Some(&v.faction)
                    && v.when
                        .as_ref()
                        .is_none_or(|w| self.when_matches(w, state, event, false, None))
            }
            When::Own(_) | When::Held(_) => {
                let index = match when {
                    When::Held(value) => value.predicate,
                    When::Own(value) => value.predicate,
                    _ => unreachable!(),
                };
                state
                    .and_then(|s| event.and_then(|event| s.clock(event, index)))
                    .is_some_and(|h| {
                        h.turns
                            >= match when {
                                When::Held(v) => v.turns,
                                When::Own(v) => v.turns.unwrap(),
                                _ => unreachable!(),
                            }
                    })
            }
        }
    }
}

pub(super) fn unit_matches_for_faction(
    rules: &Rules,
    factions: &[Faction],
    names: &Names,
    select: &UnitMatch,
    unit: &Unit,
    observer: Option<&str>,
) -> bool {
    select
        .character
        .as_ref()
        .is_none_or(|id| names.resolve(unit.character) == Some(id.as_str()))
        && select
            .retreating
            .is_none_or(|value| unit.retreating.unwrap_or(false) == value)
        && select.role.is_none_or(|r| unit.role == Some(r))
        && select
            .faction
            .as_ref()
            .is_none_or(|id| names.get(unit.faction) == id)
        && select.side.is_none_or(|side| {
            if side == ConditionSide::Observer {
                names.get(unit.faction) == observer.expect("validated observer selector")
            } else {
                faction_matches_side(factions, names.get(unit.faction), side)
            }
        })
        && select
            .unit_type
            .as_ref()
            .is_none_or(|id| &rules.units[unit.kind].id == id)
        && (select.unit_types.is_empty() || select.unit_types.contains(&rules.units[unit.kind].id))
        && select
            .ai_group
            .as_ref()
            .is_none_or(|id| names.resolve(unit.ai_group) == Some(id.as_str()))
}
