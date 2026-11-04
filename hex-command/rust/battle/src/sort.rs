//! Share the browser sorting kernel while retaining native standard-library sorting.
use std::cmp::Ordering;
#[cfg(target_arch = "wasm32")]
thread_local! { static SCRATCH:std::cell::RefCell<Vec<usize>>=const{std::cell::RefCell::new(Vec::new())}; }
#[cfg(target_arch = "wasm32")]
#[inline(never)]
fn permutation(
    indices: &mut Vec<usize>,
    count: usize,
    compare: &mut dyn FnMut(usize, usize) -> Ordering,
) {
    indices.clear();
    indices.extend(0..count);
    indices.sort_unstable_by(|&a, &b| compare(a, b).then(a.cmp(&b)));
}
#[cfg(target_arch = "wasm32")]
fn apply<T>(values: &mut [T], mut compare: impl FnMut(&T, &T) -> Ordering) {
    if values.len() < 2 {
        return;
    }
    // Release the borrow before comparing: a nested sort can use its own buffer.
    let mut indices = SCRATCH.with(|scratch| std::mem::take(&mut *scratch.borrow_mut()));
    permutation(&mut indices, values.len(), &mut |a, b| {
        compare(&values[a], &values[b])
    });
    for start in 0..indices.len() {
        let mut slot = start;
        while indices[slot] != start {
            let next = indices[slot];
            values.swap(slot, next);
            indices[slot] = slot;
            slot = next;
        }
        indices[slot] = slot;
    }
    indices.clear();
    SCRATCH.with(|scratch| {
        let mut scratch = scratch.borrow_mut();
        if indices.capacity() > scratch.capacity() {
            *scratch = indices;
        }
    });
}
pub trait SharedSort<T> {
    fn order_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering);
    fn order_by_key<K: Ord>(&mut self, key: impl FnMut(&T) -> K);
    fn order_unstable_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering);
}
impl<T> SharedSort<T> for [T] {
    #[inline]
    fn order_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering) {
        #[cfg(target_arch = "wasm32")]
        apply(self, compare);
        #[cfg(not(target_arch = "wasm32"))]
        self.sort_by(compare);
    }
    #[inline]
    fn order_by_key<K: Ord>(&mut self, key: impl FnMut(&T) -> K) {
        #[cfg(target_arch = "wasm32")]
        {
            let mut key = key;
            apply(self, |a, b| key(a).cmp(&key(b)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.sort_by_key(key);
    }
    #[inline]
    fn order_unstable_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering) {
        #[cfg(target_arch = "wasm32")]
        apply(self, compare);
        #[cfg(not(target_arch = "wasm32"))]
        self.sort_unstable_by(compare);
    }
}
