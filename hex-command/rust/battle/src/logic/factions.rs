//! Diplomacy and ownership transitions, independent of planner implementations.
use super::BattlePosition;
use crate::data::*;
impl BattlePosition<'_> {
    #[inline]
    pub(crate) fn gold_for_faction(&self, faction: &str) -> i32 {
        self.runtime.gold[faction]
    }
    #[inline]
    pub(crate) fn gold_for_unit(&self, unit: &Unit) -> i32 {
        self.runtime.gold[self.unit_faction(unit)]
    }
    #[inline]
    pub(crate) fn owns(&self, tile: usize, faction: &str) -> bool {
        self.runtime.owners[tile].as_deref() == Some(faction)
    }
    #[inline]
    pub(crate) fn captures(&self, tile: usize, faction: &str) -> bool {
        self.runtime.owners[tile]
            .as_deref()
            .is_none_or(|owner| self.faction_relation(faction, owner) == Relation::Hostile)
    }
    pub(crate) fn control_roster(&self) -> Vec<(u32, TurnPhase, Option<NameKey>, Option<Role>)> {
        self.runtime
            .units
            .iter()
            .map(|unit| (unit.id, unit.phase, Some(unit.faction), unit.role))
            .collect()
    }
    #[inline]
    pub(crate) fn unit_faction<'a>(&'a self, unit: &Unit) -> &'a str {
        self.runtime.names.get(unit.faction)
    }
    pub(crate) fn unit_faction_view(&self, unit: &Unit) -> UnitFaction {
        let id = self.unit_faction(unit);
        let faction = self.faction(id).expect("validated unit faction");
        UnitFaction {
            id: id.to_owned(),
            palette: faction.palette.clone(),
            relation: self.faction_relation("player", id),
            controllable: faction.controller == Controller::Human,
            phase: faction.phase,
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
    pub(crate) fn hostile_to_faction(&self, unit: &Unit, faction: &str) -> bool {
        self.faction_relation(faction, self.unit_faction(unit)) == Relation::Hostile
    }
    #[inline]
    pub(crate) fn belongs_to_faction(&self, unit: &Unit, faction: &str) -> bool {
        self.unit_faction(unit) == faction
    }
    #[inline]
    pub(crate) fn belongs_to_phase(&self, unit: &Unit, phase: TurnPhase) -> bool {
        unit.phase == phase
    }
    #[inline]
    pub(crate) fn may_act(&self, unit: &Unit) -> bool {
        self.belongs_to_phase(unit, self.runtime.turn)
            && self
                .active_faction
                .as_deref()
                .is_none_or(|id| self.unit_faction(unit) == id)
            && !self
                .faction(self.unit_faction(unit))
                .is_some_and(|f| matches!(f.controller, Controller::Passive))
    }
    /// Fighting units available to the field army, excluding detached local duties.

    pub(crate) fn automatic_factions(&self) -> Vec<(String, Option<crate::data::FactionPolicy>)> {
        let mut controllers = Vec::new();
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
                crate::data::presentation::rule_matches(
                    &rule.when,
                    &input,
                    Some(&TileRef::Hex(self.runtime.board.ids[unit.tile])),
                )
                .then(|| {
                    let phase = self
                        .faction(&rule.to)
                        .expect("validated transfer faction")
                        .phase;
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
            unit.phase = phase;
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
                    Some(GroupOrder::Hold { when, observer, .. }) => {
                        when.as_ref().is_some_and(|condition| {
                            !self.event_condition_met(condition, observer.as_deref())
                        })
                    }
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
