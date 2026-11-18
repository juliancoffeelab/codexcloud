//! Runtime state and commands owned by the battle engine.
pub use hex_command_catalog::*;
use std::collections::{BTreeMap, HashMap};
mod campaign;
pub use campaign::*;
mod commands;
pub use commands::*;
mod events;
pub use events::*;
mod objectives;
pub use objectives::*;
mod perception;
pub use perception::*;
pub(crate) mod presentation;
pub use presentation::*;
mod records;
pub use records::*;
mod references;
pub use references::*;
mod state;
pub use state::*;
mod units;
pub use units::*;

mod ids;
pub use ids::*;

mod factions;
pub use factions::*;

mod lighting;
pub use lighting::*;

mod inventory;
pub use inventory::*;

pub(crate) fn explicit_option<'de, D: serde::Deserializer<'de>, T: serde::Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    serde::Deserialize::deserialize(d)
}
