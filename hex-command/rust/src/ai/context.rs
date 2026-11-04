//! Read-only battle observations and named runtime protocol types.
//! Mutable contexts have private constructors under runtime.
pub use hex_command_battle::{BattleView, BattleRules, PhaseForecasts, ActionForecast, ActionForecastSource};
pub(crate) use super::runtime::context::{CampaignView, RecruitmentContext, TacticalContext};
pub(in crate::ai) type BattleDraft<'a,'catalog> = ActionForecast<'catalog>;
