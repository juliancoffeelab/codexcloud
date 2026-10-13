//! Authored campaign reasoning is the only source of mission-specific orders.
use super::{context::CampaignView, orders::CampaignOrders};
pub(crate) trait Campaigner {
    fn objectives(&mut self, view: CampaignView<'_, '_>) -> CampaignOrders;
}

mod arena;
pub(in crate::ai) mod mission;

use serde::{Deserialize, Serialize};
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CampaignerKind {
    #[serde(alias = "arenaOnly")]
    Arena,
    #[serde(alias = "objectives", alias = "adaptive")]
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
