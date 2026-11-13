//! Faction allegiance is independent of the two input/turn phases and AI policy.
use super::FactionPolicy;
use super::{ConditionRule, ConditionSide, TurnPhase};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Relation {
    Allied,
    Neutral,
    Hostile,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum Controller {
    Human,
    Passive,
    Ai {
        /// Omitted policies inherit the selected enemy/session policy.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        policy: Option<FactionPolicy>,
    },
}
/// Transfer the same unit when a shared authored condition is satisfied.
/// Contextual conditions use the unit's current tile.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct FactionTransfer {
    pub to: String,
    pub when: ConditionRule,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct FactionRecruit {
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<i32>,
}

#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct Faction {
    pub vision_group: String,
    pub id: String,
    /// Phase in which this controller acts; this is not its diplomatic relation.
    pub phase: TurnPhase,
    pub controller: Controller,
    pub palette: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Named factions author their own pool. Prices are recruitment economics,
    /// not modifications to a unit's combat profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recruits: Option<Vec<FactionRecruit>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relations: BTreeMap<String, Relation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer: Option<FactionTransfer>,
}

pub fn validate_economy(
    factions: &[Faction],
    gold: &super::Gold,
    tiles: &[super::Tile],
    types: &std::collections::HashMap<String, usize>,
) -> Result<(), String> {
    let known = |id: &str| factions.iter().any(|f| f.id == id);
    if gold.iter().any(|(id, amount)| !known(id) || *amount < 0)
        || factions.iter().any(|f| !gold.contains_key(&f.id))
    {
        return Err("invalid or missing faction treasury".into());
    }
    if tiles
        .iter()
        .any(|tile| tile.owner.as_deref().is_some_and(|id| !known(id)))
    {
        return Err("unknown settlement owner faction".into());
    }
    for faction in factions {
        if faction
            .recruits
            .as_ref()
            .is_some_and(|pool| pool.iter().any(|r| !types.contains_key(&r.unit)))
        {
            return Err("unknown faction recruit".into());
        }
    }
    Ok(())
}

pub fn validate_factions(factions: &[Faction], units: &[super::PublicUnit]) -> Result<(), String> {
    let known = |id: &str| factions.iter().any(|f| f.id == id);
    let mut ids = std::collections::HashSet::new();
    for faction in factions {
        if let Some(pool) = &faction.recruits {
            let mut seen = std::collections::HashSet::new();
            for recruit in pool {
                if recruit.cost.is_some_and(|cost| cost < 0)
                    || recruit.unit.is_empty()
                    || !seen.insert(&recruit.unit)
                {
                    return Err("invalid faction recruitment pool".into());
                }
            }
        }
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
        if matches!(faction.controller, Controller::Human) && faction.phase != TurnPhase::Player {
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
        if let Some(rule) = &faction.transfer {
            if !known(&rule.to) || rule.to == faction.id {
                return Err("invalid faction transfer".into());
            }
        }
    }
    for unit in units {
        validate_faction_reference(factions, &unit.faction)?;
    }
    Ok(())
}

/// Resolved presentation/control facts; derived in Rust rather than save data.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct UnitFaction {
    pub id: String,
    pub palette: String,
    pub relation: Relation,
    pub controllable: bool,
    pub phase: TurnPhase,
}

#[cfg_attr(not(target_arch = "wasm32"), inline)]

pub fn validate_faction_reference(factions: &[Faction], id: &str) -> Result<(), String> {
    if factions.iter().any(|faction| faction.id == id) {
        Ok(())
    } else {
        Err("unknown unit faction".to_owned())
    }
}

#[cfg_attr(not(target_arch = "wasm32"), inline)]

pub fn faction_relation(factions: &[Faction], first: &str, second: &str) -> Relation {
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
    Relation::Neutral
}
/// Match an explicit unit selector, not the phase in which it happens to act.
/// Keep this exhaustive: new allegiance selectors must define their semantics.
#[cfg_attr(not(target_arch = "wasm32"), inline)]
pub fn faction_matches_side(factions: &[Faction], id: &str, side: ConditionSide) -> bool {
    match side {
        ConditionSide::Enemy => faction_relation(factions, "player", id) == Relation::Hostile,
        ConditionSide::Neutral => faction_relation(factions, "player", id) == Relation::Neutral,
        ConditionSide::Any => true,
        ConditionSide::Observer => id == "player",
        ConditionSide::Player => factions
            .iter()
            .find(|f| f.id == id)
            .is_some_and(|f| f.controller == Controller::Human),
    }
}

#[cfg_attr(not(target_arch = "wasm32"), inline)]

pub fn validate_faction_conditions(
    factions: &[Faction],
    tiles: &[super::Tile],
    events: &[super::Event],
    types: &std::collections::HashMap<String, usize>,
) -> Result<(), String> {
    for faction in factions {
        if let Some(pool) = &faction.recruits {
            if pool.iter().any(|entry| !types.contains_key(&entry.unit)) {
                return Err("unknown faction recruit".into());
            }
        }
        if let Some(transfer) = &faction.transfer {
            transfer.when.condition.validate(tiles, events, true)?;
            transfer
                .when
                .condition
                .validate_references(types, factions)?;
        }
    }
    Ok(())
}
