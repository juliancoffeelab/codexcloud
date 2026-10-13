//! Battle rules and campaign transitions, independent of transport and policy.
mod api;
mod effects;
#[cfg(feature = "wasm")]
pub(crate) use effects::complete_effects;
pub(crate) mod battle;
pub use crate::data::MovementCost;
pub use api::{Battle, BattleError, CombatPreview, Reachable};
mod combat;
mod commands;
#[cfg(feature = "wasm")]
mod inspection;
mod movement;
mod restore;
mod turns;
pub(crate) use restore::validate_state_geometry;
pub use restore::{StateError, validate_state};
pub mod campaign;
pub use crate::data::{Command, CommandError, Effects};
pub(crate) use battle::Game;

pub(crate) mod geometry;
