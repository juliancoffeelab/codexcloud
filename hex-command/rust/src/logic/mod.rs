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
mod movement;
mod restore;
mod turns;
pub use restore::{StateError, validate_state};
pub mod campaign;
pub use crate::data::{Command, CommandError, Effects};
pub(crate) use battle::Game;
