//! Native builds use standard collections; WASM retains empty table storage.
#[cfg(target_arch = "wasm32")]
#[path = "maps_pool.rs"]
mod platform;
#[cfg(target_arch = "wasm32")]
pub(super) use platform::prepare_registered;
#[cfg(all(target_arch = "wasm32", feature = "allocation-profile"))]
pub(super) use platform::report;
#[cfg(target_arch = "wasm32")]
pub(crate) use platform::{Map, prepare};
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use rustc_hash::{FxHashMap as Map, FxHashSet as Set};
#[cfg(all(not(target_arch = "wasm32"), feature = "allocation-profile"))]
pub(super) fn report() -> std::vec::Vec<(&'static str, usize, usize, usize, usize)> {
    std::vec::Vec::new()
}
#[cfg(target_arch = "wasm32")]
pub(crate) use sets::Set;
#[cfg(target_arch = "wasm32")]
mod sets {
    use super::Map;
    use std::{borrow::Borrow, hash::Hash};
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Set<K: Eq + Hash + 'static> {
        map: Map<K, ()>,
    }
    impl<K: Eq + Hash + 'static> Set<K> {
        pub fn new() -> Self {
            Self { map: Map::new() }
        }
        pub fn insert(&mut self, key: K) -> bool {
            self.map.insert(key, ()).is_none()
        }
        pub fn contains<Q: Eq + Hash + ?Sized>(&self, key: &Q) -> bool
        where
            K: Borrow<Q>,
        {
            self.map.contains_key(key)
        }
        pub fn remove<Q: Eq + Hash + ?Sized>(&mut self, key: &Q) -> bool
        where
            K: Borrow<Q>,
        {
            self.map.remove(key).is_some()
        }
        pub fn len(&self) -> usize {
            self.map.len()
        }
        pub fn is_empty(&self) -> bool {
            self.map.is_empty()
        }
        pub fn clear(&mut self) {
            self.map.clear();
        }
        pub fn iter(&self) -> std::collections::hash_map::Keys<'_, K, ()> {
            self.map.keys()
        }
        pub fn extend<I: IntoIterator<Item = K>>(&mut self, items: I) {
            let items = items.into_iter();
            self.map.reserve(items.size_hint().0);
            for key in items {
                self.insert(key);
            }
        }
    }
    impl<K: Eq + Hash + 'static> Default for Set<K> {
        fn default() -> Self {
            Self::new()
        }
    }
    impl<K: Eq + Hash + 'static> FromIterator<K> for Set<K> {
        fn from_iter<I: IntoIterator<Item = K>>(items: I) -> Self {
            let mut set = Self::new();
            set.extend(items);
            set
        }
    }
    impl<K: Eq + Hash + 'static> Extend<K> for Set<K> {
        fn extend<I: IntoIterator<Item = K>>(&mut self, items: I) {
            Set::extend(self, items)
        }
    }
    impl<'a, K: Eq + Hash + 'static> IntoIterator for &'a Set<K> {
        type Item = &'a K;
        type IntoIter = std::collections::hash_map::Keys<'a, K, ()>;
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }
}
