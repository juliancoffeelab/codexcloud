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
    /// Validate current state before constructing runtime indexes.
    pub(crate) fn try_from_state(rules: &'catalog Rules, state: State) -> Result<Self, StateError> {
        validate_state(rules, &state)?;
        Ok(Self::from_state(rules, state))
    }
}
/// Check serialized state without modifying it or touching a live battle.
pub fn validate_state(rules: &Rules, state: &State) -> Result<(), StateError> {
    if state.cols <= 0 || state.rows <= 0 || state.round == 0 {
        return Err(StateError("invalid board dimensions or round".into()));
    }
    if state.tiles.is_empty() {
        return Err(StateError("board has no tiles".into()));
    }
    let authored = rules
        .maps
        .get(&(state.map_id.clone(), state.map_revision))
        .ok_or_else(|| StateError("unknown map revision".into()))?;
    validate_state_geometry(rules, state, authored)
}
/// Native generated arenas use a validated generated layout instead of an authored map.
pub fn validate_state_geometry(
    rules: &Rules,
    state: &State,
    authored: &State,
) -> Result<(), StateError> {
    let definition = state
        .campaign
        .as_ref()
        .and_then(|state| rules.campaigns.get(&state.id));
    validate_state_geometry_with_campaign(rules, state, authored, definition)
}
pub fn validate_state_geometry_with_campaign(
    rules: &Rules,
    state: &State,
    authored: &State,
    campaign_definition: Option<&CampaignRules>,
) -> Result<(), StateError> {
    let fail = |message: &str| StateError(message.to_owned());
    let registry = campaign_definition.or_else(|| rules.campaign_for_map(&state.map_id));
    let check_character = |id: &str| -> Result<(), StateError> {
        if registry.is_some_and(|c| c.character(id).is_some()) {
            Ok(())
        } else {
            Err(fail("unknown saved character"))
        }
    };
    for unit in &state.units {
        if let Some(id) = &unit.character {
            check_character(id)?;
        }
    }
    if let Some(campaign) = &state.campaign {
        for record in &campaign.evacuated {
            if let Some(id) = &record.unit.character {
                check_character(id)?;
            }
        }
        for veteran in campaign
            .veterans
            .iter()
            .chain(&campaign.story_veterans)
            .flatten()
        {
            if let Some(id) = &veteran.character {
                check_character(id)?;
            }
        }
        for line in &campaign.dialogue {
            if let Some(id) = &line.character {
                check_character(id)?;
            }
        }
    }

    validate_factions(&state.factions, &state.units).map_err(StateError)?;
    validate_economy(&state.factions, &state.gold, &state.tiles, &rules.types)
        .map_err(StateError)?;
    for required in &authored.factions {
        if !state
            .factions
            .iter()
            .any(|faction| faction.id == required.id)
        {
            return Err(fail("missing authored faction definition"));
        }
    }
    if state.cols <= 0 || state.rows <= 0 || state.round == 0 {
        return Err(fail("invalid board dimensions or round"));
    }
    if let Some(armies) = &state.armies {
        if state.campaign.is_some()
            || armies
                .iter()
                .any(|id| !rules.rosters.iter().any(|r| &r.id == id))
        {
            return Err(StateError("invalid skirmish factions".into()));
        }
    }
    if state.tiles.is_empty() {
        return Err(fail("board has no tiles"));
    }
    if state.map_id != authored.map_id || state.map_revision != authored.map_revision {
        return Err(fail("unknown map revision"));
    }
    if state.cols != authored.cols
        || state.rows != authored.rows
        || state.tiles.len() != authored.tiles.len()
    {
        return Err(fail("invalid map geometry"));
    }
    let authored_tiles: HashSet<_> = authored
        .tiles
        .iter()
        .map(|tile| {
            (
                tile.id.as_str(),
                tile.location.as_deref(),
                tile.col,
                tile.row,
            )
        })
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
        if !authored_tiles.contains(&(
            tile.id.as_str(),
            tile.location.as_deref(),
            tile.col,
            tile.row,
        )) {
            return Err(fail("invalid tile identity or coordinate"));
        }
        if tile.col < 0 || tile.col >= state.cols || tile.row < 0 || tile.row >= state.rows {
            return Err(fail("tile is outside the board"));
        }
        if !rules.terrains.contains_key(&tile.terrain) {
            return Err(fail("unknown terrain"));
        }
    }
    let mut allowed_memory: HashSet<String> = authored
        .tiles
        .iter()
        .filter(|tile| tile.fog.as_ref().is_some_and(|rule| rule.remember))
        .map(|tile| format!("fog:{}", tile.id))
        .collect();
    for tile in &authored.tiles {
        if tile
            .light
            .as_ref()
            .is_some_and(|light| light.when.as_ref().is_some_and(|rule| rule.remember))
        {
            allowed_memory.insert(format!("light:{}", tile.id));
        }
    }
    if let Some(campaign) = &state.campaign {
        if let Some(mission) = campaign_definition.and_then(|c| c.missions.get(campaign.mission)) {
            for group in &mission.enemy_groups {
                if group.when.as_ref().is_some_and(|rule| rule.remember) {
                    allowed_memory.insert(format!("group:{}", group.id));
                }
            }
            for unit in &state.units {
                if let Some(id) = &unit.ai_group {
                    if unit.side != Side::Enemy || !mission.enemy_groups.iter().any(|g| g.id == *id)
                    {
                        return Err(fail("invalid enemy group"));
                    }
                }
            }
        }
    }
    let mut remembered = HashSet::new();
    for key in &state.condition_memory {
        if !allowed_memory.contains(key) || !remembered.insert(key) {
            return Err(fail("invalid remembered condition"));
        }
    }
    for ground in &state.ground_items {
        if !positions.contains(&(ground.tile.col, ground.tile.row)) {
            return Err(fail("invalid ground item tile"));
        }
        ground
            .item
            .validate("ground item")
            .map_err(|e| StateError(e.to_string()))?;
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
        if !positions.contains(&(unit.tile.col, unit.tile.row)) || !occupied.insert(unit.tile) {
            return Err(fail("invalid unit position"));
        }
        let kind = *rules
            .types
            .get(&unit.r#type)
            .ok_or_else(|| fail("unknown unit type"))?;
        validate_equipment(
            &unit.items,
            &rules.units[kind].attacks,
            "saved unit equipment",
        )
        .map_err(|e| StateError(e.to_string()))?;
        if unit.hp <= 0 || unit.hp > rules.units[kind].hp {
            return Err(fail("invalid unit hit points"));
        }
        if unit
            .experience
            .ok_or_else(|| fail("missing unit experience"))?
            > rules.units[kind].experience_needed.unwrap_or(0)
        {
            return Err(fail("invalid unit experience"));
        }
    }
    if let Some(campaign) = &state.campaign {
        let definition = campaign_definition.ok_or_else(|| fail("unknown campaign"))?;
        if definition.id != campaign.id {
            return Err(fail("campaign definition and saved campaign differ"));
        }
        if campaign.mission >= definition.missions.len() {
            return Err(fail("unknown mission"));
        }
        let mission = &definition.missions[campaign.mission];
        if mission.map.id != state.map_id {
            return Err(fail("campaign mission and map differ"));
        }
        for (id, history) in &campaign.flags {
            crate::data::catalog::identifier(id, "saved flag")
                .map_err(|e| StateError(e.message))?;
            if history.is_empty()
                || history.iter().any(|v| {
                    v.stage > campaign.mission
                        || (v.stage == campaign.mission && v.turn > state.round)
                })
                || history
                    .windows(2)
                    .any(|pair| (pair[0].stage, pair[0].turn) > (pair[1].stage, pair[1].turn))
            {
                return Err(fail("invalid flag history"));
            }
        }
        if mission
            .all_goals()
            .flat_map(Goal::sites)
            .any(|site| !state.tiles.iter().any(|t| t.matches_reference(&site)))
        {
            return Err(fail("missing objective tile"));
        }
        super::events::validate_event_state(rules, state, mission).map_err(StateError)?;
        for record in &campaign.evacuated {
            let unit = &record.unit;
            let id = unit
                .id
                .strip_prefix('u')
                .and_then(|id| id.parse::<u32>().ok())
                .filter(|id| unit.id == format!("u{id}") && *id < state.next_id)
                .ok_or_else(|| fail("invalid evacuated unit identity"))?;
            if !ids.insert(id) {
                return Err(fail("duplicate evacuated unit identity"));
            }
            let event = mission
                .events
                .iter()
                .find(|event| event.id == record.event)
                .ok_or_else(|| fail("unknown evacuation event"))?;
            let effect = event
                .evacuate
                .as_ref()
                .ok_or_else(|| fail("event does not evacuate units"))?;
            if !campaign.marked_prefixed("event:", &record.event)
                || !state
                    .tiles
                    .iter()
                    .any(|t| t.hex() == unit.tile && t.matches_reference(&effect.tile))
                || unit.side != effect.units.side
                || effect
                    .units
                    .role
                    .is_some_and(|role| unit.role != Some(role))
                || effect
                    .units
                    .character
                    .as_ref()
                    .is_some_and(|name| unit.character.as_ref() != Some(name))
            {
                return Err(fail("invalid evacuated unit outcome"));
            }
            let kind = *rules
                .types
                .get(&unit.r#type)
                .ok_or_else(|| fail("unknown evacuated unit type"))?;
            if unit.hp <= 0 || unit.hp > rules.units[kind].hp {
                return Err(fail("invalid evacuated unit hit points"));
            }
            validate_equipment(
                &unit.items,
                &rules.units[kind].attacks,
                "evacuated unit equipment",
            )
            .map_err(|error| StateError(error.to_string()))?;
            validate_faction_reference(&state.factions, unit.faction.as_deref(), unit.side)
                .map_err(StateError)?;
        }
        if campaign.dialogue_index > campaign.dialogue.len() {
            return Err(fail("invalid dialogue index"));
        }
        if !(0..=8).contains(&campaign.carry_gold)
            || campaign.recall.len() > 4
            || campaign
                .recall
                .iter()
                .any(|id| rules.types.get(id).is_none())
        {
            return Err(fail("invalid campaign carryover"));
        }
        let previous_companions: HashSet<_> = definition.missions
            [..campaign.mission + usize::from(!mission.scripted_events.is_empty())]
            .iter()
            .flat_map(|mission| mission.companion_ids())
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
            validate_equipment(
                &veteran.items,
                &rules.units[kind].attacks,
                "veteran equipment",
            )
            .map_err(|e| StateError(e.to_string()))?;
            if veteran.experience > rules.units[kind].experience_needed.unwrap_or(0) {
                return Err(fail("invalid veteran experience"));
            }
        }
        if let Some(story) = &campaign.story_veterans {
            if story.len() > 8
                || story.iter().any(|veteran| {
                    let family = &rules.units[rules.types[&veteran.r#type]].family;
                    !(veteran.role.is_none()
                        && mission
                            .arrivals
                            .iter()
                            .any(|arrival| veteran.character.as_ref() == Some(&arrival.character)))
                        && !mission.map.units.iter().any(|unit| {
                            unit.side == Side::Player
                                && (if veteran.role == Some(Role::Leader) {
                                    unit.role == Some(Role::Leader)
                                } else {
                                    veteran.character.is_some()
                                        && veteran.character == unit.character
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
                && !state
                    .units
                    .iter()
                    .flat_map(|u| &u.items)
                    .flat_map(|i| &i.attacks)
                    .any(|a| a.id == combat.attack_id)
                && !state
                    .ground_items
                    .iter()
                    .flat_map(|g| &g.item.attacks)
                    .any(|a| a.id == combat.attack_id)
                && !rules
                    .items
                    .values()
                    .flat_map(|i| &i.attacks)
                    .any(|a| a.id == combat.attack_id)
                && !rules
                    .campaigns
                    .values()
                    .filter(|candidate| {
                        campaign_definition.is_none_or(|definition| candidate.id != definition.id)
                    })
                    .chain(campaign_definition)
                    .flat_map(|c| &c.missions)
                    .flat_map(|m| &m.events)
                    .flat_map(|e| &e.items)
                    .flat_map(|r| &r.item.attacks)
                    .any(|a| a.id == combat.attack_id)
        {
            return Err(fail("invalid last combat"));
        }
    }
    Ok(())
}
