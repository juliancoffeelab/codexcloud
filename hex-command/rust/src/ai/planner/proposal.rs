//! A tactical action proposal, independent of the selecting algorithm.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::ai) struct Plan {
    pub(in crate::ai) unit: u32,
    pub(in crate::ai) tile: usize,
    pub(in crate::ai) score: f64,
    pub(in crate::ai) target: Option<u32>,
    pub(in crate::ai) weapon: Option<usize>,
    pub(in crate::ai) advance: Option<usize>,
    pub(in crate::ai) withdrawing: bool,
}
