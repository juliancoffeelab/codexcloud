//! Existing Frontline recruitment behavior, isolated behind the Recruiter contract.
pub(super) struct Frontline;
impl super::Recruiter for Frontline {
    fn kind(&self) -> Option<super::RecruiterKind> {
        Some(super::RecruiterKind::Frontline)
    }
    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, crate::data::CommandError> {
        super::drive(super::RecruiterKind::Frontline, context, orders)
    }
}
