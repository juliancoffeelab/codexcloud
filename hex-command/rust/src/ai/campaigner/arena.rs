//! Arena orders contain ordinary combat work without authored mission duties.

pub(super) struct Arena;
impl super::Campaigner for Arena {
    fn objectives(
        &mut self,
        view: crate::ai::context::CampaignView<'_, '_>,
    ) -> crate::ai::orders::CampaignOrders {
        view.orders(super::CampaignerKind::Arena)
    }
}
