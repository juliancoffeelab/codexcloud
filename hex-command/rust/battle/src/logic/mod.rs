//! Battle rules and campaign transitions, independent of transport and policy.
pub(crate) mod battle;
mod combat;
mod commands;
pub mod damage;
pub(crate) mod events;
pub mod exchange;
mod factions;
mod inspection;
mod items;
mod movement;
mod restore;
mod turns;
pub use restore::{StateError, validate_state};
#[cfg(not(target_arch = "wasm32"))]
pub use restore::{validate_state_geometry, validate_state_geometry_with_campaign};
pub mod campaign;
pub(crate) use battle::Game;

pub mod geometry;

pub mod conditions;
