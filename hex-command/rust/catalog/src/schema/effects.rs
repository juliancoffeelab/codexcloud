use super::*;
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectScope {
    pub radius: u16,
    pub include_self: bool,
    pub targets: HealingTargets,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectCombine {
    Add,
    Max,
    Union,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackingRule {
    pub group: Text,
    pub combine: EffectCombine,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedEffect {
    pub scope: EffectScope,
    pub stacking: StackingRule,
    pub effect: EffectValue,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EffectValue {
    Healing {
        power: i32,
    },
    Thorns {
        #[serde(rename = "damagePerStrike")]
        damage_per_strike: i32,
        #[serde(rename = "incomingCategory")]
        incoming_category: Category,
    },
    Sight {
        range: i32,
        unlimited: bool,
        #[serde(rename = "sightModifiers")]
        sight_modifiers: SightModifiers,
        penetrates: Vec<ConcealmentKind>,
    },
    Concealment {
        rule: ConcealmentRule,
    },
    Light {
        radius: u16,
    },
    StatModifiers {
        modifiers: StatModifiers,
    },
}
