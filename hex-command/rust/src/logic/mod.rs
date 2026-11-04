//! Mechanical battle API is owned by the internal engine crate.
pub use hex_command_battle::{BattleSession as Game, StateError, validate_state, conditions, damage, exchange, geometry};
pub mod battle { pub use hex_command_battle::{BattleSession as Game, Board, RuntimeCache, hex_distance}; }
#[cfg(not(target_arch="wasm32"))]
pub use hex_command_battle::{validate_state_geometry, validate_state_geometry_with_campaign};

pub use hex_command_catalog::data::{Command, CommandError};
