//! Event-engine traversal and compilation helpers for authored condition trees.
use crate::data::*;
pub(crate) trait ConditionTree {
    fn needs_observer(&self) -> bool;
    fn state_only(&self) -> bool;
    fn visit_objective_refs(&self, visit: &mut impl FnMut(&str));
    fn has_death(&self) -> bool;
    fn index_holds(&mut self);
    fn holds(&self) -> Vec<EventClock<'_>>;
    fn uses_path(&self) -> bool;
}
pub(crate) trait ScriptTree {
    fn condition_observer(&self) -> Option<&str>;
    fn objective_id(&self) -> &str;
    fn actions_recursive(&self) -> Vec<&EventAction>;
}
impl ConditionTree for When {
    fn needs_observer(&self) -> bool {
        let unit = |v: &UnitMatch| v.side == Some(ConditionSide::Observer);
        match self {
            Self::VisibleUnits(_) => true,
            Self::All(v) | Self::Any(v) => v.iter().any(Self::needs_observer),
            Self::Not(v) => v.needs_observer(),
            Self::Stand(v) | Self::Left(v) => unit(&v.units),
            Self::Proximity(v) => unit(&v.units) || v.target.as_ref().is_some_and(unit),
            Self::Killed(v) => unit(&v.units),
            Self::HasUnits(v) | Self::NoUnits(v) => unit(v),
            Self::Held(v) => unit(&v.units) || v.when.as_ref().is_some_and(|w| w.needs_observer()),
            Self::Own(v) => v.when.as_ref().is_some_and(|w| w.needs_observer()),
            Self::Survives(_)
            | Self::Always(_)
            | Self::Turn(_)
            | Self::TurnAtLeast(_)
            | Self::ChapterStarted(_)
            | Self::Flag(_)
            | Self::ObjectiveSatisfied(_)
            | Self::Owner(_) => false,
        }
    }
    fn state_only(&self) -> bool {
        match self {
            Self::All(v) | Self::Any(v) => v.iter().all(Self::state_only),
            Self::Not(v) => v.state_only(),
            Self::Stand(v) => !v.through,
            Self::Proximity(v) => !v.through,
            Self::Own(v) => v.turns.is_none() && v.when.as_ref().is_none_or(|w| w.state_only()),
            Self::Held(_)
            | Self::Killed(_)
            | Self::Left(_)
            | Self::Turn(_)
            | Self::ChapterStarted(_) => false,
            _ => true,
        }
    }

    fn visit_objective_refs(&self, visit: &mut impl FnMut(&str)) {
        match self {
            Self::ObjectiveSatisfied(id) => visit(id),
            Self::All(values) | Self::Any(values) => {
                for value in values {
                    value.visit_objective_refs(visit);
                }
            }
            Self::Not(value) => value.visit_objective_refs(visit),
            Self::Own(value) => {
                if let Some(when) = &value.when {
                    when.visit_objective_refs(visit);
                }
            }
            Self::Held(value) => {
                if let Some(when) = &value.when {
                    when.visit_objective_refs(visit);
                }
            }
            _ => {}
        }
    }

    fn has_death(&self) -> bool {
        match self {
            Self::Killed(_) => true,
            Self::All(v) | Self::Any(v) => v.iter().any(Self::has_death),
            Self::Not(v) => v.has_death(),
            _ => false,
        }
    }
    // Match the durable counter numbering without depending on evaluation order.
    fn index_holds(&mut self) {
        fn visit(when: &mut When, next: &mut u32) {
            match when {
                When::Held(value) => {
                    value.predicate = *next;
                    *next += 1;
                }
                When::Own(value) if value.turns.is_some() => {
                    value.predicate = *next;
                    *next += 1;
                }
                When::All(values) | When::Any(values) => {
                    for value in values {
                        visit(value, next);
                    }
                }
                When::Not(value) => visit(value, next),
                _ => {}
            }
        }
        visit(self, &mut 0);
    }
    fn holds(&self) -> Vec<EventClock<'_>> {
        match self {
            Self::Held(v) => vec![EventClock {
                predicate: v.predicate,
                tile: &v.tile,
                turns: v.turns,
                units: Some(&v.units),
                faction: None,
                when: v.when.as_deref(),
            }],
            Self::Own(v) if v.turns.is_some() => vec![EventClock {
                predicate: v.predicate,
                tile: &v.tile,
                turns: v
                    .turns
                    .expect("timed ownership predicate has authored turns"),
                units: None,
                faction: Some(&v.faction),
                when: v.when.as_deref(),
            }],
            Self::All(v) | Self::Any(v) => v.iter().flat_map(Self::holds).collect(),
            Self::Not(v) => v.holds(),
            _ => vec![],
        }
    }
    fn uses_path(&self) -> bool {
        match self {
            Self::Proximity(v) => v.through,
            Self::Stand(v) => v.through,
            Self::All(v) | Self::Any(v) => v.iter().any(Self::uses_path),
            Self::Not(v) => v.uses_path(),
            _ => false,
        }
    }
}

impl ScriptTree for ScriptEvent {
    fn condition_observer(&self) -> Option<&str> {
        self.observer
            .as_deref()
            .or_else(|| self.objective.as_ref().map(|o| o.faction.as_str()))
    }
    fn objective_id(&self) -> &str {
        match self.objective.as_ref().and_then(|o| o.id.as_deref()) {
            Some(id) => id,
            None => &self.id,
        }
    }
    fn actions_recursive(&self) -> Vec<&EventAction> {
        fn collect<'a>(actions: &'a [EventAction], out: &mut Vec<&'a EventAction>) {
            for a in actions {
                out.push(a);
                match a {
                    EventAction::ForEachUnit(v) => collect(&v.actions, out),
                    EventAction::ForEachMark(v) => collect(&v.actions, out),
                    _ => {}
                }
            }
        }
        let mut out = vec![];
        collect(&self.actions, &mut out);
        out
    }
}
