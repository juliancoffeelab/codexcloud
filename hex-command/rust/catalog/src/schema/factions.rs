use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Relation {
    Allied,
    Neutral,
    Hostile,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Controller {
    Human,
    Passive,
    Ai {
        /// Omitted policies inherit the selected enemy/session policy.
        #[serde(deserialize_with = "super::explicit_option")]
        policy: Option<FactionPolicy>,
    },
}
/// Transfer the same unit when a shared authored condition is satisfied.
/// Contextual conditions use the unit's current tile.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FactionTransfer {
    pub to: Text,
    pub when: ConditionRule,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FactionRecruit {
    pub unit: Text,
    #[serde(deserialize_with = "super::explicit_option")]
    pub cost: Option<i32>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FactionSetup {
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
