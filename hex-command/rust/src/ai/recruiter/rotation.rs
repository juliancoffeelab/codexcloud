//! Existing Rotation recruitment behavior, isolated behind the Recruiter contract.
pub(super) struct Rotation;
impl super::Recruiter for Rotation {
    fn recruit(
        &mut self,
        context: &mut crate::ai::context::RecruitmentContext<'_, '_>,
        orders: &crate::ai::orders::RecruitmentOrders,
    ) -> Result<crate::ai::orders::RecruitmentReport, crate::data::CommandError> {
        super::drive(super::RecruiterKind::Rotation, context, orders)
    }
}
