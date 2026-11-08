//! Courier/captive travel incentives shared by Full and Queued.
use crate::{ai::orders::TacticalOrders, data::*};

pub(in crate::ai) const EXPOSURE: f64 = 20.0;

pub(in crate::ai) fn subject(orders: &TacticalOrders, unit: &Unit) -> bool {
    orders.campaign && unit.role.is_some_and(Role::is_escort)
}

/// At most five points for closing the remaining route and ten for arrival.
/// Arrival uses endpoint membership: zero-cost terrain can also have zero route cost.
/// Leaving reverses the arrival bonus; standing there pays nothing again.
pub(in crate::ai) fn progress(before: u16, after: u16, at_before: bool, at_after: bool) -> f64 {
    5.0 * ((before as f64 - after as f64) / before.max(1) as f64).clamp(-1.0, 1.0)
        + 10.0 * (at_after as i32 - at_before as i32) as f64
}
