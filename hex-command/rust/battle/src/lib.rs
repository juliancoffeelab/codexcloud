//! Private-storage mechanical battle engine. Hosts lend a catalog and own sessions.
pub mod data {
    pub use hex_command_catalog::data::*;
}
mod logic;
pub mod model;
mod sort;
mod view;
pub use logic::battle::{Board, RuntimeCache, hex_distance};
pub use logic::geometry::TileSet;
pub use logic::{StateError, validate_state};
pub use logic::{conditions, damage, exchange, geometry};
#[cfg(not(target_arch = "wasm32"))]
pub use logic::{validate_state_geometry, validate_state_geometry_with_campaign};
pub use sort::SharedSort;
pub use view::*;

#[cfg(test)]
mod tests;
