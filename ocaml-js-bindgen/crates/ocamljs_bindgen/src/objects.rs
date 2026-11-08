//! Direct owning pointers. Only borrow state is consulted on ordinary calls.
use crate::{BoundaryError, BoundaryResult};
use std::{
    cell::{Cell, UnsafeCell},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::NonNull,
    rc::Rc,
};

/// Marker supplied by the opaque declaration macro.
pub trait Opaque: Sized + 'static {}
#[doc(hidden)]
#[macro_export]
macro_rules! opaque_storage {
    ($ty:ty) => {
        impl $crate::objects::Opaque for $ty {}
    };
}

// The destructor is used by heterogeneous packet rollback and finalization.
// Typed access never consults it. No directory, type key, store or page exists.
#[repr(C)]
struct Header {
    borrow: Cell<isize>,
    destroy: unsafe fn(u32) -> BoundaryResult<()>,
}
#[repr(C)]
struct Allocation<T> {
    header: Header,
    value: UnsafeCell<ManuallyDrop<T>>,
}
impl Header {
    fn acquire(&self, mutable: bool) -> BoundaryResult<()> {
        let n = self.borrow.get();
        if (mutable && n != 0) || (!mutable && (n < 0 || n == isize::MAX)) {
            return Err(BoundaryError("conflicting opaque borrow"));
        }
        self.borrow.set(if mutable { -1 } else { n + 1 });
        Ok(())
    }
    fn release(&self, mutable: bool) {
        self.borrow
            .set(if mutable { 0 } else { self.borrow.get() - 1 });
    }
}
thread_local! { static LIVE: Cell<usize> = const { Cell::new(0) }; }

// This registry is absent from trusted wasm32. Native tests need a u32 bridge
// because their pointers are 64 bit; paranoid builds additionally check types.
#[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
mod checked {
    use super::*;
    use std::{any::TypeId, cell::RefCell, collections::HashMap};
    #[derive(Default)]
    struct Registry {
        #[cfg(not(target_arch = "wasm32"))]
        next: u32,
        entries: HashMap<u32, (usize, TypeId)>,
    }
    thread_local! { static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default()); }
    pub fn insert<T: Opaque>(pointer: usize) -> u32 {
        REGISTRY.with_borrow_mut(|r| {
            #[cfg(target_arch = "wasm32")]
            let id = pointer as u32;
            #[cfg(not(target_arch = "wasm32"))]
            let id = {
                r.next = r
                    .next
                    .checked_add(1)
                    .expect("native handle namespace exhausted");
                r.next
            };
            assert!(r.entries.insert(id, (pointer, TypeId::of::<T>())).is_none());
            id
        })
    }
    pub fn pointer(id: u32, ty: Option<std::any::TypeId>) -> BoundaryResult<usize> {
        REGISTRY.with_borrow(|r| {
            let (pointer, actual) = r
                .entries
                .get(&id)
                .ok_or(BoundaryError("invalid or disposed handle"))?;
            if ty.is_some_and(|ty| ty != *actual) {
                return Err(BoundaryError("opaque type mismatch"));
            }
            Ok(*pointer)
        })
    }
    pub fn remove(id: u32) {
        REGISTRY.with_borrow_mut(|r| {
            r.entries.remove(&id);
        });
    }
    pub fn contains(id: u32) -> bool {
        REGISTRY.with_borrow(|r| r.entries.contains_key(&id))
    }
}
#[inline]
fn pointer<T: Opaque>(id: u32) -> BoundaryResult<NonNull<Allocation<T>>> {
    #[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
    let raw = checked::pointer(id, Some(std::any::TypeId::of::<T>()))?;
    #[cfg(not(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid)))]
    let raw = id as usize;
    NonNull::new(raw as *mut Allocation<T>).ok_or(BoundaryError("disposed handle"))
}
pub fn insert<T: Opaque>(value: T) -> u32 {
    let allocation = Box::new(Allocation {
        header: Header {
            borrow: Cell::new(0),
            destroy: destroy::<T>,
        },
        value: UnsafeCell::new(ManuallyDrop::new(value)),
    });
    let raw = Box::into_raw(allocation) as usize;
    #[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
    let id = checked::insert::<T>(raw);
    #[cfg(not(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid)))]
    let id = raw as u32;
    if crate::RESOURCE_DIAGNOSTICS {
        LIVE.with(|live| live.set(live.get() + 1));
    }
    id
}
/// # Safety
/// `id` must name a live allocation of the matching generated instance. It must
/// not be reused after successful disposal or ownership transfer.
pub unsafe fn dispose(id: u32) -> BoundaryResult<()> {
    #[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
    let raw = checked::pointer(id, None)?;
    #[cfg(not(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid)))]
    let raw = id as usize;
    if raw == 0 {
        return Err(BoundaryError("disposed handle"));
    }
    let header = unsafe { &*(raw as *const Header) };
    unsafe { (header.destroy)(id) }
}
unsafe fn destroy<T: Opaque>(id: u32) -> BoundaryResult<()> {
    let pointer = pointer::<T>(id)?;
    let allocation = unsafe { pointer.as_ref() };
    if allocation.header.borrow.get() != 0 {
        return Err(BoundaryError("cannot dispose borrowed object"));
    }
    // No allocation/reference is destroyed until all borrowers have released it.
    let value = unsafe { take_allocation(id, pointer) };
    drop(value);
    Ok(())
}
unsafe fn take_allocation<T: Opaque>(id: u32, pointer: NonNull<Allocation<T>>) -> T {
    #[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
    checked::remove(id);
    let _ = id;
    if crate::RESOURCE_DIAGNOSTICS {
        LIVE.with(|live| live.set(live.get() - 1));
    }
    let allocation = unsafe { Box::from_raw(pointer.as_ptr()) };
    ManuallyDrop::into_inner(allocation.value.into_inner())
}
pub(crate) fn live() -> usize {
    LIVE.with(Cell::get)
}
#[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
pub(crate) fn contains(id: u32) -> bool {
    checked::contains(id)
}

struct CallGuard<'a> {
    header: &'a Header,
    mutable: bool,
}
impl Drop for CallGuard<'_> {
    fn drop(&mut self) {
        self.header.release(self.mutable);
    }
}
/// # Safety
/// `id` must name a live `T` in this instance; it remains owned throughout `f`.
pub unsafe fn with_ref<T: Opaque, R>(id: u32, f: impl FnOnce(&T) -> R) -> BoundaryResult<R> {
    let pointer = pointer::<T>(id)?;
    let allocation = unsafe { pointer.as_ref() };
    allocation.header.acquire(false)?;
    let _guard = CallGuard {
        header: &allocation.header,
        mutable: false,
    };
    Ok(f(unsafe { &*allocation.value.get() }))
}
/// # Safety
/// `id` must name a live `T` in this instance; it remains owned throughout `f`.
pub unsafe fn with_mut<T: Opaque, R>(id: u32, f: impl FnOnce(&mut T) -> R) -> BoundaryResult<R> {
    let pointer = pointer::<T>(id)?;
    let allocation = unsafe { pointer.as_ref() };
    allocation.header.acquire(true)?;
    let _guard = CallGuard {
        header: &allocation.header,
        mutable: true,
    };
    Ok(f(unsafe { &mut *allocation.value.get() }))
}
/// A borrow pins the allocation: dispose and move reject any outstanding borrow.
/// No extra reference count is necessary because the owner cannot free it.
pub(crate) struct Anchor<T: Opaque> {
    pointer: NonNull<Allocation<T>>,
    mutable: bool,
    active: bool,
    _local: PhantomData<Rc<()>>,
}
impl<T: Opaque> Anchor<T> {
    pub(crate) unsafe fn new(id: u32, mutable: bool) -> BoundaryResult<Self> {
        let pointer = pointer::<T>(id)?;
        unsafe { pointer.as_ref() }.header.acquire(mutable)?;
        Ok(Self {
            pointer,
            mutable,
            active: true,
            _local: PhantomData,
        })
    }
    pub(crate) fn get(&self) -> &T {
        unsafe { &*self.pointer.as_ref().value.get() }
    }
    pub(crate) unsafe fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.pointer.as_ref().value.get() }
    }
    pub(crate) fn take(mut self, id: u32) -> T {
        self.active = false;
        unsafe { take_allocation(id, self.pointer) }
    }
}
impl<T: Opaque> Drop for Anchor<T> {
    fn drop(&mut self) {
        if self.active {
            unsafe { self.pointer.as_ref() }
                .header
                .release(self.mutable);
        }
    }
}
// Generated consuming shims reset this before preflight and increment it only
// after each successful transfer. JS invalidates exactly that prefix, even when
// later user code fails. No contains lookup or stale-pointer probe is needed.
thread_local! { static MOVED: Cell<u32> = const { Cell::new(0) }; }
pub fn reset_moves() {
    MOVED.with(|m| m.set(0));
}
pub fn record_move() {
    MOVED.with(|m| m.set(m.get() + 1));
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_moved() -> u32 {
    MOVED.with(Cell::get)
}
