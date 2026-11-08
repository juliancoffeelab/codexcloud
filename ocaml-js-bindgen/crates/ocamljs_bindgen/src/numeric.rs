//! Aligned numeric argument storage, handed to Rust before fallible lifting.
use crate::{BoundaryError, BoundaryResult};
use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    cell::Cell,
    marker::PhantomData,
    ptr::NonNull,
};
mod sealed {
    pub trait Sealed {}
}
pub trait Element: sealed::Sealed + Copy + 'static {
    const KIND: u32;
}
macro_rules! elements { ($($t:ty => $kind:expr),*) => {$ (
    impl sealed::Sealed for $t {}
    impl Element for $t { const KIND: u32 = $kind; }
)*}; }
elements!(u8=>0,i8=>1,u16=>2,i16=>3,u32=>4,i32=>5,f32=>6,f64=>7,usize=>8,isize=>9);
thread_local! { static PENDING: Cell<usize> = const {Cell::new(0)}; }
pub(crate) fn pending() -> usize {
    PENDING.with(Cell::get)
}
pub(crate) fn retained() {
    if crate::RESOURCE_DIAGNOSTICS {
        PENDING.with(|n| n.set(n.get() + 1));
    }
}
fn adopted() {
    if crate::RESOURCE_DIAGNOSTICS {
        PENDING.with(|n| n.set(n.get() - 1));
    }
}
#[inline(always)]
pub fn allocate<T: Element>(len: usize) -> BoundaryResult<usize> {
    if crate::PARANOID && len > crate::MAX_ITEMS {
        return Err(BoundaryError("numeric input exceeds item limit"));
    }
    allocate_raw::<T>(len)
}
#[inline(always)]
pub(crate) fn allocate_raw<T: Element>(len: usize) -> BoundaryResult<usize> {
    let layout =
        Layout::array::<T>(len).map_err(|_| BoundaryError("numeric input size overflow"))?;
    let pointer = if len == 0 {
        NonNull::<T>::dangling().as_ptr().cast::<u8>()
    } else {
        let pointer = unsafe { alloc(layout) };
        if pointer.is_null() {
            handle_alloc_error(layout);
        }
        pointer
    };
    if crate::RESOURCE_DIAGNOSTICS {
        PENDING.with(|n| n.set(n.get() + 1));
    }
    Ok(pointer as usize)
}
unsafe fn release<T: Element>(pointer: usize, len: usize) {
    if len != 0 {
        unsafe { dealloc(pointer as *mut u8, Layout::array::<T>(len).unwrap()) };
    }
}
/// # Safety
/// The pointer and exact length must come from allocate::<T>, fully initialized,
/// and be transferred once. Its allocation layout is exactly Vec's length/capacity.
#[inline(always)]
pub unsafe fn take_vec<T: Element>(pointer: usize, len: usize) -> Vec<T> {
    unsafe { take_vec_parts(pointer, len, len) }
}
/// # Safety
/// Allocation layout is capacity elements; exactly the len prefix is initialized.
#[inline(always)]
pub unsafe fn take_vec_parts<T: Element>(pointer: usize, len: usize, capacity: usize) -> Vec<T> {
    adopted();
    unsafe { Vec::from_raw_parts(pointer as *mut T, len, capacity) }
}
/// Owns temporary slice storage; no Vec or refcount is constructed.
pub struct Slice<T: Element> {
    pointer: NonNull<T>,
    len: usize,
    _local: PhantomData<std::rc::Rc<()>>,
}
impl<T: Element> Slice<T> {
    /// # Safety
    /// Same transfer contract as take_vec. Storage remains live through this guard.
    pub unsafe fn adopt(pointer: usize, len: usize) -> Self {
        adopted();
        Self {
            pointer: unsafe { NonNull::new_unchecked(pointer as *mut T) },
            len,
            _local: PhantomData,
        }
    }
    pub fn get(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), self.len) }
    }
}
impl<T: Element> std::ops::Deref for Slice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.get()
    }
}
impl<T: Element> Drop for Slice<T> {
    fn drop(&mut self) {
        unsafe { release::<T>(self.pointer.as_ptr() as usize, self.len) };
    }
}
impl<T: Element> crate::host::Borrowable for [T] {
    type A1 = usize;
    type A2 = ();
    type Adopted = Option<Slice<T>>;
    type Anchor = Slice<T>;
    unsafe fn adopt(pointer: usize, len: usize, _: ()) -> Self::Adopted {
        Some(unsafe { Slice::adopt(pointer, len) })
    }
    unsafe fn anchor(
        _: usize,
        _: usize,
        _: (),
        guard: Self::Adopted,
    ) -> BoundaryResult<Self::Anchor> {
        Ok(guard.unwrap())
    }
    unsafe fn with_shared<R>(
        _: usize,
        _: usize,
        _: (),
        guard: Self::Adopted,
        f: impl FnOnce(&Self) -> R,
    ) -> BoundaryResult<R> {
        Ok(f(guard.as_ref().unwrap().get()))
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_numeric_alloc(len: usize, kind: u32) -> usize {
    crate::boundary(match kind {
        0 => allocate::<u8>(len),
        1 => allocate::<i8>(len),
        2 => allocate::<u16>(len),
        3 => allocate::<i16>(len),
        4 => allocate::<u32>(len),
        5 => allocate::<i32>(len),
        6 => allocate::<f32>(len),
        7 => allocate::<f64>(len),
        8 => allocate::<usize>(len),
        9 => allocate::<isize>(len),
        _ => Err(BoundaryError("unknown numeric input kind")),
    })
}
/// # Safety
/// Free only a matching allocation which has not been transferred to Rust.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_numeric_free(pointer: usize, len: usize, kind: u32) {
    adopted();
    unsafe {
        match kind {
            0 => release::<u8>(pointer, len),
            1 => release::<i8>(pointer, len),
            2 => release::<u16>(pointer, len),
            3 => release::<i16>(pointer, len),
            4 => release::<u32>(pointer, len),
            5 => release::<i32>(pointer, len),
            6 => release::<f32>(pointer, len),
            7 => release::<f64>(pointer, len),
            8 => release::<usize>(pointer, len),
            9 => release::<isize>(pointer, len),
            _ => panic!("unknown numeric input kind"),
        }
    }
}
