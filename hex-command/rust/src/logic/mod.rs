//! Battle rules and campaign transitions, independent of transport and policy.
pub(crate) mod battle;
mod combat;
mod commands;
pub(crate) mod damage;
#[cfg(any(test, feature = "wasm"))]
mod inspection;
mod movement;
mod restore;
mod turns;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use restore::validate_state_geometry;
pub use restore::{StateError, validate_state};
pub mod campaign;
pub use crate::data::{Command, CommandError};
pub(crate) use battle::Game;

pub(crate) mod geometry;
