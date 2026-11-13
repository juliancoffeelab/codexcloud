//! Diplomacy and ownership transitions, independent of planner implementations.
use super::BattlePosition;
use crate::data::*;
impl BattlePosition<'_> {
    #[inline]
    pub(crate) fn faction_for_phase(&self, phase: TurnPhase) -> &str {
        if let Some(id) = self
            .runtime
            .active_faction
            .as_deref()
            .filter(|_| self.scoped_phase() == Some(phase))
        {
            return id;
        }
        self.faction(phase.faction_id())
            .or_else(|| {
                self.factions
                    .iter()
                    .find(|f| f.phase == phase && f.controller != Controller::Passive)
            })
            .or_else(|| self.faction("player"))
            .expect("a battle has a primary player faction")
            .id
            .as_str()
    }
    #[inline]
    pub(crate) fn gold_for_phase(&self, side: TurnPhase) -> i32 {
        self.runtime.gold[self.faction_for_phase(side)]
    }
    #[inline]
    pub(crate) fn gold_for_unit(&self, unit: &Unit) -> i32 {
        self.runtime.gold[self.unit_faction(unit)]
    }
    #[inline]
    pub(crate) fn owns(&self, tile: usize, side: TurnPhase) -> bool {
        self.runtime.owners[tile].as_deref() == Some(self.faction_for_phase(side))
    }
    #[inline]
    pub(crate) fn captures(&self, tile: usize, side: TurnPhase) -> bool {
        self.runtime.owners[tile].as_deref().is_none_or(|owner| {
            self.faction_relation(self.faction_for_phase(side), owner) == Relation::Hostile
        })
    }
    pub(crate) fn control_roster(&self) -> Vec<(u32, TurnPhase, Option<NameKey>, Option<Role>)> {
        self.runtime
            .units
            .iter()
            .map(|unit| (unit.id, unit.side, Some(unit.faction), unit.role))
            .collect()
    }
    #[inline]
    pub(crate) fn unit_faction<'a>(&'a self, unit: &Unit) -> &'a str {
        self.runtime.names.get(unit.faction)
    }
    pub(crate) fn unit_faction_view(&self, unit: &Unit) -> UnitFaction {
        let id = self.unit_faction(unit);
        let faction = self.faction(id);
        UnitFaction {
            id: id.to_owned(),
            palette: faction
                .map(|f| f.palette.clone())
                .unwrap_or_else(|| unit.side.faction_id().to_owned()),
            relation: self.faction_relation("player", id),
            controllable: faction.map_or(unit.side == TurnPhase::Player, |f| {
                f.controller == Controller::Human
            }),
        }
    }
    #[inline]
    pub(crate) fn faction(&self, id: &str) -> Option<&Faction> {
        self.runtime
            .factions
            .iter()
            .find(|faction| faction.id == id)
    }
    #[inline]
    pub(crate) fn faction_relation(&self, first: &str, second: &str) -> Relation {
        crate::data::faction_relation(&self.runtime.factions, first, second)
    }
    #[inline]
    pub(crate) fn unit_relation(&self, first: &Unit, second: &Unit) -> Relation {
        self.faction_relation(self.unit_faction(first), self.unit_faction(second))
    }
    #[inline]
    pub(crate) fn hostile(&self, first: &Unit, second: &Unit) -> bool {
        self.unit_relation(first, second) == Relation::Hostile
    }
    #[inline]
    pub(crate) fn allied(&self, first: &Unit, second: &Unit) -> bool {
        self.unit_relation(first, second) == Relation::Allied
    }
    #[inline]
    fn scoped_phase(&self) -> Option<TurnPhase> {
        self.runtime.active_faction.as_deref().map(|id| {
            self.faction(id)
                .map(|f| f.phase)
                .unwrap_or(if id == "player" {
                    TurnPhase::Player
                } else {
                    TurnPhase::Ai
                })
        })
    }
    #[inline]
    pub(crate) fn hostile_to_phase(&self, unit: &Unit, phase: TurnPhase) -> bool {
        let faction = self
            .active_faction
            .as_deref()
            .filter(|_| self.scoped_phase() == Some(phase))
            .unwrap_or(phase.faction_id());
        self.faction_relation(faction, self.unit_faction(unit)) == Relation::Hostile
    }
    #[inline]
    pub(crate) fn belongs_to_phase(&self, unit: &Unit, phase: TurnPhase) -> bool {
        unit.side == phase
            && (self.scoped_phase() != Some(phase)
                || self
                    .active_faction
                    .as_deref()
                    .is_none_or(|faction| self.unit_faction(unit) == faction))
    }
    #[inline]
    pub(crate) fn may_act(&self, unit: &Unit) -> bool {
        self.belongs_to_phase(unit, self.runtime.turn)
            && !self
                .faction(self.unit_faction(unit))
                .is_some_and(|f| matches!(f.controller, Controller::Passive))
    }
    /// Fighting units available to the field army, excluding detached local duties.

    pub(crate) fn automatic_factions(&self) -> Vec<(String, Option<crate::data::FactionPolicy>)> {
        let mut controllers = Vec::new();
        // The implicit legacy army remains an independent controller when authored
        // factions coexist with old, unassigned units or ordinary recruitment.
        if self.units.iter().any(|u| {
            u.side == self.runtime.turn && self.unit_faction(u) == self.runtime.turn.faction_id()
        }) && self.faction(self.runtime.turn.faction_id()).is_none()
        {
            controllers.push((self.runtime.turn.faction_id().to_owned(), None));
        }
        for faction in self
            .runtime
            .factions
            .iter()
            .filter(|f| f.phase == self.runtime.turn)
        {
            match faction.controller {
                Controller::Ai { policy } => controllers.push((faction.id.clone(), policy)),
                Controller::Human => controllers.push((faction.id.clone(), None)),
                Controller::Passive => {}
            }
        }
        controllers
    }
}

impl super::Game<'_> {
    pub(crate) fn apply_faction_transfers(&mut self) -> bool {
        if self
            .factions
            .iter()
            .all(|faction| faction.transfer.is_none())
        {
            return false;
        }
        let input = self.condition_context();
        let changes: Vec<_> = self
            .units
            .iter()
            .filter_map(|unit| {
                let rule = self.faction(self.unit_faction(unit))?.transfer.as_ref()?;
                crate::logic::conditions::matches(
                    &rule.when.condition,
                    &input,
                    Some(&TileRef::Hex(self.runtime.board.ids[unit.tile])),
                )
                .then(|| {
                    let phase = self.faction(&rule.to).map(|f| f.phase).unwrap_or_else(|| {
                        if rule.to == "player" {
                            TurnPhase::Player
                        } else {
                            TurnPhase::Ai
                        }
                    });
                    (unit.id, rule.to.clone(), phase)
                })
            })
            .collect();
        if changes.is_empty() {
            return false;
        }
        for (id, faction, phase) in changes {
            if self
                .traversal
                .as_ref()
                .is_some_and(|(actor, _)| *actor == id)
            {
                self.runtime.traversal = None;
            }
            let index = self.index(id).expect("transferred unit exists");
            let key = self.intern_name(&faction);
            let vision_group = self.vision_group(&faction);
            let unit = &mut self.runtime.units[index];
            unit.faction = key;
            unit.vision_group = vision_group;
            unit.side = phase;
            unit.ai_group = None;
            unit.retreating = None;
            unit.rebuilding = None;
            let kind = unit.kind;
            let label = self.rules.units[kind].name.clone();
            self.record(format!("{label} joined {faction}."));
        }
        self.roster_changed();
        true
    }
    pub(crate) fn available_combatant(&self, unit: &Unit) -> bool {
        if self
            .faction(self.unit_faction(unit))
            .is_some_and(|f| matches!(f.controller, Controller::Passive))
            || self.weapons(unit).next().is_none()
        {
            return false;
        }
        let group = self.story.mission.as_ref().and_then(|mission| {
            self.name(unit.ai_group)
                .and_then(|id| mission.enemy_groups.iter().find(|group| group.id == id))
        });
        group.is_none_or(|group| {
            self.group_active(group)
                && match &group.order {
                    Some(GroupOrder::Hold { when, .. }) => when
                        .as_ref()
                        .is_some_and(|condition| !self.event_condition_met(condition)),
                    None => true,
                }
        })
    }
}

impl BattlePosition<'_> {
    #[inline]
    pub(crate) fn available_combatant(&self, unit: &Unit) -> bool {
        unit.field_available
    }
}
