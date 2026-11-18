use super::*;
use serde::{Deserialize, Serialize};
/// A named survivor enters the next mission automatically, outside recall slots.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanionArrival {
    pub character: Text,
    pub tile: Hex,
}
/// Local orders constrain only members of the authored group.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum GroupOrder {
    Hold {
        observer: Option<Text>,
        when: Option<Condition>,
        area: Vec<TileRef>,
        #[serde(rename = "attackRadius")]
        attack_radius: u16,
    },
}
/// Independent mission-controlled enemies; activation and goals do not require leaders.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnemyGroup {
    pub id: Text,
    pub order: Option<GroupOrder>,
    pub when: Option<ConditionRule>,
}
/// An ordered set of authored chapters and their final dialogue.
/// Authored limits and ranking for chapter-to-chapter carryover.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CampaignCarryover {
    pub gold_limit: i32,
    pub veteran_limit: usize,
    pub deployment_radius: u16,
    pub weights: VeteranWeights,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VeteranWeights {
    pub hp: f64,
    pub damage: f64,
    pub healing: f64,
    pub level: f64,
    pub experience: f64,
}
