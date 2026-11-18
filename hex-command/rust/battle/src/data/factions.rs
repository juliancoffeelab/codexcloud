use super::*;
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Faction {
    pub vision_group: Text,
    pub id: Text,
    /// Phase in which this controller acts; this is not its diplomatic relation.
    pub phase: TurnPhase,
    pub controller: Controller,
    pub palette: Text,
    pub name: Option<Text>,
    /// Named factions author their own pool. Prices are recruitment economics,
    /// not modifications to a unit's combat profile.
    #[serde(deserialize_with = "super::explicit_option")]
    pub recruits: Option<Vec<FactionRecruit>>,
    pub relations: BTreeMap<Text, Relation>,
    #[serde(deserialize_with = "super::explicit_option")]
    pub transfer: Option<FactionTransfer>,
}
/// Resolved presentation/control facts; derived in Rust rather than save data.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitFaction {
    pub id: Text,
    pub palette: Text,
    pub relation: Relation,
    pub controllable: bool,
    pub phase: TurnPhase,
}
pub fn validate_faction_reference(factions: &[Faction], id: &str) -> Result<(), Text> {
    if factions.iter().any(|faction| faction.id == id) {
        Ok(())
    } else {
        Err("unknown unit faction".to_owned())
    }
}
pub fn faction_relation(factions: &[Faction], first: &str, second: &str) -> Relation {
    *factions
        .iter()
        .find(|f| f.id == first)
        .expect("validated faction identifier")
        .relations
        .get(second)
        .expect("validated complete relation matrix")
}
/// Match an explicit unit selector, not the phase in which it happens to act.
/// Keep this exhaustive: new allegiance selectors must define their semantics.
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

pub fn validate_economy(
    factions: &[Faction],
    gold: &super::Gold,
    tiles: &[super::Tile],
    catalogue: &Catalogue,
) -> Result<(), Text> {
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
        if faction.recruits.as_ref().is_some_and(|pool| {
            pool.iter()
                .any(|r| !catalogue.units().any(|unit| unit.id() == r.unit))
        }) {
            return Err("unknown faction recruit".into());
        }
    }
    Ok(())
}

pub fn validate_factions(factions: &[Faction], units: &[super::PublicUnit]) -> Result<(), Text> {
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
        if factions
            .iter()
            .any(|other| !faction.relations.contains_key(&other.id))
        {
            return Err("missing faction relation; author the complete matrix".into());
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
