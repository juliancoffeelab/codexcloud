use serde::{Deserialize, Serialize};
/// Clockwise edge names for a pointy-top hex. These describe artwork only.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HexDirection {
    E,
    Se,
    Sw,
    W,
    Nw,
    Ne,
}

impl HexDirection {
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
}
