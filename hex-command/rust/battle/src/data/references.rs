use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NameKey(std::num::NonZeroU32);
#[derive(Clone, Debug, Default)]
pub struct Names {
    values: Vec<String>,
    index: HashMap<String, NameKey>,
}
impl Names {
    pub fn find(&self, value: &str) -> Option<NameKey> {
        self.index.get(value).copied()
    }
    pub fn intern(&mut self, value: String) -> NameKey {
        if let Some(key) = self.find(&value) {
            return key;
        }
        let key = NameKey(
            std::num::NonZeroU32::new(u32::try_from(self.values.len() + 1).expect("name limit"))
                .unwrap(),
        );
        self.index.insert(value.clone(), key);
        self.values.push(value);
        key
    }
    #[inline]
    pub fn get(&self, key: NameKey) -> &str {
        &self.values[key.0.get() as usize - 1]
    }
    pub fn resolve(&self, key: Option<NameKey>) -> Option<&str> {
        key.map(|key| self.get(key))
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InventoryId(pub usize);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ItemId(pub usize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnitKind(pub usize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileIndex(pub usize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WeaponKey(pub u32);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub len: usize,
}
impl Span {
    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }
}
#[derive(Clone, Copy, Debug)]
pub struct GroundItemState {
    pub tile: usize,
    pub item: ItemId,
}
