use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbilityDefinition {
    pub id: Text,
    pub name: Text,
    pub description: Text,
    pub effects: Vec<ScopedEffect>,
}
