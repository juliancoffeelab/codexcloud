use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SightContribution {
    pub unlimited: bool,
    pub sight_modifiers: SightModifiers,
    pub penetrates: Vec<ConcealmentKind>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StatModifiers {
    pub sight: SightContribution,
    pub damage_modifiers: DamageModifiers,
    pub terrain_modifiers: BTreeMap<TerrainRole, TerrainStatModifiers>,
}
impl SightContribution {
    pub(crate) fn stack(&mut self, other: &Self) -> Result<(), Text> {
        self.unlimited |= other.unlimited;
        self.sight_modifiers.lit = self
            .sight_modifiers
            .lit
            .checked_add(other.sight_modifiers.lit)
            .ok_or("lit sight sum overflow")?;
        self.sight_modifiers.unlit = self
            .sight_modifiers
            .unlit
            .checked_add(other.sight_modifiers.unlit)
            .ok_or("unlit sight sum overflow")?;
        self.sight_modifiers.darkness = self
            .sight_modifiers
            .darkness
            .checked_add(other.sight_modifiers.darkness)
            .ok_or("darkness sight sum overflow")?;
        for kind in &other.penetrates {
            if !self.penetrates.contains(kind) {
                self.penetrates.push(*kind);
            }
        }
        Ok(())
    }
}
