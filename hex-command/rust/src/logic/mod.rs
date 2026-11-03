//! Battle rules and campaign transitions, independent of transport and policy.
pub(crate) mod battle;
mod combat;
mod commands;
pub(crate) mod damage;
pub(crate) mod events;
pub(crate) mod exchange;
mod factions;
#[cfg(any(test, feature = "wasm"))]
mod inspection;
mod items;
mod movement;
mod restore;
mod turns;
pub use restore::{StateError, validate_state};
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use restore::{validate_state_geometry, validate_state_geometry_with_campaign};
pub mod campaign;
pub use crate::data::{Command, CommandError};
pub(crate) use battle::Game;

pub(crate) mod geometry;

pub mod conditions;
