use super::*;
use serde::{Deserialize, Serialize};
/// Stable campaign-local identity; display names are presentation data.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub id: Text,
    pub name: Text,
}
