use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Tactical and recruitment policy, preserving each policy's stable choices.
#[cfg_attr(
    any(feature = "bake", all(feature = "wasm", target_arch = "wasm32")),
    derive(hex_catalog_freeze_derive::Freeze)
)]
pub enum FactionPolicy {
    #[default]
    Classic,
    Rush,
    Balanced,
    Rotated,
    Situational,
    Predictor,
}
impl FactionPolicy {
    /// Supported presets in their stable authored order.
    pub const ALL: [Self; 6] = [
        Self::Classic,
        Self::Rush,
        Self::Balanced,
        Self::Rotated,
        Self::Situational,
        Self::Predictor,
    ];
    /// Stable JSON/CLI policy identifier.
    #[cfg_attr(not(target_arch = "wasm32"), inline)]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Rush => "rush",
            Self::Balanced => "balanced",
            Self::Rotated => "rotated",
            Self::Situational => "situational",
            Self::Predictor => "predictor",
        }
    }
}
