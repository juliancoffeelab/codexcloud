//! Courier/captive travel incentives shared by Full and Queued.
use crate::{ai::orders::TacticalOrders, data::*};

pub(in crate::ai) const EXPOSURE: f64 = 20.0;

pub(in crate::ai) fn subject(orders: &TacticalOrders, unit: &Unit) -> bool {
    orders.campaign && unit.role.is_some_and(Role::is_escort)
}

/// At most five points for closing the remaining route and ten for arrival.
/// Leaving the endpoint reverses both terms; standing there pays nothing again.
pub(in crate::ai) fn progress(before: u16, after: u16) -> f64 {
    5.0 * ((before as f64 - after as f64) / before.max(1) as f64).clamp(-1.0, 1.0)
        + 10.0 * ((after == 0) as i32 - (before == 0) as i32) as f64
}
