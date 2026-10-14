//! Battle rules and campaign transitions, independent of transport and policy.
#[cfg(feature = "wasm")]
mod effects;
#[cfg(feature = "wasm")]
pub(crate) use effects::complete_effects;
pub(crate) mod battle;
mod combat;
mod commands;
#[cfg(feature = "wasm")]
mod inspection;
mod movement;
mod restore;
mod turns;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use restore::validate_state_geometry;
pub use restore::{StateError, validate_state};
pub mod campaign;
pub use crate::data::{Command, CommandError, Effects};
pub(crate) use battle::Game;

pub(crate) mod geometry;
