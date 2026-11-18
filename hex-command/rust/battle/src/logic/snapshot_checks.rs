//! Referential checks for current runtime snapshots, independent of source DTOs.
use crate::data::*;
use std::collections::HashSet;

pub(super) fn identifier(id: &str) -> Result<(), String> {
    if id.is_empty()
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("invalid identifier".into());
    }
    Ok(())
}
pub(super) fn equipment(
    catalogue: &Catalogue,
    state: &State,
    unit_id: &str,
    items: &[String],
) -> Result<(), String> {
    let unit = catalogue
        .units()
        .find(|unit| unit.id() == unit_id)
        .ok_or("unknown unit type")?;
    let equipment = items
        .iter()
        .map(|id| {
            catalogue
                .items()
                .find(|item| item.id() == id)
                .ok_or("unknown item")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut ids = HashSet::new();
    if !equipment.iter().all(|item| ids.insert(item.id())) {
        return Err("duplicate item".into());
    }
    let group = state.vision_groups.first().ok_or("missing vision groups")?;
    let capabilities = unit.compose(UnitInputs {
        vision_group_sight: &group.sight,
        equipment: &equipment,
        gained_abilities: &[],
        selected_effects: &[],
        map_modifiers: &state.map_modifiers,
        perception_contributions: &[],
    });
    let mut ids = HashSet::new();
    if !capabilities.attacks().all(|attack| ids.insert(attack.id())) {
        return Err("duplicate equipped attack identity".into());
    }
    Ok(())
}
pub(super) fn condition(
    rule: &ConditionRule,
    catalogue: &Catalogue,
    state: &State,
) -> Result<(), String> {
    if rule
        .observer
        .as_ref()
        .is_some_and(|id| !state.factions.iter().any(|f| &f.id == id))
    {
        return Err("unknown condition observer".into());
    }
    fn visit(c: &Condition, catalogue: &Catalogue, state: &State) -> Result<(), String> {
        let unit = |id: &str| -> Result<(), String> {
            if catalogue.units().any(|unit| unit.id() == id) {
                Ok(())
            } else {
                Err("unknown condition unit type".into())
            }
        };
        match c {
            Condition::Owner { side, .. } => validate_faction_reference(&state.factions, side)?,
            Condition::Proximity {
                unit_type,
                unit_types,
                ..
            } => {
                for id in unit_type.iter().chain(unit_types) {
                    unit(id)?;
                }
            }
            Condition::VisibleUnits { unit_types, .. } => {
                for id in unit_types {
                    unit(id)?;
                }
            }
            Condition::HasUnits { unit_type, .. } | Condition::NoUnits { unit_type, .. } => {
                if let Some(id) = unit_type {
                    unit(id)?;
                }
            }
            Condition::All { conditions } | Condition::Any { conditions } => {
                for c in conditions {
                    visit(c, catalogue, state)?;
                }
            }
            Condition::Choice { .. }
            | Condition::Flag { .. }
            | Condition::Event { .. }
            | Condition::Round { .. } => {}
        }
        Ok(())
    }
    visit(&rule.condition, catalogue, state)
}
pub(super) fn light(light: &TileLight, catalogue: &Catalogue, state: &State) -> Result<(), String> {
    if !light.radius.is_finite()
        || light.radius <= 0.
        || !light.power.is_finite()
        || light.power < 0.
        || light.dx.is_some_and(|v| !v.is_finite())
        || light.dy.is_some_and(|v| !v.is_finite())
    {
        return Err("invalid saved light".into());
    }
    if let Some(rule) = &light.when {
        condition(rule, catalogue, state)?;
    }
    Ok(())
}
pub(super) fn perception(state: &State) -> Result<(), String> {
    let mut groups = HashSet::new();
    let positions: HashSet<_> = state.tiles.iter().map(Tile::hex).collect();
    for group in &state.vision_groups {
        identifier(&group.id)?;
        if !groups.insert(&group.id) {
            return Err("duplicate vision group".into());
        }
        let mut explored = HashSet::new();
        if group
            .explored
            .iter()
            .any(|hex| !positions.contains(hex) || !explored.insert(hex))
        {
            return Err("invalid explored hex".into());
        }
        let mut kinds = HashSet::new();
        if group
            .sight
            .penetrates
            .iter()
            .any(|kind| !kinds.insert(kind))
        {
            return Err("duplicate penetration kind".into());
        }
    }
    if state
        .factions
        .iter()
        .any(|f| !groups.contains(&f.vision_group))
    {
        return Err("unknown faction vision group".into());
    }
    for unit in &state.units {
        let mut revealed = HashSet::new();
        if unit
            .revealed_to
            .iter()
            .any(|id| !groups.contains(id) || !revealed.insert(id))
        {
            return Err("invalid combat reveal group".into());
        }
        let mut effects = HashSet::new();
        for effect in &unit.perception_effects {
            identifier(&effect.id)?;
            if !effects.insert(&effect.id) {
                return Err("duplicate perception effect".into());
            }
        }
    }
    Ok(())
}
