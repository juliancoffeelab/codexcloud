//! Private-storage mechanical battle engine. Hosts lend a catalog and own sessions.
mod command;
pub mod data;
mod logic;
pub use command::{ActionProposal, ProposalOutcome, RuntimeCommand};
pub mod model;
pub use logic::campaign_program::CampaignProgram;
mod sort;
mod view;
pub use logic::battle::{BattleState, Board, hex_distance};
pub use logic::geometry::{AttackOriginRun, TileSet};
pub use logic::movement::{MovementField, Reachability};
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

#[cfg(test)]
#[path = "../../testing/effects.rs"]
mod effect_fixture;

#[cfg(test)]
#[path = "../../testing/catalogue.rs"]
mod catalogue_fixture;
