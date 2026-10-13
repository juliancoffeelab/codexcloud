//! Validation of saved states before runtime indexes are constructed.

use super::battle::Game;
use crate::data::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
/// A malformed snapshot rejected before constructing runtime indexes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateError(pub String);
impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StateError {}
impl<'catalog> Game<'catalog> {
    /// Restore against current unit limits, then validate the roster and references.
    pub(crate) fn try_from_state(
        rules: &'catalog Rules,
        mut state: State,
    ) -> Result<Self, StateError> {
        for unit in &mut state.units {
            if let Some(&kind) = rules.types.get(&unit.r#type) {
                unit.hp = unit.hp.min(rules.units[kind].hp);
            }
        }
        validate_state(rules, &state)?;
        Ok(Self::from_state(rules, state))
    }
}
/// Check serialized state without modifying it or touching a live battle.
pub fn validate_state(rules: &Rules, state: &State) -> Result<(), StateError> {
    let fail = |message: &str| StateError(message.to_owned());
    if state.cols <= 0 || state.rows <= 0 || state.round == 0 {
        return Err(fail("invalid board dimensions or round"));
    }
    if state.tiles.is_empty() {
        return Err(fail("board has no tiles"));
    }
    let authored = rules
        .maps
        .get(&(state.map_id.clone(), state.map_revision))
        .ok_or_else(|| fail("unknown map revision"))?;
    if state.cols != authored.cols
        || state.rows != authored.rows
        || state.tiles.len() != authored.tiles.len()
    {
        return Err(fail("invalid map geometry"));
    }
    let authored_tiles: HashSet<_> = authored
        .tiles
        .iter()
        .map(|tile| (tile.id.as_str(), tile.col, tile.row))
        .collect();
    let mut tiles = HashSet::new();
    let mut positions = HashSet::new();
    for tile in &state.tiles {
        if !tile.valid_connections() {
            return Err(fail("invalid artwork connections"));
        }
        if !tiles.insert(tile.id.as_str()) || !positions.insert((tile.col, tile.row)) {
            return Err(fail("duplicate tile identity or coordinate"));
        }
        if !authored_tiles.contains(&(tile.id.as_str(), tile.col, tile.row)) {
            return Err(fail("invalid tile identity or coordinate"));
        }
        if tile.col < 0 || tile.col >= state.cols || tile.row < 0 || tile.row >= state.rows {
            return Err(fail("tile is outside the board"));
        }
        if !rules.terrains.contains_key(&tile.terrain) {
            return Err(fail("unknown terrain"));
        }
    }
    let mut ids = HashSet::new();
    let mut occupied = HashSet::new();
    for unit in &state.units {
        let id = unit
            .id
            .strip_prefix('u')
            .and_then(|id| id.parse::<u32>().ok())
            .ok_or_else(|| fail("invalid unit identity"))?;
        if unit.id != format!("u{id}") {
            return Err(fail("invalid unit identity"));
        }
        if !ids.insert(id) || id >= state.next_id {
            return Err(fail("duplicate unit identity or invalid nextId"));
        }
        if !tiles.contains(unit.tile.as_str()) || !occupied.insert(unit.tile.as_str()) {
            return Err(fail("invalid unit position"));
        }
        let kind = *rules
            .types
            .get(&unit.r#type)
            .ok_or_else(|| fail("unknown unit type"))?;
        if unit.hp <= 0 || unit.hp > rules.units[kind].hp {
            return Err(fail("invalid unit hit points"));
        }
        if unit.experience.unwrap_or(0) > rules.units[kind].experience_needed.unwrap_or(0) {
            return Err(fail("invalid unit experience"));
        }
    }
    if let Some(campaign) = &state.campaign {
        let definition = rules
            .campaigns
            .get(&campaign.id)
            .ok_or_else(|| fail("unknown campaign"))?;
        if campaign.mission >= definition.missions.len() {
            return Err(fail("unknown mission"));
        }
        let mission = &definition.missions[campaign.mission];
        if mission.map.id != state.map_id {
            return Err(fail("campaign mission and map differ"));
        }
        if mission
            .goals
            .iter()
            .flat_map(Goal::sites)
            .any(|site| !tiles.contains(site.as_str()))
        {
            return Err(fail("missing objective tile"));
        }
        if campaign.dialogue_index > campaign.dialogue.len() {
            return Err(fail("invalid dialogue index"));
        }
        if !(0..=8).contains(&campaign.carry_gold)
            || campaign.recall.len() > 4
            || campaign.recall.iter().any(|id| {
                rules
                    .types
                    .get(id)
                    .is_none_or(|&kind| rules.units[kind].leader)
            })
        {
            return Err(fail("invalid campaign carryover"));
        }
        let previous_companions: HashSet<_> = definition.missions[..campaign.mission]
            .iter()
            .flat_map(|mission| &mission.map.units)
            .filter(|unit| unit.side == Side::Player && unit.role != Some(Role::Leader))
            .filter_map(|unit| unit.name.as_deref())
            .collect();
        if let Some(fallen) = &campaign.fallen_companions {
            let unique: HashSet<_> = fallen.iter().collect();
            if unique.len() != fallen.len()
                || fallen
                    .iter()
                    .any(|name| !previous_companions.contains(name.as_str()))
            {
                return Err(fail("invalid fallen companion"));
            }
        }
        if let Some(veterans) = &campaign.veterans {
            if veterans.len() != campaign.recall.len()
                || veterans
                    .iter()
                    .zip(&campaign.recall)
                    .any(|(veteran, id)| veteran.r#type != *id || veteran.role.is_some())
            {
                return Err(fail("invalid recalled veterans"));
            }
        }
        for veteran in campaign
            .veterans
            .iter()
            .chain(&campaign.story_veterans)
            .flatten()
        {
            let kind = *rules
                .types
                .get(&veteran.r#type)
                .ok_or_else(|| fail("unknown veteran type"))?;
            if veteran.experience > rules.units[kind].experience_needed.unwrap_or(0) {
                return Err(fail("invalid veteran experience"));
            }
        }
        if let Some(story) = &campaign.story_veterans {
            if story.len() > 8
                || story.iter().any(|veteran| {
                    let family = &rules.units[rules.types[&veteran.r#type]].family;
                    !mission.map.units.iter().any(|unit| {
                        unit.side == Side::Player
                            && (if veteran.role == Some(Role::Leader) {
                                unit.role == Some(Role::Leader)
                            } else {
                                veteran.name.is_some() && veteran.name == unit.name
                            })
                            && rules.units[rules.types[&unit.r#type]].family == *family
                    })
                })
            {
                return Err(fail("invalid story veterans"));
            }
        }
    }
    if let Some(Some(combat)) = &state.last_combat {
        let attacker_id = combat
            .attacker_id
            .strip_prefix('u')
            .and_then(|id| id.parse::<u32>().ok());
        if attacker_id.is_none()
            || !rules
                .units
                .iter()
                .any(|u| u.attacks.iter().any(|a| a.id == combat.attack_id))
        {
            return Err(fail("invalid last combat"));
        }
    }
    Ok(())
}
