//! Private-storage mechanical battle engine. Hosts lend a catalog and own sessions.
pub mod data {
    pub use hex_command_catalog::data::*;
}
mod command;
mod logic;
pub use command::RuntimeCommand;
pub mod model;
mod sort;
mod view;
pub use logic::battle::{BattleState, Board, hex_distance};
pub use logic::combat::CombatMatchup;
pub use logic::geometry::{AttackOriginRun, TileSet};
pub use logic::movement::Reachability;
pub use logic::{StateError, validate_state};
pub use logic::{conditions, damage, exchange, geometry};
#[cfg(not(target_arch = "wasm32"))]
pub use logic::{validate_state_geometry, validate_state_geometry_with_campaign};
pub use sort::SharedSort;
pub use view::*;

#[cfg(test)]
mod tests;

/// Runtime scope for transient numeric commands and AI continuations.
#[derive(Clone, Debug)]
pub struct BattleScope(std::rc::Rc<()>);
impl BattleScope {
    pub(crate) fn new() -> Self {
        Self(std::rc::Rc::new(()))
    }
    #[inline]
    pub fn same_session(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}
