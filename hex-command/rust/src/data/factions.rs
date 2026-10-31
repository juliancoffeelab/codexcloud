//! Faction allegiance is independent of the two input/turn phases and AI policy.
use super::Side;
use crate::ai::Policy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Relation {
    Allied,
    Neutral,
    Hostile,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Controller {
    Human,
    Passive,
    Ai {
        #[serde(default)]
        policy: Policy,
    },
}
/// Transfer the same unit when another faction approaches; useful for rescuing
/// civilians, joining allies, and other proximity-based changes of control.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProximityTransfer {
    pub by: String,
    pub to: String,
    pub radius: u16,
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Faction {
    pub id: String,
    /// Phase in which this controller acts; this is not its diplomatic relation.
    pub phase: Side,
    pub controller: Controller,
    pub palette: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub relations: BTreeMap<String, Relation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub transfer_on_proximity: Option<ProximityTransfer>,
}
impl Side {
    pub fn faction_id(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Enemy => "enemy",
        }
    }
}

pub(crate) fn validate_factions(
    factions: &[Faction],
    units: &[super::PublicUnit],
) -> Result<(), String> {
    let known = |id: &str| matches!(id, "player" | "enemy") || factions.iter().any(|f| f.id == id);
    let mut ids = std::collections::HashSet::new();
    for faction in factions {
        if faction.palette.is_empty()
            || !faction
                .palette
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("invalid faction palette".into());
        }
        if faction.id.is_empty()
            || !faction
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !ids.insert(&faction.id)
        {
            return Err("invalid or duplicate faction identifier".into());
        }
        if faction.id == "player" && faction.phase != Side::Player
            || faction.id == "enemy" && faction.phase != Side::Enemy
            || matches!(faction.controller, Controller::Human) && faction.phase != Side::Player
        {
            return Err("invalid faction controller phase".into());
        }
        for (other, relation) in &faction.relations {
            if !known(other) || other == &faction.id && *relation != Relation::Allied {
                return Err("invalid faction relation".into());
            }
            if factions
                .iter()
                .find(|f| &f.id == other)
                .and_then(|f| f.relations.get(&faction.id))
                .is_some_and(|reverse| reverse != relation)
            {
                return Err("conflicting faction relation".into());
            }
        }
        if let Some(rule) = &faction.transfer_on_proximity {
            if !known(&rule.by) || !known(&rule.to) || rule.to == faction.id || rule.radius == 0 {
                return Err("invalid faction proximity transfer".into());
            }
        }
    }
    for unit in units {
        validate_faction_reference(factions, unit.faction.as_deref(), unit.side)?;
    }
    Ok(())
}

/// Resolved presentation/control facts; derived in Rust rather than save data.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitFaction {
    pub id: String,
    pub palette: String,
    pub relation: Relation,
    pub controllable: bool,
}

pub(crate) fn validate_faction_reference(
    factions: &[Faction],
    id: Option<&str>,
    phase: Side,
) -> Result<(), String> {
    let Some(id) = id else {
        return Ok(());
    };
    let faction = factions.iter().find(|f| f.id == id);
    if faction.is_none() && !matches!(id, "player" | "enemy") {
        return Err("unknown unit faction".into());
    }
    let expected = faction.map(|f| f.phase).unwrap_or(if id == "player" {
        Side::Player
    } else {
        Side::Enemy
    });
    if phase != expected {
        return Err("unit phase differs from its faction".into());
    }
    Ok(())
}

pub(crate) fn faction_relation(factions: &[Faction], first: &str, second: &str) -> Relation {
    if first == second {
        return Relation::Allied;
    }
    if let Some(relation) = factions
        .iter()
        .find(|f| f.id == first)
        .and_then(|f| f.relations.get(second))
        .or_else(|| {
            factions
                .iter()
                .find(|f| f.id == second)
                .and_then(|f| f.relations.get(first))
        })
    {
        return *relation;
    }
    if matches!((first, second), ("player", "enemy") | ("enemy", "player")) {
        Relation::Hostile
    } else {
        Relation::Neutral
    }
}
/// Authored player/enemy selectors refer to human control and hostile factions,
/// not merely the phase in which a unit happens to act.
pub(crate) fn faction_matches_side(
    factions: &[Faction],
    id: Option<&str>,
    phase: Side,
    side: Side,
) -> bool {
    let id = id.unwrap_or(phase.faction_id());
    match side {
        Side::Enemy => faction_relation(factions, "player", id) == Relation::Hostile,
        Side::Player => factions
            .iter()
            .find(|f| f.id == id)
            .map_or(phase == Side::Player, |f| f.controller == Controller::Human),
    }
}
