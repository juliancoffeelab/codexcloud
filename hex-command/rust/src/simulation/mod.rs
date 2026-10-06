//! Native match execution, deterministic schedules, indexed journals and reports.
pub mod cli;
pub mod compare;
pub mod error;
pub mod matches;
pub mod records;
pub mod report;
pub mod schedule;
pub mod types;
pub use error::SimulationError;
pub use matches::{run, run_arena, run_campaign};
pub use types::*;
