use super::*;
use crate::CampaignProgram;
use crate::logic::campaign_program::ChapterProgram;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
/// ChapterProgram result is independent of which controller's phase delivered it.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BattleResult {
    PlayerVictory,
    PlayerDefeat,
}
/// An owned unit snapshot preserving stable IDs and optional campaign metadata.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUnit {
    pub revealed_to: Vec<Text>,
    pub perception_effects: Vec<PerceptionEffect>,
    pub faction: Text,
    pub items: Vec<Text>,
    pub ai_group: Option<Text>,
    pub id: Text,
    pub r#type: Text,
    pub tile: Hex,
    pub hp: i32,
    pub moved: bool,
    pub attack_spent: bool,
    pub experience: Option<u32>,
    pub character: Option<Text>,
    pub role: Option<Role>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
}
/// Compact live facts. Names and inventories are numeric references into battle tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Unit {
    pub field_available: bool,
    pub perception: PerceptionId,
    pub id: u32,
    pub kind: usize,
    /// Derived from the owning faction; never serialized independently.
    pub phase: TurnPhase,
    pub tile: usize,
    pub hp: i32,
    pub moved: bool,
    pub attack_spent: bool,
    pub experience: Option<u32>,
    pub role: Option<Role>,
    pub retreating: Option<bool>,
    pub rebuilding: Option<bool>,
    pub faction: NameKey,
    /// Derived from faction membership, never serialized independently.
    pub vision_group: VisionGroupKey,
    pub ai_group: Option<NameKey>,
    pub character: Option<NameKey>,
    pub inventory: InventoryId,
}
impl Unit {
    #[inline]
    pub fn is_leader(&self) -> bool {
        self.role == Some(Role::Leader)
    }
}
/// Feedback identifying the latest attack until the next turn clears it.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastCombat {
    pub attacker_id: Text,
    pub attack_id: Text,
    pub sound: Text,
}
/// A surviving unit carried between campaign chapters with its earned experience.
#[cfg_attr(feature = "wasm", ocamljsify::ocamljsify(borrow))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Veteran {
    pub items: Vec<Text>,
    pub r#type: Text,
    pub experience: u32,
    pub character: Option<Text>,
    pub role: Option<Role>,
}
