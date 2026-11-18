//! Authored campaign reasoning is the only source of mission-specific orders.
use super::{context::CampaignerContext, orders::CampaignOrders};
pub(crate) trait Campaigner {
    fn objectives(&mut self, view: CampaignerContext<'_, '_>) -> CampaignOrders;
}

mod arena;
pub(in crate::ai) mod mission;
mod patrol;

use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampaignerKind {
    Arena,
    Mission,
}
impl Campaigner for CampaignerKind {
    fn objectives(&mut self, view: CampaignerContext<'_, '_>) -> CampaignOrders {
        match self {
            Self::Arena => arena::Arena.objectives(view),
            Self::Mission => mission::MissionCampaigner.objectives(view),
        }
    }
}

/// Authored campaign goals for host diagnostics and route evaluation.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn authored_goals<'a, 'catalog>(
    authored: hex_command_battle::CampaignObservation<'a, 'catalog>,
    phases: hex_command_battle::PhaseForecasts<'a, 'catalog>,
) -> Vec<crate::data::Objective> {
    mission::MissionPolicy::new(authored, phases).objectives()
}
