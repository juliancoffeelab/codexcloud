//! Ordering without cloning the sorting algorithm for every comparator and record type.
use std::cmp::Ordering;
#[cfg(target_arch = "wasm32")]
thread_local! {
    static INDICES: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(target_arch = "wasm32")]
#[inline(never)]
fn recycle(indices: Vec<usize>) {
    INDICES.with_borrow_mut(|cached| {
        if indices.capacity() > cached.capacity() {
            *cached = indices;
        }
    });
}

// WASM sorts indices through one non-generic kernel. Original ordinals resolve ties,
// preserving stable order. Only the small comparison thunk and record swaps depend
// on T; native builds keep the standard library's specialized implementations.
#[cfg(target_arch = "wasm32")]
#[inline(never)]
fn permutation(count: usize, compare: &mut dyn FnMut(usize, usize) -> Ordering) -> Vec<usize> {
    // Taking the buffer releases the RefCell borrow before invoking comparisons,
    // so a comparator can perform a nested sort safely.
    let mut indices = INDICES.with(std::cell::RefCell::take);
    indices.clear();
    indices.extend(0..count);
    indices.sort_unstable_by(|&a, &b| compare(a, b).then(a.cmp(&b)));
    indices
}

#[cfg(target_arch = "wasm32")]
fn apply<T>(values: &mut [T], mut compare: impl FnMut(&T, &T) -> Ordering) {
    if values.len() < 2 {
        return;
    }
    let mut sources = permutation(values.len(), &mut |a, b| compare(&values[a], &values[b]));
    apply_permutation(&mut sources, &mut Records(values));
    recycle(sources);
}

#[cfg(target_arch = "wasm32")]
trait SwapRecords {
    fn swap(&mut self, left: usize, right: usize);
}
#[cfg(target_arch = "wasm32")]
struct Records<'a, T>(&'a mut [T]);
#[cfg(target_arch = "wasm32")]
impl<T> SwapRecords for Records<'_, T> {
    fn swap(&mut self, left: usize, right: usize) {
        self.0.swap(left, right);
    }
}
#[cfg(target_arch = "wasm32")]
#[inline(never)]
fn apply_permutation(sources: &mut [usize], values: &mut dyn SwapRecords) {
    // Follow each destination-to-source cycle directly. Mark completed slots with
    // their own index, avoiding an inverse permutation and a second allocation.
    for start in 0..sources.len() {
        let mut slot = start;
        while sources[slot] != start {
            let next = sources[slot];
            values.swap(slot, next);
            sources[slot] = slot;
            slot = next;
        }
        sources[slot] = slot;
    }
}

pub(crate) trait Order<T> {
    fn order_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering);
    fn order_by_key<K: Ord>(&mut self, key: impl FnMut(&T) -> K);
    fn order_unstable_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering);
}
impl<T> Order<T> for [T] {
    fn order_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering) {
        #[cfg(target_arch = "wasm32")]
        apply(self, compare);
        #[cfg(not(target_arch = "wasm32"))]
        self.sort_by(compare);
    }
    fn order_by_key<K: Ord>(&mut self, key: impl FnMut(&T) -> K) {
        #[cfg(target_arch = "wasm32")]
        {
            let mut key = key;
            apply(self, |a, b| key(a).cmp(&key(b)));
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.sort_by_key(key);
    }
    fn order_unstable_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering) {
        #[cfg(target_arch = "wasm32")]
        apply(self, compare);
        #[cfg(not(target_arch = "wasm32"))]
        self.sort_unstable_by(compare);
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn prepare(tiles: usize) {
    INDICES.with_borrow_mut(|indices| {
        indices.reserve_exact(
            tiles
                .saturating_mul(tiles)
                .saturating_add(1)
                .saturating_sub(indices.len()),
        )
    });
}
