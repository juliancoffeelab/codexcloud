//! Validation of saved states before runtime indexes are constructed.

use super::battle::LiveBattle;
use super::snapshot_checks as checks;
use crate::CampaignProgram;
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
impl<'catalog> LiveBattle<'catalog> {
    /// Validate current state before constructing runtime indexes.
    pub(crate) fn try_from_state(
        rules: &'catalog Catalogue,
        state: State,
    ) -> Result<Self, StateError> {
        validate_state(rules, &state)?;
        Ok(Self::from_state(rules, state))
    }
}
/// Check serialized state without modifying it or touching a live battle.
pub fn validate_state(rules: &Catalogue, state: &State) -> Result<(), StateError> {
    if state.cols <= 0 || state.rows <= 0 || state.round == 0 {
        return Err(StateError("invalid board dimensions or round".into()));
    }
    if state.tiles.is_empty() {
        return Err(StateError("board has no tiles".into()));
    }
    let map = rules
        .maps()
        .find(|map| map.id() == state.map_id && map.revision() == state.map_revision)
        .ok_or_else(|| StateError("unknown map revision".into()))?;
    let authored = super::setup::initial_state(rules, map);
    validate_state_geometry(rules, state, &authored)
}
/// Native generated arenas use a validated generated layout instead of an authored map.
pub fn validate_state_geometry(
    rules: &Catalogue,
    state: &State,
    authored: &State,
) -> Result<(), StateError> {
    let definition = match &state.campaign {
        Some(saved) => {
            let definition = rules
                .campaigns()
                .find(|definition| definition.id() == saved.id)
                .ok_or_else(|| StateError("unknown campaign".into()))?;
            Some(CampaignProgram::compile(rules, definition, None))
        }
        None => None,
    };
    validate_state_geometry_with_campaign(rules, state, authored, definition.as_ref())
}
pub fn validate_state_geometry_with_campaign(
    rules: &Catalogue,
    state: &State,
    authored: &State,
    campaign_definition: Option<&CampaignProgram>,
) -> Result<(), StateError> {
    let fail = |message: &str| StateError(message.to_owned());
    let registry = match campaign_definition {
        Some(campaign) => Some(campaign.characters()),
        None => rules
            .campaigns()
            .find(|campaign| {
                campaign.chapters().iter().any(|chapter| {
                    chapter.map_id() == state.map_id && chapter.map_revision() == state.map_revision
                })
            })
            .map(|campaign| campaign.characters()),
    };
    let check_character = |id: &str| -> Result<(), StateError> {
        if registry.is_some_and(|characters| characters.iter().any(|character| character.id == id))
        {
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
    for faction in &state.factions {
        if let Some(transfer) = &faction.transfer {
            checks::condition(&transfer.when, rules, state).map_err(StateError)?;
        }
    }
    for rule in state
        .tiles
        .iter()
        .filter_map(|t| t.light.as_ref())
        .filter_map(|l| l.when.as_ref())
    {
        checks::condition(rule, rules, state).map_err(StateError)?;
    }

    validate_economy(&state.factions, &state.gold, &state.tiles, rules).map_err(StateError)?;
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
                .any(|id| !rules.meta().simulation.rosters.iter().any(|r| &r.id == id))
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
    checks::perception(state).map_err(StateError)?;
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
        if !rules
            .terrains()
            .any(|terrain| terrain.id().as_str() == tile.terrain)
        {
            return Err(fail("unknown terrain"));
        }
    }
    let mut allowed_memory: HashSet<String> = HashSet::new();
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
        if let Some(mission) = campaign_definition.and_then(|c| c.chapters().get(campaign.mission))
        {
            for group in &mission.enemy_groups {
                if group.when.as_ref().is_some_and(|rule| rule.remember) {
                    allowed_memory.insert(format!("group:{}", group.id));
                }
            }
            for unit in &state.units {
                if let Some(id) = &unit.ai_group {
                    if !mission.enemy_groups.iter().any(|g| g.id == *id) {
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
        if !rules.items().any(|item| item.id() == ground.item) {
            return Err(fail("unknown ground item"));
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
        if !positions.contains(&(unit.tile.col, unit.tile.row)) || !occupied.insert(unit.tile) {
            return Err(fail("invalid unit position"));
        }
        let definition = rules
            .units()
            .find(|definition| definition.id() == unit.r#type)
            .ok_or_else(|| fail("unknown unit type"))?;
        checks::equipment(rules, state, &unit.r#type, &unit.items).map_err(StateError)?;
        if unit.hp <= 0 || unit.hp > definition.max_hp {
            return Err(fail("invalid unit hit points"));
        }
        if unit
            .experience
            .ok_or_else(|| fail("missing unit experience"))?
            > definition.max_xp
        {
            return Err(fail("invalid unit experience"));
        }
    }
    if let Some(campaign) = &state.campaign {
        let definition = campaign_definition.ok_or_else(|| fail("unknown campaign"))?;
        if definition.id() != campaign.id {
            return Err(fail("campaign definition and saved campaign differ"));
        }
        if campaign.mission >= definition.chapters().len() {
            return Err(fail("unknown mission"));
        }
        let mission = &definition.chapters()[campaign.mission];
        if campaign.veterans.is_none()
            || campaign.story_veterans.is_none()
            || campaign.fallen_companions.is_none()
        {
            return Err(fail("missing resolved campaign carryover"));
        }

        if mission.map.id != state.map_id {
            return Err(fail("campaign mission and map differ"));
        }
        for (id, history) in &campaign.flags {
            checks::identifier(id).map_err(StateError)?;
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
        super::events::validate_event_state(rules, state, mission).map_err(StateError)?;
        if !campaign.evacuated.is_empty() {
            return Err(fail("evacuation records require retired campaign events"));
        }
        if campaign.dialogue_index > campaign.dialogue.len() {
            return Err(fail("invalid dialogue index"));
        }
        if !(0..=definition.carryover().gold_limit).contains(&campaign.carry_gold)
            || campaign.recall.len() > definition.carryover().veteran_limit
            || campaign
                .recall
                .iter()
                .any(|id| !rules.units().any(|unit| unit.id() == id))
        {
            return Err(fail("invalid campaign carryover"));
        }
        let previous_companions: HashSet<_> = definition.chapters()
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
            let definition = rules
                .units()
                .find(|definition| definition.id() == veteran.r#type)
                .ok_or_else(|| fail("unknown veteran type"))?;
            checks::equipment(rules, state, &veteran.r#type, &veteran.items).map_err(StateError)?;
            if veteran.experience > definition.max_xp {
                return Err(fail("invalid veteran experience"));
            }
        }
        if let Some(story) = &campaign.story_veterans {
            if story.iter().any(|veteran| {
                let family = rules.unit(&veteran.r#type).family();
                !(veteran.role.is_none()
                    && mission
                        .arrivals
                        .iter()
                        .any(|arrival| veteran.character.as_ref() == Some(&arrival.character)))
                    && !mission.map.units.iter().any(|unit| {
                        unit.faction == "player"
                            && (if veteran.role == Some(Role::Leader) {
                                unit.role == Some(Role::Leader)
                            } else {
                                veteran.character.is_some() && veteran.character == unit.character
                            })
                            && rules.unit(&unit.r#type).family() == family
                    })
            }) {
                return Err(fail("invalid story veterans"));
            }
        }
    }
    if let Some(Some(combat)) = &state.last_combat {
        let valid_actor = combat
            .attacker_id
            .strip_prefix('u')
            .and_then(|id| id.parse::<u32>().ok())
            .is_some();
        let group = state
            .vision_groups
            .first()
            .ok_or_else(|| fail("missing vision groups"))?;
        let known_attack = rules
            .items()
            .any(|item| item.attacks().any(|attack| attack.id() == combat.attack_id))
            || rules.units().any(|unit| {
                let capabilities = unit.compose(UnitInputs {
                    vision_group_sight: &group.sight,
                    equipment: &[],
                    gained_abilities: &[],
                    selected_effects: &[],
                    map_modifiers: &state.map_modifiers,
                    perception_contributions: &[],
                });
                capabilities
                    .attacks()
                    .any(|attack| attack.id() == combat.attack_id)
            });
        if !valid_actor || !known_attack {
            return Err(fail("invalid last combat"));
        }
    }

    Ok(())
}
