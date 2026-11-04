//! Existing Balanced recruitment behavior, isolated behind the Recruiter contract.
pub(super) struct Balanced;
impl super::Recruiter for Balanced {
    fn kind(&self) -> Option<super::RecruiterKind> {
        Some(super::RecruiterKind::Balanced)
    }
    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, crate::data::CommandError> {
        super::drive(super::RecruiterKind::Balanced, context, orders)
    }
}
