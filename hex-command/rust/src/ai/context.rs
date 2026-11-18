//! Read-only battle observations and named runtime protocol types.
//! Mutable contexts have private constructors under runtime.
pub(crate) use super::runtime::context::{CampaignerContext, RecruitmentContext, TacticalContext};
pub use hex_command_battle::{ActionForecast, ActionForecastSource, BattleView, PhaseForecasts};
