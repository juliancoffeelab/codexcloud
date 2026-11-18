use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentEffectDefinition {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub effects: Vec<EnvironmentEffect>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EnvironmentEffect {
    AmbientLight { level: Illumination },
    Concealment { rule: ConcealmentRule },
    RememberTerrain { remember: bool },
    Light { light: TileLight },
    Unit { effect: ScopedEffect },
}
