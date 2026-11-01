//! Reuse empty collection storage without replacing the global allocator.
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use std::cell::Cell;
#[cfg(target_arch = "wasm32")]
use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};
#[cfg(all(target_arch = "wasm32", feature = "allocation-profile"))]
#[derive(Default)]
struct Stats {
    leased: usize,
    peak: usize,
}
#[cfg(target_arch = "wasm32")]
thread_local! {static REGISTERING:Cell<bool>=const {Cell::new(false)};}
#[cfg(target_arch = "wasm32")]
pub(crate) struct Registration(bool);
#[cfg(target_arch = "wasm32")]
pub(crate) fn registration() -> Registration {
    Registration(REGISTERING.with(|flag| flag.replace(true)))
}
#[cfg(target_arch = "wasm32")]
impl Drop for Registration {
    fn drop(&mut self) {
        REGISTERING.with(|flag| flag.set(self.0));
    }
}
#[cfg(target_arch = "wasm32")]
mod buffers;
#[cfg(target_arch = "wasm32")]
use buffers::{adopt, give, register, sized, take};
#[cfg(target_arch = "wasm32")]
pub(crate) use buffers::{prepare, prepare_large, recycle_owned};
#[cfg(target_arch = "wasm32")]
pub(crate) fn prepare_registered(tiles: usize) {
    maps::prepare_registered(tiles);
    #[cfg(target_arch = "wasm32")]
    buffers::prepare_registered(tiles);
}
#[cfg(feature = "allocation-profile")]
pub(crate) fn report() -> std::vec::Vec<(&'static str, usize, usize, usize, usize)> {
    let mut rows = maps::report();
    #[cfg(target_arch = "wasm32")]
    rows.extend(buffers::report());
    rows
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn recycle_owned<T: 'static>(_values: std::vec::Vec<T>) {}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use std::vec::Vec;
#[cfg(target_arch = "wasm32")]
pub(crate) use vectors::Vec;
#[cfg(target_arch = "wasm32")]
mod vectors {
    use super::*;
    #[derive(Serialize)]
    #[serde(transparent)]
    pub struct Vec<T: 'static> {
        inner: std::vec::Vec<T>,
    }
    impl<T: 'static> Vec<T> {
        pub fn reserve(&mut self, additional: usize) {
            let capacity = self.inner.len().saturating_add(additional);
            if capacity <= self.inner.capacity() {
                return;
            }
            if self.inner.capacity() == 0 {
                self.inner = take();
            }
            if capacity <= self.inner.capacity() {
                return;
            }
            if let Some(mut replacement) = sized::<T>(capacity) {
                replacement.append(&mut self.inner);
                give(std::mem::replace(&mut self.inner, replacement));
            } else {
                self.inner.reserve(additional);
            }
        }
        pub fn push(&mut self, value: T) {
            if self.inner.len() == self.inner.capacity() {
                self.reserve(1);
            }
            self.inner.push(value);
        }
        pub fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
            let items = items.into_iter();
            self.reserve(items.size_hint().0);
            for item in items {
                self.push(item);
            }
        }
        pub fn new() -> Self {
            register::<T>();
            Self {
                inner: std::vec::Vec::new(),
            }
        }
        pub fn with_capacity(capacity: usize) -> Self {
            let mut this = Self::new();
            this.reserve(capacity);
            this
        }
        pub fn into_std(mut self) -> std::vec::Vec<T> {
            if self.inner.capacity() > 0 {
                give::<T>(std::vec::Vec::new());
            }
            std::mem::take(&mut self.inner)
        }
        pub fn resize_with(&mut self, len: usize, create: impl FnMut() -> T) {
            self.reserve(len.saturating_sub(self.len()));
            self.inner.resize_with(len, create);
        }
        pub fn append(&mut self, other: &mut std::vec::Vec<T>) {
            self.reserve(other.len());
            self.inner.append(other);
        }
        pub fn insert(&mut self, index: usize, value: T) {
            self.reserve(1);
            self.inner.insert(index, value);
        }
    }
    impl<'de, T: Deserialize<'de> + 'static> Deserialize<'de> for Vec<T> {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(std::vec::Vec::<T>::deserialize(deserializer)?.into())
        }
    }
    impl<T: 'static> Default for Vec<T> {
        fn default() -> Self {
            Self::new()
        }
    }
    impl<T: 'static> Drop for Vec<T> {
        fn drop(&mut self) {
            if self.inner.capacity() > 0 {
                give(std::mem::take(&mut self.inner));
            }
        }
    }
    impl<T: 'static> Deref for Vec<T> {
        type Target = std::vec::Vec<T>;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<T: 'static> DerefMut for Vec<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.inner
        }
    }
    impl<T: 'static> AsRef<[T]> for Vec<T> {
        fn as_ref(&self) -> &[T] {
            &self.inner
        }
    }
    impl<T: Clone + 'static> Vec<T> {
        pub fn resize(&mut self, len: usize, value: T) {
            self.reserve(len.saturating_sub(self.len()));
            self.inner.resize(len, value);
        }
        pub fn extend_from_slice(&mut self, source: &[T]) {
            self.reserve(source.len());
            self.inner.extend_from_slice(source);
        }
        pub fn clone_from<S: AsRef<[T]> + ?Sized>(&mut self, source: &S) {
            let source = source.as_ref();
            self.reserve(source.len().saturating_sub(self.inner.len()));
            for (to, from) in self.inner.iter_mut().zip(source) {
                to.clone_from(from);
            }
            if self.inner.len() < source.len() {
                self.inner.extend_from_slice(&source[self.inner.len()..]);
            }
            self.inner.truncate(source.len());
        }
    }
    impl<T: 'static> From<Vec<T>> for std::vec::Vec<T> {
        fn from(items: Vec<T>) -> Self {
            items.into_std()
        }
    }
    impl<T: 'static> From<Vec<T>> for VecDeque<T> {
        fn from(items: Vec<T>) -> Self {
            items.into_std().into()
        }
    }
    impl<T: Clone + 'static> Clone for Vec<T> {
        fn clone(&self) -> Self {
            let mut copy = Self::with_capacity(self.len());
            copy.inner.clone_from(&self.inner);
            copy
        }
        fn clone_from(&mut self, source: &Self) {
            Vec::clone_from(self, &source.inner)
        }
    }
    impl<T: std::fmt::Debug + 'static> std::fmt::Debug for Vec<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.inner.fmt(f)
        }
    }
    impl<T: PartialEq + 'static> PartialEq for Vec<T> {
        fn eq(&self, other: &Self) -> bool {
            self.inner == other.inner
        }
    }
    impl<T: Eq + 'static> Eq for Vec<T> {}
    impl<T: std::hash::Hash + 'static> std::hash::Hash for Vec<T> {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.inner.hash(state)
        }
    }
    impl<T: Ord + 'static> Ord for Vec<T> {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.inner.cmp(&other.inner)
        }
    }
    impl<T: PartialOrd + 'static> PartialOrd for Vec<T> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.inner.partial_cmp(&other.inner)
        }
    }
    impl<T: 'static> FromIterator<T> for Vec<T> {
        fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
            let mut this = Self::new();
            this.extend(items);
            this
        }
    }
    impl<T: 'static> Extend<T> for Vec<T> {
        fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
            Vec::extend(self, items)
        }
    }
    impl<T: 'static, const N: usize> From<[T; N]> for Vec<T> {
        fn from(items: [T; N]) -> Self {
            items.into_iter().collect()
        }
    }
    impl<T: 'static> From<std::vec::Vec<T>> for Vec<T> {
        fn from(inner: std::vec::Vec<T>) -> Self {
            let active = inner.capacity() > 0;
            if active {
                adopt::<T>();
            }
            Self { inner }
        }
    }
    impl<'a, T: 'static> IntoIterator for &'a Vec<T> {
        type Item = &'a T;
        type IntoIter = std::slice::Iter<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            self.inner.iter()
        }
    }
    impl<'a, T: 'static> IntoIterator for &'a mut Vec<T> {
        type Item = &'a mut T;
        type IntoIter = std::slice::IterMut<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            self.inner.iter_mut()
        }
    }
    impl<T: 'static> IntoIterator for Vec<T> {
        type Item = T;
        type IntoIter = IntoIter<T>;
        fn into_iter(mut self) -> Self::IntoIter {
            let mut inner = std::mem::take(&mut self.inner);
            let end = inner.len();
            // The iterator owns every initialized element in [start, end).
            // Keep Vec's length zero so consumed elements cannot be dropped twice.
            unsafe {
                inner.set_len(0);
            }
            IntoIter {
                inner,
                start: 0,
                end,
            }
        }
    }
    pub struct IntoIter<T: 'static> {
        inner: std::vec::Vec<T>,
        start: usize,
        end: usize,
    }
    impl<T: 'static> Iterator for IntoIter<T> {
        type Item = T;
        fn next(&mut self) -> Option<T> {
            if self.start == self.end {
                return None;
            }
            let index = self.start;
            self.start += 1;
            // This element is initialized, within the allocation, and is
            // removed from the iterator's remaining range before ownership moves.
            Some(unsafe { self.inner.as_ptr().add(index).read() })
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            let length = self.end - self.start;
            (length, Some(length))
        }
    }
    impl<T: 'static> DoubleEndedIterator for IntoIter<T> {
        fn next_back(&mut self) -> Option<T> {
            if self.start == self.end {
                return None;
            }
            self.end -= 1;
            // Same ownership invariant as next(), consuming the other end.
            Some(unsafe { self.inner.as_ptr().add(self.end).read() })
        }
    }
    impl<T: 'static> ExactSizeIterator for IntoIter<T> {}
    impl<T: 'static> std::iter::FusedIterator for IntoIter<T> {}
    impl<T: 'static> Drop for IntoIter<T> {
        fn drop(&mut self) {
            // Only the unconsumed contiguous range still contains live values.
            // The Vec remains empty while dropping nested reusable collections.
            unsafe {
                std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
                    self.inner.as_mut_ptr().add(self.start),
                    self.end - self.start,
                ));
            }
            if self.inner.capacity() > 0 {
                give(std::mem::take(&mut self.inner));
            }
        }
    }
    impl<T: PartialEq + 'static> PartialEq<std::vec::Vec<T>> for Vec<T> {
        fn eq(&self, other: &std::vec::Vec<T>) -> bool {
            self.inner == *other
        }
    }
    impl<T: PartialEq + 'static> PartialEq<Vec<T>> for std::vec::Vec<T> {
        fn eq(&self, other: &Vec<T>) -> bool {
            *self == other.inner
        }
    }
}

macro_rules! vec {
    ($value:expr; $count:expr)=>{{let mut result=$crate::reuse::Vec::with_capacity($count);result.resize($count,$value);result}};
    ($($value:expr),* $(,)?)=>{$crate::reuse::Vec::from([$($value),*])};
}
pub(crate) use vec;

mod maps;
#[cfg(target_arch = "wasm32")]
pub(crate) use maps::prepare as prepare_maps;
pub(crate) use maps::{Map, Set};

/// A bounded scoped table can retain borrowed values without putting them in TLS.
pub(crate) struct ScopedMap<K, V, const N: usize> {
    entries: smallvec::SmallVec<[(K, V); N]>,
}
impl<K, V, const N: usize> Default for ScopedMap<K, V, N> {
    fn default() -> Self {
        Self {
            entries: smallvec::SmallVec::new(),
        }
    }
}
impl<K: Eq, V, const N: usize> ScopedMap<K, V, N> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn retain(&mut self, mut keep: impl FnMut(&K) -> bool) {
        self.entries.retain(|(key, _)| keep(key));
    }
    pub fn entry(&mut self, key: K) -> ScopedEntry<'_, K, V, N> {
        let index = self.entries.iter().position(|(k, _)| k == &key);
        ScopedEntry {
            entries: &mut self.entries,
            key,
            index,
        }
    }
}
pub(crate) struct ScopedEntry<'a, K, V, const N: usize> {
    entries: &'a mut smallvec::SmallVec<[(K, V); N]>,
    key: K,
    index: Option<usize>,
}
impl<'a, K, V, const N: usize> ScopedEntry<'a, K, V, N> {
    pub fn or_insert_with(self, value: impl FnOnce() -> V) -> &'a mut V {
        let index = if let Some(index) = self.index {
            index
        } else {
            let index = self.entries.len();
            self.entries.push((self.key, value()));
            index
        };
        &mut self.entries[index].1
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Local<T> = smallvec::SmallVec<[T; 16]>;
#[cfg(target_arch = "wasm32")]
pub(crate) type Local<T> = smallvec::SmallVec<[T; 128]>;

/// Fixed-capacity FIFO for searches with an explicit candidate budget.
#[cfg(target_arch = "wasm32")]
pub(crate) struct Queue<T, const N: usize> {
    slots: [Option<T>; N],
    head: usize,
    len: usize,
}
#[cfg(target_arch = "wasm32")]
impl<T, const N: usize> Queue<T, N> {
    pub fn push_back(&mut self, value: T) {
        assert!(self.len < N, "search queue exceeded its candidate budget");
        self.slots[(self.head + self.len) % N] = Some(value);
        self.len += 1;
    }
    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.slots[self.head].take();
        self.head = (self.head + 1) % N;
        self.len -= 1;
        value
    }
}
#[cfg(target_arch = "wasm32")]
impl<T, const N: usize, const M: usize> From<[T; M]> for Queue<T, N> {
    fn from(values: [T; M]) -> Self {
        let mut queue = Self {
            slots: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
        };
        for value in values {
            queue.push_back(value);
        }
        queue
    }
}
