//! Stable ownership anchors for exported futures. Lifetime erasure is confined here.
use crate::{BoundaryError, BoundaryResult};
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

pub type LocalFuture<'a> = Pin<Box<dyn Future<Output = BoundaryResult<u32>> + 'a>>;
pub type CompletionFuture<'a> =
    Pin<Box<dyn Future<Output = BoundaryResult<crate::abi::Completion>> + 'a>>;

pub struct SharedAnchor<T: crate::objects::Opaque>(crate::objects::Anchor<T>);
impl<T: crate::objects::Opaque> SharedAnchor<T> {
    pub unsafe fn new(id: u32) -> BoundaryResult<Self> {
        Ok(Self(unsafe { crate::objects::Anchor::new(id, false)? }))
    }
    pub fn get(&self) -> &T {
        self.0.get()
    }
}
impl<T: crate::objects::Opaque> std::ops::Deref for SharedAnchor<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.get()
    }
}
pub struct MutAnchor<T: crate::objects::Opaque>(crate::objects::Anchor<T>);
impl<T: crate::objects::Opaque> MutAnchor<T> {
    pub unsafe fn new(id: u32) -> BoundaryResult<Self> {
        Ok(Self(unsafe { crate::objects::Anchor::new(id, true)? }))
    }
    pub fn get_mut(&mut self) -> &mut T {
        // MutAnchor::new always acquires an exclusive anchor.
        unsafe { self.0.get_mut() }
    }
}

pub struct BytesAnchor(Rc<Vec<u8>>);
impl BytesAnchor {
    pub fn new(id: u32) -> BoundaryResult<Self> {
        Ok(Self(crate::input_owner(id)?))
    }
    pub fn get(&self) -> &[u8] {
        &self.0
    }
    pub fn text(&self) -> BoundaryResult<&str> {
        std::str::from_utf8(self.get()).map_err(|_| BoundaryError("invalid UTF-8"))
    }
}

/// An exclusive reservation checked before any ownership-consuming operation.
pub struct MoveAnchor<T: crate::objects::Opaque> {
    id: u32,
    anchor: crate::objects::Anchor<T>,
}
impl<T: crate::objects::Opaque> MoveAnchor<T> {
    pub unsafe fn new(id: u32) -> BoundaryResult<Self> {
        Ok(Self {
            id,
            anchor: unsafe { crate::objects::Anchor::new(id, true)? },
        })
    }
    pub fn take(self) -> T {
        self.anchor.take(self.id)
    }
}

// A raw allocation owner avoids Box's unique-owner retagging when the task is
// moved after creating references into A. Only Drop reconstructs Box, after the
// future and every reference into the allocation have been destroyed.
struct StableAnchor {
    allocation: std::ptr::NonNull<()>,
    destroy: unsafe fn(std::ptr::NonNull<()>),
}
impl Drop for StableAnchor {
    fn drop(&mut self) {
        // SAFETY: this owns exactly one original Box allocation, and future is gone.
        unsafe {
            (self.destroy)(self.allocation);
        }
    }
}
unsafe fn destroy_anchor<A>(pointer: std::ptr::NonNull<()>) {
    // SAFETY: pointer originated from Box<A>::into_raw and is reconstructed once.
    unsafe {
        drop(Box::from_raw(pointer.cast::<A>().as_ptr()));
    }
}
pub(crate) struct ScopedFuture {
    // Explicit Drop below is essential: future first, then its borrowed anchors.
    future: Option<CompletionFuture<'static>>,
    _anchors: StableAnchor,
}
impl ScopedFuture {
    pub(crate) fn new<A: 'static, F>(anchors: A, build: F) -> Self
    where
        F: for<'a> FnOnce(&'a mut A) -> CompletionFuture<'a>,
    {
        let mut pointer = std::ptr::NonNull::new(Box::into_raw(Box::new(anchors))).unwrap();
        let owner = StableAnchor {
            allocation: pointer.cast(),
            destroy: destroy_anchor::<A>,
        };
        // SAFETY: no prior references exist. Future is the only accessor until Drop.
        let future = build(unsafe { pointer.as_mut() });
        // SAFETY: the raw-owned allocation is stable when this struct moves. A is private
        // and never accessed/replaced while future exists. The HRTB builder cannot
        // return references with any longer lifetime than its anchor borrow. This
        // local !Send future never escapes. Drop always destroys it before StableAnchor,
        // on Ready, boundary error, cancellation, and normal registry destruction.
        let future = unsafe {
            std::mem::transmute::<CompletionFuture<'_>, CompletionFuture<'static>>(future)
        };
        Self {
            future: Some(future),
            _anchors: owner,
        }
    }
    pub(crate) fn poll(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<BoundaryResult<crate::abi::Completion>> {
        self.future.as_mut().unwrap().as_mut().poll(cx)
    }
}
impl Drop for ScopedFuture {
    fn drop(&mut self) {
        self.future.take();
    }
}
