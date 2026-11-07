//! Shared game data, deterministic battle rules, AI policies and native simulations.
//!
//! Catalogs are authored as JSON. The host owns a catalog and lends it to game
//! operations; each battle and planning workspace is exclusively owned.

pub mod ai;
pub mod data;
pub mod logic;
pub mod protocol;
#[cfg(not(target_arch = "wasm32"))]
pub mod simulation;
#[cfg(feature = "wasm")]
mod undo;
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(test)]
mod tests;

mod sort;
