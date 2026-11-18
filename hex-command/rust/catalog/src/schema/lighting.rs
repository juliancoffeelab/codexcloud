use super::*;
use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeOfDay {
    Day,
    Night,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BattlefieldLight {
    pub color: Option<[u8; 3]>,
    pub fixture: Option<bool>,
    pub col: i32,
    pub row: i32,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub radius: f64,
    pub power: f64,
    pub when: Option<ConditionRule>,
}
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TileLight {
    #[serde(deserialize_with = "super::explicit_option")]
    pub emission: Option<super::LightEmission>,
    pub color: Option<[u8; 3]>,
    pub fixture: Option<bool>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub radius: f64,
    pub power: f64,
    pub when: Option<ConditionRule>,
}
/// Map ambience; explicit tile environmental effects may override it.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapLighting {
    pub time: TimeOfDay,
}
