//! Battle rules and campaign transitions, independent of transport and policy.
pub(crate) mod battle;
mod capabilities;
pub(crate) mod combat;
pub mod combat_matchup;
mod commands;
pub mod damage;
pub(crate) mod definition_index;
mod detection;
#[cfg(feature = "simulation")]
mod diagnostics;
pub(crate) mod effect_index;
pub(crate) mod event_conditions;
pub(crate) mod event_queries;
mod event_state;
pub(crate) mod events;
pub mod exchange;
mod execution;
mod factions;
#[cfg(feature = "wasm")]
pub(crate) mod inspection;
mod items;
pub(crate) mod movement;
mod perception;
pub(crate) use detection::Perception;
mod recruit_pool;
mod restore;
mod setup;
mod snapshot_checks;
mod strike;
pub use strike::CombatStrike;
mod turns;
mod visibility;
pub use restore::{StateError, validate_state};
#[cfg(not(target_arch = "wasm32"))]
pub use restore::{validate_state_geometry, validate_state_geometry_with_campaign};
pub mod campaign;
pub(crate) mod campaign_program;
pub(crate) use battle::{BattlePosition, LiveBattle};

pub mod geometry;

pub mod conditions;
