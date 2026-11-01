//! Pool bookkeeping is shared by every key/value type. Boxes retain table
//! metadata as well as buckets, so returning a table needs no allocation.
use super::super::REGISTERING;
#[cfg(feature = "allocation-profile")]
use super::super::Stats;
use rustc_hash::FxHashMap;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    hash::Hash,
    ops::{Deref, DerefMut},
};
#[derive(Clone, Copy)]
struct Kind {
    id: TypeId,
    make: fn(usize) -> Box<dyn Any>,
    capacity: fn(&dyn Any) -> usize,
    set: bool,
    #[cfg(feature = "allocation-profile")]
    name: fn() -> &'static str,
    #[cfg(feature = "allocation-profile")]
    size: usize,
}
fn make<K: Eq + Hash + 'static, V: 'static>(capacity: usize) -> Box<dyn Any> {
    Box::new(FxHashMap::<K, V>::with_capacity_and_hasher(
        capacity,
        Default::default(),
    ))
}
fn capacity<K: Eq + Hash + 'static, V: 'static>(table: &dyn Any) -> usize {
    table
        .downcast_ref::<FxHashMap<K, V>>()
        .expect("typed map pool")
        .capacity()
}
struct Description<K, V>(std::marker::PhantomData<(K, V)>);
impl<K: Eq + Hash + 'static, V: 'static> Description<K, V> {
    const KIND: Kind = Kind {
        capacity: capacity::<K, V>,
        id: TypeId::of::<FxHashMap<K, V>>(),
        make: make::<K, V>,
        set: std::mem::size_of::<V>() == 0,
        #[cfg(feature = "allocation-profile")]
        name: std::any::type_name::<FxHashMap<K, V>>,
        #[cfg(feature = "allocation-profile")]
        size: std::mem::size_of::<(K, V)>(),
    };
}
#[inline(always)]
fn kind<K: Eq + Hash + 'static, V: 'static>() -> &'static Kind {
    &Description::<K, V>::KIND
}

struct Bucket {
    capacity: usize,
    limit: usize,
    maps: Vec<Box<dyn Any>>,
}
struct Pool {
    kind: &'static Kind,
    buckets: Vec<Bucket>,
    limit: Option<(usize, usize)>,
    #[cfg(feature = "allocation-profile")]
    stats: Stats,
}
impl Pool {
    fn new(kind: &'static Kind) -> Self {
        Self {
            kind,
            buckets: Vec::new(),
            limit: None,
            #[cfg(feature = "allocation-profile")]
            stats: Stats::default(),
        }
    }
    fn reserve(&mut self, tiles: usize) {
        let limit = self.limit.map_or(tiles, |v| v.0);
        let mut wanted = 1;
        loop {
            let capacity = if wanted == 1 { 3 } else { wanted / 4 * 7 };
            let count = self.limit.map_or_else(
                || {
                    if self.kind.set && wanted < 16 {
                        16
                    } else if wanted > tiles / 4 {
                        1
                    } else {
                        2
                    }
                },
                |v| v.1,
            );
            let index = match self.buckets.binary_search_by_key(&capacity, |b| b.capacity) {
                Ok(i) => i,
                Err(i) => {
                    self.buckets.insert(
                        i,
                        Bucket {
                            capacity,
                            limit: count,
                            maps: Vec::new(),
                        },
                    );
                    i
                }
            };
            let bucket = &mut self.buckets[index];
            bucket.limit = bucket.limit.max(count);
            while bucket.maps.len() < count {
                bucket.maps.push((self.kind.make)(wanted));
            }
            if capacity >= limit {
                break;
            }
            wanted = if wanted == 1 { 4 } else { wanted * 2 };
        }
    }
}
thread_local! {static TABLES:RefCell<FxHashMap<TypeId,Pool>>=RefCell::new(FxHashMap::default());}
#[inline(never)]
fn register(kind: &'static Kind) {
    if REGISTERING.with(std::cell::Cell::get) {
        TABLES.with_borrow_mut(|pools| {
            pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        });
    }
}
#[inline(never)]
fn take(kind: &'static Kind, capacity: usize) -> Box<dyn Any> {
    TABLES.with_borrow_mut(|pools| {
        let pool = pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        #[cfg(feature = "allocation-profile")]
        {
            pool.stats.leased += 1;
            pool.stats.peak = pool.stats.peak.max(pool.stats.leased);
        }
        pool.buckets
            .iter_mut()
            .find(|b| b.capacity >= capacity && !b.maps.is_empty())
            .and_then(|b| b.maps.pop())
            .unwrap_or_else(|| (kind.make)(capacity))
    })
}
#[inline(never)]
fn give(kind: &'static Kind, map: Box<dyn Any>) {
    // The caller has cleared the typed table. Bucket identity is its capacity;
    // capacity is supplied without a type-dependent pool-management kernel.
    return_table(kind, map);
}
// Store capacity alongside the erased box using its registered type's accessor.
// The accessor is a tiny monomorphic thunk; all inventory traversal stays shared.
#[inline(never)]
fn return_table(kind: &'static Kind, map: Box<dyn Any>) {
    let _ = TABLES.try_with(|pools| {
        if let Ok(mut pools) = pools.try_borrow_mut() {
            if let Some(pool) = pools.get_mut(&kind.id) {
                #[cfg(feature = "allocation-profile")]
                {
                    pool.stats.leased = pool.stats.leased.saturating_sub(1);
                }
                let capacity = (kind.capacity)(map.as_ref());
                if let Some(bucket) = pool.buckets.iter_mut().find(|b| b.capacity == capacity) {
                    if bucket.maps.len() < bucket.limit {
                        bucket.maps.push(map);
                    }
                }
            }
        }
    });
}
pub(crate) fn prepare<K: Eq + Hash + 'static, V: 'static>(limit: usize, buffers: usize) {
    prepare_kind(kind::<K, V>(), limit, buffers);
}
#[inline(never)]
fn prepare_kind(kind: &'static Kind, limit: usize, buffers: usize) {
    TABLES.with_borrow_mut(|pools| {
        let pool = pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        pool.limit = Some((pool.limit.map_or(limit, |v| v.0.max(limit)), buffers));
    });
}
pub(crate) fn prepare_registered(tiles: usize) {
    TABLES.with_borrow_mut(|pools| {
        for pool in pools.values_mut() {
            pool.reserve(tiles);
        }
    });
}
#[cfg(feature = "allocation-profile")]
pub(crate) fn report() -> Vec<(&'static str, usize, usize, usize, usize)> {
    TABLES.with_borrow(|pools| {
        pools
            .values()
            .map(|p| {
                (
                    (p.kind.name)(),
                    p.kind.size,
                    p.buckets
                        .iter()
                        .map(|b| b.maps.len() * b.capacity * (p.kind.size + 1))
                        .sum(),
                    p.stats.peak,
                    p.buckets.iter().map(|b| b.maps.len()).sum(),
                )
            })
            .collect()
    })
}

pub struct Map<K: Eq + Hash + 'static, V: 'static> {
    inner: Option<Box<FxHashMap<K, V>>>,
}
impl<K: Eq + Hash + 'static, V: 'static> Map<K, V> {
    const EMPTY: &'static FxHashMap<K, V> = &FxHashMap::with_hasher(rustc_hash::FxBuildHasher);
    pub fn new() -> Self {
        register(kind::<K, V>());
        Self { inner: None }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        let mut this = Self::new();
        this.reserve(capacity);
        this
    }
    pub fn reserve(&mut self, additional: usize) {
        let wanted = self.len().saturating_add(additional);
        if wanted <= self.capacity() {
            return;
        }
        let mut next = take(kind::<K, V>(), wanted)
            .downcast::<FxHashMap<K, V>>()
            .expect("typed map pool");
        if next.capacity() < wanted {
            next.reserve(wanted);
        }
        if let Some(old) = &mut self.inner {
            next.extend(old.drain());
        }
        if let Some(old) = self.inner.replace(next) {
            give(kind::<K, V>(), old);
        }
    }
    fn room(&mut self, _key: &K) {
        // HashMap can grow a full table even when replacing an existing key.
        if self.len() == self.capacity() {
            self.reserve(1);
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.room(&key);
        self.deref_mut().insert(key, value)
    }
    pub fn entry(&mut self, key: K) -> std::collections::hash_map::Entry<'_, K, V> {
        self.room(&key);
        self.deref_mut().entry(key)
    }
    pub fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, items: I) {
        let items = items.into_iter();
        self.reserve(items.size_hint().0);
        for (key, value) in items {
            self.insert(key, value);
        }
    }
}
impl<K: Eq + Hash + 'static, V: 'static> Default for Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: Eq + Hash + 'static, V: 'static> Drop for Map<K, V> {
    fn drop(&mut self) {
        if let Some(mut map) = self.inner.take() {
            // Drain resets tombstones even when the final entry was removed.
            // clear() skips empty tables, leaving a reduced reported capacity.
            drop(map.drain());
            give(kind::<K, V>(), map);
        }
    }
}
impl<K: Eq + Hash + 'static, V: 'static> Deref for Map<K, V> {
    type Target = FxHashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        self.inner.as_deref().unwrap_or(Self::EMPTY)
    }
}
impl<K: Eq + Hash + 'static, V: 'static> DerefMut for Map<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.inner.is_none() {
            self.reserve(1);
        }
        self.inner.as_deref_mut().expect("leased mutable table")
    }
}
impl<K: Eq + Hash + Clone + 'static, V: Clone + 'static> Clone for Map<K, V> {
    fn clone(&self) -> Self {
        if self.capacity() == 0 {
            return Self::new();
        }
        let mut copy = Self::with_capacity(self.capacity());
        if copy.capacity() == self.capacity() {
            copy.deref_mut().clone_from(self.deref());
        } else {
            copy.extend(self.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        copy
    }
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}
impl<K: Eq + Hash + std::fmt::Debug + 'static, V: std::fmt::Debug + 'static> std::fmt::Debug
    for Map<K, V>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
impl<K: Eq + Hash + 'static, V: PartialEq + 'static> PartialEq for Map<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<K: Eq + Hash + 'static, V: Eq + 'static> Eq for Map<K, V> {}
impl<K: Eq + Hash + 'static, V: 'static> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(items: I) -> Self {
        let items = items.into_iter();
        let mut map = Self::with_capacity(items.size_hint().0);
        map.extend(items);
        map
    }
}
impl<K: Eq + Hash + 'static, V: 'static> Extend<(K, V)> for Map<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, items: I) {
        Map::extend(self, items)
    }
}
impl<'a, K: Eq + Hash + 'static, V: 'static> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.deref().iter()
    }
}
impl<'a, K: Eq + Hash + 'static, V: 'static> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = std::collections::hash_map::IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().iter_mut()
    }
}
