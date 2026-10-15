//! Native match execution, deterministic schedules, indexed journals and reports.
mod arrow;
mod canonical;
pub mod checkpoint;
pub mod cli;
pub mod error;
pub mod leader_audit;
pub mod matches;
pub mod records;
pub mod report;
pub mod schedule;
mod summaries;
pub mod types;
pub use error::SimulationError;
pub use matches::{run, run_arena, run_campaign};
pub use types::*;
