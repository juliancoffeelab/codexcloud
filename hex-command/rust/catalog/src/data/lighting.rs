//! Authored scenario lighting, loaded and validated with the Rust catalog.
use super::{ConditionRule, Event, Hex, Tile};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum TimeOfDay {
    #[default]
    Day,
    Night,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct BattlefieldLight {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub color: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub fixture: Option<bool>,
    pub col: i32,
    pub row: i32,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dx: Option<f64>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dy: Option<f64>,
    pub radius: f64,
    pub power: f64,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<ConditionRule>,
}

#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct TileLight {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub color: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "wasm", tsify(optional))]
    pub fixture: Option<bool>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dx: Option<f64>,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dy: Option<f64>,
    pub radius: f64,
    pub power: f64,
    #[cfg_attr(feature = "wasm", tsify(optional))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<ConditionRule>,
}

/// Map-wide ambience; terrain emitters belong to their tiles.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct MapLighting {
    pub time: TimeOfDay,
}
impl TileLight {
    pub fn at(&self, col: i32, row: i32) -> BattlefieldLight {
        BattlefieldLight {
            col,
            row,
            color: self.color,
            fixture: self.fixture,
            dx: self.dx,
            dy: self.dy,
            radius: self.radius,
            power: self.power,
            when: None,
        }
    }
    pub fn validate(&self, tiles: &[Tile], events: &[Event]) -> Result<(), String> {
        if !self.radius.is_finite()
            || self.radius <= 0.0
            || !self.power.is_finite()
            || self.power < 0.0
            || self.dx.is_some_and(|v| !v.is_finite())
            || self.dy.is_some_and(|v| !v.is_finite())
        {
            return Err(
                "light requires a positive radius, nonnegative power and finite offsets".into(),
            );
        }
        if let Some(rule) = &self.when {
            rule.condition.validate_presentation()?;
            rule.condition.validate(tiles, events, true)?;
        }
        Ok(())
    }
}
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct BattlefieldLighting {
    pub time: TimeOfDay,
    #[serde(default)]
    pub lights: Vec<BattlefieldLight>,
}

impl BattlefieldLighting {
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub fn validate(&self, tiles: &[Tile], events: &[Event]) -> Result<(), String> {
        for light in &self.lights {
            if !tiles
                .iter()
                .any(|tile| tile.col == light.col && tile.row == light.row)
            {
                return Err(format!(
                    "light is outside the map: {},{}",
                    light.col, light.row
                ));
            }
            if !light.radius.is_finite()
                || light.radius <= 0.0
                || !light.power.is_finite()
                || light.power < 0.0
                || light.dx.is_some_and(|value| !value.is_finite())
                || light.dy.is_some_and(|value| !value.is_finite())
            {
                return Err(
                    "light requires a positive radius, nonnegative power and finite offsets".into(),
                );
            }
            if let Some(rule) = &light.when {
                rule.condition.validate_presentation()?;
                rule.condition.validate(tiles, events, true)?;
            }
        }
        Ok(())
    }
}

/// Resolved current presentation; sent with views, never persisted as a cache.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub struct BattlefieldPresentation {
    pub lighting: BattlefieldLighting,
    pub fog: Vec<Hex>,
}
