//! Authored campaign reasoning is the only source of mission-specific orders.
use super::{context::CampaignView, orders::CampaignOrders};
pub(crate) trait Campaigner {
    fn objectives(&mut self, view: CampaignView<'_, '_>) -> CampaignOrders;
}

mod arena;
pub(in crate::ai) mod mission;

use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampaignerKind {
    Arena,
    Mission,
}
impl Campaigner for CampaignerKind {
    fn objectives(&mut self, view: CampaignView<'_, '_>) -> CampaignOrders {
        match self {
            Self::Arena => arena::Arena.objectives(view),
            Self::Mission => mission::Mission.objectives(view),
        }
    }
}

/// Authored campaign goals for host diagnostics and route evaluation.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn authored_goals(
    authored: hex_command_battle::CampaignView<'_, '_>,
    phases: hex_command_battle::PhaseForecasts<'_, '_>,
    side: crate::data::Side,
) -> Vec<crate::data::Objective> {
    mission::MissionPolicy::new(authored, phases).objectives(side)
}
