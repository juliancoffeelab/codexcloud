//! Diplomacy and ownership transitions, independent of planner implementations.
use super::Game;
use crate::data::*;
impl Game<'_> {
    #[inline]
    pub(crate) fn faction_for_phase(&self, side: Side) -> &str {
        self.active_faction
            .as_deref()
            .filter(|_| self.scoped_phase() == Some(side))
            .unwrap_or(side.faction_id())
    }
    #[inline]
    pub(crate) fn gold_for_phase(&self, side: Side) -> i32 {
        self.gold[self.faction_for_phase(side)]
    }
    #[inline]
    pub(crate) fn gold_for_unit(&self, unit: &Unit) -> i32 {
        self.gold[self.unit_faction(unit)]
    }
    #[inline]
    pub(crate) fn owns(&self, tile: usize, side: Side) -> bool {
        self.owners[tile].as_deref() == Some(self.faction_for_phase(side))
    }
    #[inline]
    pub(crate) fn captures(&self, tile: usize, side: Side) -> bool {
        self.owners[tile].as_deref().is_none_or(|owner| {
            self.faction_relation(self.faction_for_phase(side), owner) == Relation::Hostile
        })
    }
    pub(crate) fn control_roster(&self) -> Vec<(u32, Side, Option<String>, Option<Role>)> {
        self.units
            .iter()
            .map(|unit| (unit.id, unit.side, unit.faction.clone(), unit.role))
            .collect()
    }
    #[inline]
    pub(crate) fn unit_faction<'a>(&self, unit: &'a Unit) -> &'a str {
        unit.faction.as_deref().unwrap_or(unit.side.faction_id())
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
            controllable: faction.map_or(unit.side == Side::Player, |f| {
                f.controller == Controller::Human
            }),
        }
    }
    #[inline]
    pub(crate) fn faction(&self, id: &str) -> Option<&Faction> {
        self.factions.iter().find(|faction| faction.id == id)
    }
    #[inline]
    pub(crate) fn faction_relation(&self, first: &str, second: &str) -> Relation {
        crate::data::faction_relation(&self.factions, first, second)
    }
    #[inline]
    pub(crate) fn hostile(&self, first: &Unit, second: &Unit) -> bool {
        self.faction_relation(self.unit_faction(first), self.unit_faction(second))
            == Relation::Hostile
    }
    #[inline]
    pub(crate) fn allied(&self, first: &Unit, second: &Unit) -> bool {
        self.faction_relation(self.unit_faction(first), self.unit_faction(second))
            == Relation::Allied
    }
    #[inline]
    fn scoped_phase(&self) -> Option<Side> {
        self.active_faction.as_deref().map(|id| {
            self.faction(id)
                .map(|f| f.phase)
                .unwrap_or(if id == "player" {
                    Side::Player
                } else {
                    Side::Enemy
                })
        })
    }
    #[inline]
    pub(crate) fn hostile_to_phase(&self, unit: &Unit, phase: Side) -> bool {
        let faction = self
            .active_faction
            .as_deref()
            .filter(|_| self.scoped_phase() == Some(phase))
            .unwrap_or(phase.faction_id());
        self.faction_relation(faction, self.unit_faction(unit)) == Relation::Hostile
    }
    #[inline]
    pub(crate) fn belongs_to_phase(&self, unit: &Unit, phase: Side) -> bool {
        unit.side == phase
            && (self.scoped_phase() != Some(phase)
                || self
                    .active_faction
                    .as_deref()
                    .is_none_or(|faction| self.unit_faction(unit) == faction))
    }
    #[inline]
    pub(crate) fn may_act(&self, unit: &Unit) -> bool {
        self.belongs_to_phase(unit, self.turn)
            && !self
                .faction(self.unit_faction(unit))
                .is_some_and(|f| matches!(f.controller, Controller::Passive))
    }
    /// Fighting units available to the field army, excluding detached local duties.
    pub(crate) fn available_combatant(&self, unit: &Unit) -> bool {
        if self
            .faction(self.unit_faction(unit))
            .is_some_and(|f| matches!(f.controller, Controller::Passive))
            || self.weapons(unit).next().is_none()
        {
            return false;
        }
        let group = self.mission.as_ref().and_then(|mission| {
            unit.ai_group
                .as_ref()
                .and_then(|id| mission.enemy_groups.iter().find(|group| &group.id == id))
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
    pub(crate) fn automatic_factions(&self) -> Vec<(String, Option<crate::data::FactionPolicy>)> {
        let mut controllers = Vec::new();
        // The implicit legacy army remains an independent controller when authored
        // factions coexist with old, unassigned units or ordinary recruitment.
        if self
            .units
            .iter()
            .any(|u| u.side == self.turn && self.unit_faction(u) == self.turn.faction_id())
            && self.faction(self.turn.faction_id()).is_none()
        {
            controllers.push((self.turn.faction_id().to_owned(), None));
        }
        for faction in self.factions.iter().filter(|f| f.phase == self.turn) {
            match faction.controller {
                Controller::Ai { policy } => controllers.push((faction.id.clone(), policy)),
                Controller::Human => controllers.push((faction.id.clone(), None)),
                Controller::Passive => {}
            }
        }
        controllers
    }
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
                    Some(&self.board.ids[unit.tile]),
                )
                .then(|| {
                    let phase = self.faction(&rule.to).map(|f| f.phase).unwrap_or_else(|| {
                        if rule.to == "player" {
                            Side::Player
                        } else {
                            Side::Enemy
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
            let index = self.index(id).expect("transferred unit exists");
            let unit = &mut self.units[index];
            unit.faction = Some(faction.clone());
            unit.side = phase;
            unit.ai_group = None;
            unit.retreating = None;
            unit.rebuilding = None;
            let label = self.rules.units[unit.kind].name.clone();
            self.record(format!("{label} joined {faction}."));
        }
        self.roster_changed();
        true
    }
}
