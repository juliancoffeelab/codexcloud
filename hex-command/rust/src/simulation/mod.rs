//! Native match execution, deterministic schedules, indexed journals and reports.
mod arrow;
mod canonical;
pub mod checkpoint;
pub mod cli;
mod damage;
pub mod error;
pub mod leader_audit;
pub mod matches;
mod objectives;
pub mod records;
pub mod report;
pub mod schedule;
mod summaries;
pub mod types;
pub use error::SimulationError;
pub use matches::{run, run_arena, run_campaign};
pub use types::*;
