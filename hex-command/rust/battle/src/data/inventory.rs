//! Runtime equipment membership. Item mechanics stay exclusively in Catalogue definitions.
use super::{InventoryId, ItemId, Text};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Inventories {
    ids: Vec<Text>,
    index: HashMap<Text, ItemId>,
    rows: Vec<InventoryRow>,
}
#[derive(Clone, Debug)]
struct InventoryRow {
    items: Vec<ItemId>,
    identity: usize,
}
impl InventoryRow {
    fn new(items: Vec<ItemId>) -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        let identity = NEXT
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |id| id.checked_add(1),
            )
            .expect("inventory identity space");
        Self { items, identity }
    }
}
impl Default for Inventories {
    fn default() -> Self {
        Self {
            ids: Vec::new(),
            index: HashMap::new(),
            rows: vec![InventoryRow {
                items: Vec::new(),
                identity: 0,
            }],
        }
    }
}
impl Inventories {
    pub fn add_item(&mut self, id: Text) -> ItemId {
        if let Some(&key) = self.index.get(&id) {
            return key;
        }
        let key = ItemId(self.ids.len());
        self.index.insert(id.clone(), key);
        self.ids.push(id);
        key
    }
    pub fn load(&mut self, ids: Vec<Text>) -> InventoryId {
        if ids.is_empty() {
            return InventoryId(0);
        }
        let members = ids.into_iter().map(|id| self.add_item(id)).collect();
        let key = InventoryId(self.rows.len());
        self.rows.push(InventoryRow::new(members));
        key
    }
    /// Process-unique immutable row identity. Cloned forks retain old identities;
    /// edited/appended rows receive new identities, regardless of local row ID.
    #[inline]
    pub(crate) fn identity(&self, key: InventoryId) -> usize {
        self.rows[key.0].identity
    }
    pub fn item(&self, key: ItemId) -> &str {
        &self.ids[key.0]
    }
    pub fn item_ids(&self, key: InventoryId) -> &[ItemId] {
        &self.rows[key.0].items
    }
    pub fn items(&self, key: InventoryId) -> impl ExactSizeIterator<Item = &str> + Clone {
        self.rows[key.0].items.iter().map(|&key| self.item(key))
    }
    pub fn edit(
        &mut self,
        key: InventoryId,
        remove: Option<usize>,
        add: Option<ItemId>,
    ) -> InventoryId {
        let mut row = self.rows[key.0].items.clone();
        if let Some(slot) = remove {
            row.remove(slot);
        }
        if let Some(item) = add {
            row.push(item);
        }
        let key = InventoryId(self.rows.len());
        self.rows.push(InventoryRow::new(row));
        key
    }
}
