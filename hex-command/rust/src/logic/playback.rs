//! Numeric combat playback retained in battle-owned storage.
use crate::data::{Side, Unit};
use serde::Serialize;
#[derive(Debug)]
pub(crate) struct Playback {
    pub units: Option<[Unit; 2]>,
    pub strikes: Vec<Strike>,
    pub present: bool,
}
impl Playback {
    pub fn new(capacity: usize) -> Self {
        Self {
            units: None,
            strikes: Vec::with_capacity(capacity),
            present: false,
        }
    }
    pub fn clear(&mut self) {
        self.units = None;
        self.strikes.clear();
        self.present = false;
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Strike {
    #[serde(skip)]
    pub kind: usize,
    #[serde(skip)]
    pub weapon: usize,
    pub side: Side,
    pub roll_min: i32,
    pub roll_max: i32,
    pub raw: i32,
    pub modifier: i32,
    pub terrain: i32,
    pub damage: i32,
    pub effective: i32,
    pub drain: u8,
    pub healed: i32,
    pub thorns: i32,
    pub reflected: i32,
    pub health: [i32; 2],
}
