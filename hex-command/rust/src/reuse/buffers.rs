//! Element-independent management of empty vector allocations.
//! The typed boundary drops every value before transferring ownership here.
use super::REGISTERING;
#[cfg(feature = "allocation-profile")]
use super::Stats;
use rustc_hash::FxHashMap;
use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    any::TypeId,
    cell::RefCell,
    mem::ManuallyDrop,
    ptr::NonNull,
};

struct Buffer {
    pointer: NonNull<u8>,
    capacity: usize,
    layout: Layout,
}
impl Buffer {
    fn new(element: Layout, capacity: usize) -> Self {
        let layout = Layout::from_size_align(
            element
                .size()
                .checked_mul(capacity)
                .expect("vector capacity overflow"),
            element.align(),
        )
        .expect("vector layout");
        // Both dimensions come from Layout::new::<T>(); zero-sized allocations
        // never enter this module. Allocation uses the existing global allocator.
        let pointer =
            NonNull::new(unsafe { alloc(layout) }).unwrap_or_else(|| handle_alloc_error(layout));
        Self {
            pointer,
            capacity,
            layout,
        }
    }
    fn from_vec<T>(mut values: std::vec::Vec<T>) -> Option<Self> {
        values.clear();
        if values.capacity() == 0 || std::mem::size_of::<T>() == 0 {
            return None;
        }
        let mut values = ManuallyDrop::new(values);
        Some(Self {
            pointer: NonNull::new(values.as_mut_ptr().cast()).expect("vector pointer"),
            capacity: values.capacity(),
            layout: Layout::array::<T>(values.capacity()).expect("vector layout"),
        })
    }
    fn into_vec<T>(self) -> std::vec::Vec<T> {
        debug_assert_eq!(self.layout, Layout::array::<T>(self.capacity).unwrap());
        let this = ManuallyDrop::new(self);
        // Pools are keyed by TypeId. This allocation originally held T, or was
        // allocated with its exact layout. No initialized elements are retained.
        unsafe { std::vec::Vec::from_raw_parts(this.pointer.as_ptr().cast(), 0, this.capacity) }
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { dealloc(self.pointer.as_ptr(), self.layout) }
    }
}

#[derive(Clone, Copy)]
struct Kind {
    id: TypeId,
    element: Layout,
    #[cfg(feature = "allocation-profile")]
    name: fn() -> &'static str,
}
struct Description<T>(std::marker::PhantomData<T>);
impl<T: 'static> Description<T> {
    const KIND: Kind = Kind {
        id: TypeId::of::<T>(),
        element: Layout::new::<T>(),
        #[cfg(feature = "allocation-profile")]
        name: std::any::type_name::<T>,
    };
}
#[inline(always)]
fn kind<T: 'static>() -> &'static Kind {
    &Description::<T>::KIND
}

struct Pool {
    kind: &'static Kind,
    free: std::vec::Vec<Buffer>,
    baseline: usize,
    fixed: bool,
    limit: usize,
    #[cfg(feature = "allocation-profile")]
    stats: Stats,
}
impl Pool {
    fn new(kind: &'static Kind) -> Self {
        Self {
            kind,
            free: std::vec::Vec::with_capacity(8),
            baseline: 0,
            fixed: false,
            limit: 8,
            #[cfg(feature = "allocation-profile")]
            stats: Stats::default(),
        }
    }
    fn prepare(&mut self, buffers: usize, elements: usize) {
        self.baseline = self.baseline.max(elements);
        self.limit = self.limit.max(buffers);
        self.free
            .reserve(self.limit.saturating_sub(self.free.len()));
        for buffer in &mut self.free {
            if buffer.capacity < elements {
                *buffer = Buffer::new(self.kind.element, elements);
            }
        }
        while self.free.len() < buffers {
            self.free.push(Buffer::new(self.kind.element, elements));
        }
    }
    fn lease(&mut self, index: usize) -> Buffer {
        #[cfg(feature = "allocation-profile")]
        {
            self.stats.leased += 1;
            self.stats.peak = self.stats.peak.max(self.stats.leased);
        }
        self.free.swap_remove(index)
    }
}
thread_local! {static VECTORS:RefCell<FxHashMap<TypeId,Pool>>=RefCell::new(FxHashMap::default());}
#[inline(never)]
fn register_kind(kind: &'static Kind) {
    if kind.element.size() != 0 && REGISTERING.with(std::cell::Cell::get) {
        VECTORS.with_borrow_mut(|pools| {
            pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        });
    }
}
pub(super) fn register<T: 'static>() {
    register_kind(kind::<T>());
}
#[inline(never)]
fn take_buffer(kind: &'static Kind, capacity: Option<usize>) -> Option<Buffer> {
    if kind.element.size() == 0 {
        return None;
    }
    VECTORS.with_borrow_mut(|pools| {
        let pool = pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        let index = if let Some(capacity) = capacity {
            pool.free.iter().position(|b| b.capacity >= capacity)
        } else {
            pool.free
                .iter()
                .rposition(|b| b.capacity <= pool.baseline.saturating_mul(2))
                .or_else(|| pool.free.len().checked_sub(1))
        };
        match index {
            Some(index) => Some(pool.lease(index)),
            None => {
                #[cfg(feature = "allocation-profile")]
                if capacity.is_none() {
                    pool.stats.leased += 1;
                    pool.stats.peak = pool.stats.peak.max(pool.stats.leased);
                }
                None
            }
        }
    })
}
pub(super) fn take<T: 'static>() -> std::vec::Vec<T> {
    take_buffer(kind::<T>(), None).map_or_else(std::vec::Vec::new, Buffer::into_vec)
}
pub(super) fn sized<T: 'static>(capacity: usize) -> Option<std::vec::Vec<T>> {
    take_buffer(kind::<T>(), Some(capacity)).map(Buffer::into_vec)
}
#[inline(never)]
fn adopt_kind(kind: &'static Kind) {
    if kind.element.size() == 0 {
        return;
    }
    VECTORS.with_borrow_mut(|pools| {
        let pool = pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        #[cfg(feature = "allocation-profile")]
        {
            pool.stats.leased += 1;
            pool.stats.peak = pool.stats.peak.max(pool.stats.leased);
        }
        #[cfg(not(feature = "allocation-profile"))]
        let _ = pool;
    });
}
pub(super) fn adopt<T: 'static>() {
    adopt_kind(kind::<T>());
}
#[inline(never)]
fn return_buffer(kind: &'static Kind, buffer: Option<Buffer>) {
    let _ = VECTORS.try_with(|pools| {
        if let Ok(mut pools) = pools.try_borrow_mut() {
            if let Some(pool) = pools.get_mut(&kind.id) {
                #[cfg(feature = "allocation-profile")]
                {
                    pool.stats.leased = pool.stats.leased.saturating_sub(1);
                }
                if let Some(buffer) = buffer {
                    if pool.free.len() < pool.limit {
                        pool.free.push(buffer);
                    } else if let Some((index, smallest)) =
                        pool.free.iter().enumerate().min_by_key(|(_, b)| b.capacity)
                    {
                        if buffer.capacity > smallest.capacity {
                            pool.free[index] = buffer;
                        }
                    }
                }
            }
        }
    });
}
pub(super) fn give<T: 'static>(values: std::vec::Vec<T>) {
    return_buffer(kind::<T>(), Buffer::from_vec(values));
}
pub(crate) fn recycle_owned<T: 'static>(values: std::vec::Vec<T>) {
    if values.capacity() != 0 {
        adopt::<T>();
        give(values);
    }
}
#[inline(never)]
fn prepare_kind(kind: &'static Kind, buffers: usize, elements: usize, fixed: bool) {
    if kind.element.size() == 0 || elements == 0 {
        return;
    }
    VECTORS.with_borrow_mut(|pools| {
        let pool = pools.entry(kind.id).or_insert_with(|| Pool::new(kind));
        if fixed {
            pool.fixed = true;
            pool.prepare(buffers, elements);
        } else if !pool.free.iter().any(|b| b.capacity >= elements) {
            pool.limit = pool.limit.max(pool.free.len() + 1);
            pool.free.reserve(1);
            pool.free.push(Buffer::new(kind.element, elements));
        }
    });
}
pub(crate) fn prepare<T: 'static>(buffers: usize, elements: usize) {
    prepare_kind(kind::<T>(), buffers, elements, true);
}
pub(crate) fn prepare_large<T: 'static>(elements: usize) {
    prepare_kind(kind::<T>(), 1, elements, false);
}
pub(super) fn prepare_registered(tiles: usize) {
    VECTORS.with_borrow_mut(|pools| {
        for pool in pools.values_mut() {
            if !pool.fixed {
                let numeric = pool.kind.id == TypeId::of::<usize>();
                if numeric && !pool.free.iter().any(|b| b.capacity >= tiles * tiles) {
                    pool.free
                        .push(Buffer::new(pool.kind.element, tiles * tiles));
                }
                pool.prepare(if numeric { 64 } else { 2 }, tiles + 1);
            }
        }
    });
}
#[cfg(feature = "allocation-profile")]
pub(super) fn report() -> std::vec::Vec<(&'static str, usize, usize, usize, usize)> {
    VECTORS.with_borrow(|pools| {
        pools
            .values()
            .map(|p| {
                (
                    (p.kind.name)(),
                    p.kind.element.size(),
                    p.free.iter().map(|b| b.layout.size()).sum(),
                    p.stats.peak,
                    p.free.len(),
                )
            })
            .collect()
    })
}
