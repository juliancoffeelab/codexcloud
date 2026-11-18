//! Event predicates can see explicit battle facts and explicit current progress only.
use super::{
    LiveBattle, battle::Board, conditions::LiveContext, event_state::EventProgress,
    events::DeathOccurrence,
};
use crate::data::*;
use crate::logic::event_conditions::{ConditionTree, ScriptTree};
pub(crate) struct EventFacts<'a, 'catalog> {
    game: &'a LiveBattle<'catalog>,
    observer: Option<&'a str>,
    rules: &'catalog Catalogue,
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
    pub(crate) fn new(game: &'a LiveBattle<'catalog>) -> Self {
        Self::for_context(game, None)
    }
    pub(crate) fn for_faction(game: &'a LiveBattle<'catalog>, observer: &'a str) -> Self {
        Self::for_context(game, Some(observer))
    }
    pub(crate) fn for_context(game: &'a LiveBattle<'catalog>, observer: Option<&'a str>) -> Self {
        Self {
            game,
            observer,
            rules: game.catalogue,
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
    pub(crate) fn for_event(game: &'a LiveBattle<'catalog>, event: &'a ScriptEvent) -> Self {
        Self::for_context(game, event.condition_observer())
    }
    pub(crate) fn round(&self) -> u32 {
        self.round
    }
    pub(crate) fn turn(&self) -> TurnPhase {
        self.turn
    }
    pub(crate) fn owned_by(&self, tile: &TileRef, faction: &str) -> bool {
        self.owners[self.site(tile)].as_deref() == Some(faction)
    }
    pub(crate) fn site(&self, id: &TileRef) -> usize {
        static_tile(self.board, id).expect("validated event location")
    }
    fn faction(&self, id: &str) -> Option<&Faction> {
        self.factions.iter().find(|f| f.id == id)
    }
    #[inline]
    pub(crate) fn script_unit_matches(&self, select: &UnitMatch, unit: &Unit) -> bool {
        unit_matches_for_faction(
            &self.game.definitions,
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
    pub(crate) fn stands(&self, select: &UnitMatch, tile: &TileRef, through: bool) -> bool {
        let target = self.site(tile);
        self.units.iter().any(|u| {
            self.script_unit_matches(select, u)
                && self.script_positions(u, through).contains(&target)
        })
    }
    /// One current-satisfaction rule for victory, inspection and permitted planning.
    pub(crate) fn objective_satisfied(&self, state: &EventProgress, id: &str) -> bool {
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
    pub(crate) fn when_matches(
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
                tick && self.round
                    == match turn {
                        TurnSelection::Number(number) | TurnSelection::Faction { number, .. } => {
                            *number
                        }
                    }
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
            When::Survives(v) => self.survives(v),
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

pub(crate) fn unit_matches_for_faction(
    definitions: &super::definition_index::DefinitionIndex<'_>,
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
            .is_none_or(|id| definitions.unit(unit.kind).id() == id)
        && (select.unit_types.is_empty()
            || select
                .unit_types
                .iter()
                .any(|id| id == definitions.unit(unit.kind).id()))
        && select
            .ai_group
            .as_ref()
            .is_none_or(|id| names.resolve(unit.ai_group) == Some(id.as_str()))
}

impl EventFacts<'_, '_> {
    pub(crate) fn condition_matches(&self, when: &When) -> bool {
        let game = self.game;
        let state = game.story.scripts.as_deref();
        assert!(
            state.is_some() || game.story.campaign.is_none(),
            "event evaluation requires explicit current progress"
        );
        self.when_matches(when, state, None, false, None)
    }
    pub(crate) fn approach_tiles<'unit>(
        &self,
        selection: &ProximitySelection,
        faction: &str,
        units: impl Iterator<Item = &'unit Unit>,
    ) -> Vec<Hex> {
        let game = self.game;
        let origins: Vec<_> = if let Some(t) = &selection.tile {
            vec![reference_site(&game.board, t)]
        } else {
            units
                .filter(|u| {
                    EventFacts::for_faction(game, faction)
                        .script_unit_matches(selection.target.as_ref().unwrap(), u)
                })
                .map(|u| u.tile)
                .collect()
        };
        (0..game.position.runtime.terrain.len())
            .filter(|&t| {
                origins
                    .iter()
                    .any(|&o| game.position.runtime.board.distance(t, o) <= selection.radius)
            })
            .map(|t| game.position.runtime.board.ids[t].clone())
            .collect()
    }
    pub(crate) fn objective_available(&self, event: &ScriptEvent) -> bool {
        let game = self.game;
        let state = game.story.scripts.as_deref().unwrap();
        let objective = event.objective.as_ref().unwrap();
        !state.failed.iter().any(|id| id == event.objective_id())
            && (objective.satisfied_when.is_some()
                || !state.completed.iter().any(|id| id == event.objective_id()))
            && objective.available_when.as_ref().is_none_or(|when| {
                EventFacts::for_faction(game, &objective.faction).when_matches(
                    when,
                    Some(state),
                    None,
                    false,
                    None,
                )
            })
    }
    pub(crate) fn objective_progress(&self, event: &ScriptEvent) -> Progress {
        let game = self.game;
        let objective = event.objective.as_ref().unwrap();
        let state = game.story.scripts.as_deref().unwrap();
        let done = EventFacts::for_faction(game, &objective.faction)
            .objective_satisfied(state, event.objective_id());
        let (tiles, total) = match &objective.intent {
            EventIntent::Hold(v) => (
                vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                v.turns,
            ),
            EventIntent::Escort(v) | EventIntent::GoTo(v) | EventIntent::Escape(v) => (
                vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                1,
            ),
            EventIntent::Approach(v) => (
                self.approach_tiles(v, &objective.faction, game.units.iter()),
                1,
            ),
            EventIntent::Own(v) => (
                vec![game.position.runtime.board.ids[reference_site(&game.board, &v.tile)]],
                v.turns.unwrap_or(1),
            ),
            EventIntent::Capture(v) => (
                v.iter()
                    .map(|t| game.position.runtime.board.ids[reference_site(&game.board, t)])
                    .collect(),
                v.len() as u32,
            ),
            EventIntent::Defeat(v) => (
                game.runtime
                    .units
                    .iter()
                    .filter(|u| {
                        EventFacts::for_faction(game, &objective.faction).script_unit_matches(v, u)
                    })
                    .map(|u| game.position.runtime.board.ids[u.tile].clone())
                    .collect(),
                1,
            ),
        };
        let current = if done {
            total
        } else if let EventIntent::Capture(tiles) = &objective.intent {
            tiles
                .iter()
                .filter(|t| {
                    game.position.runtime.owners[reference_site(&game.board, t)].as_deref()
                        == Some(objective.faction.as_str())
                })
                .count() as u32
        } else if matches!(
            &objective.intent,
            EventIntent::Hold(_) | EventIntent::Own(_)
        ) {
            state
                .holds
                .iter()
                .filter(|h| h.event == event.id)
                .map(|h| h.turns)
                .max()
                .unwrap_or(0)
                .min(total)
        } else {
            0
        };
        Progress {
            label: objective.label.clone(),
            current,
            total,
            done,
            tiles,
        }
    }
    fn survives(&self, selection: &SurvivalSelection) -> bool {
        let eligible = |faction: &str, character: Option<&str>, role: Option<Role>| {
            (faction == selection.company
                || selection.allies
                    && self.game.faction_relation(&selection.company, faction) == Relation::Allied)
                && selection
                    .character
                    .as_deref()
                    .is_none_or(|id| character == Some(id))
                && selection.role.is_none_or(|required| role == Some(required))
        };
        self.units.iter().any(|u| {
            eligible(
                self.game.unit_faction(u),
                self.game.name(u.character),
                u.role,
            )
        }) || selection.include_evacuated
            && self.game.story.campaign.as_ref().is_some_and(|state| {
                state.evacuated.iter().any(|record| {
                    let u = &record.unit;
                    eligible(&u.faction, u.character.as_deref(), u.role)
                })
            })
    }
    pub(crate) fn campaign_companion_survives(&self, company: &str, character: &str) -> bool {
        let game = self.game;
        let eligible = |faction: &str| game.faction_relation(company, faction) == Relation::Allied;
        game.units.iter().any(|unit| {
            game.name(unit.character) == Some(character) && eligible(game.unit_faction(unit))
        }) || game.story.campaign.as_ref().is_some_and(|state| {
            state.evacuated.iter().any(|record| {
                record.unit.character.as_deref() == Some(character)
                    && eligible(&record.unit.faction)
            })
        })
    }
    pub(crate) fn campaign_leader_survives(&self, company: &str) -> bool {
        let game = self.game;
        game.units
            .iter()
            .any(|unit| unit.is_leader() && game.unit_faction(unit) == company)
            || game.story.campaign.as_ref().is_some_and(|state| {
                state.evacuated.iter().any(|record| {
                    record.unit.role == Some(Role::Leader) && record.unit.faction == company
                })
            })
    }
    pub(crate) fn mediation_at_risk(&self, target: &Unit) -> bool {
        let game = self.game;
        game.story.mission.as_ref().is_some_and(|m| {
            m.scripted_events.iter().any(|e| {
                e.objective
                    .as_ref()
                    .and_then(|o| o.preserve_lives.as_ref())
                    .is_some_and(|s| {
                        game.story
                            .scripts
                            .as_deref()
                            .is_some_and(|state| state.active.contains(&e.id))
                            && EventFacts::for_faction(game, &e.objective.as_ref().unwrap().faction)
                                .script_unit_matches(s, target)
                    })
            })
        })
    }
}

pub(crate) fn reference_site(board: &Board, target: &TileRef) -> usize {
    static_tile(board, target).expect("validated authored target")
}

pub(crate) fn static_tile(board: &Board, target: &TileRef) -> Option<usize> {
    match target {
        TileRef::Hex(hex) => board.tile_at(*hex),
        TileRef::Location(name) => board.named_tile(name),
        TileRef::Binding(_) => panic!("tile binding must be resolved in its event context"),
    }
}
