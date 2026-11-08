//! Host references live in a Wasm externref table, never a JavaScript ID map.
//!
//! The packager replaces the numeric placeholder imports below with Wasm table
//! operations. Borrowed ABI slots are call-scoped; owning values clone into a
//! separate root slot so async suspension and Rust Drop have ordinary semantics.
use crate::{BoundaryError, BoundaryResult, Decode, Encode, Encoder};
use std::{cell::RefCell, marker::PhantomData, ops::Deref};

pub const BORROW_SLOTS: u32 = 128;
const MAX_ROOTS: u32 = 1_000_000;

#[derive(Default)]
struct Roots {
    next: u32,
    free: Vec<u32>,
    occupied: Vec<bool>,
    live: u32,
}
thread_local! { static ROOTS: RefCell<Roots> = RefCell::new(Roots::default()); }

pub fn allocate() -> u32 {
    ROOTS.with_borrow_mut(|r| {
        assert!(r.live < MAX_ROOTS, "host reference limit");
        let id = r.free.pop().unwrap_or_else(|| {
            let id = BORROW_SLOTS + r.next;
            r.next += 1;
            r.occupied.push(false);
            id
        });
        assert!(!std::mem::replace(
            &mut r.occupied[(id - BORROW_SLOTS) as usize],
            true
        ));
        r.live += 1;
        id
    })
}
pub fn release(id: u32) {
    ROOTS.with_borrow_mut(|r| {
        assert!(id >= BORROW_SLOTS && id < BORROW_SLOTS + r.next && r.live > 0);
        assert!(
            std::mem::replace(&mut r.occupied[(id - BORROW_SLOTS) as usize], false),
            "released host root"
        );
        r.free.push(id);
        r.live -= 1;
    });
}
pub fn live() -> u32 {
    ROOTS.with_borrow(|r| r.live)
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs_ref")]
unsafe extern "C" {
    fn clone(id: u32) -> u32;
    fn drop(id: u32);
}

/// An owning, thread-affine root. Its numeric representation is internal only.
pub struct Ref {
    id: u32,
    local: PhantomData<std::rc::Rc<()>>,
}
impl Ref {
    /// # Safety
    /// `id` must be a live, exclusively transferred root from the generated ABI.
    pub unsafe fn from_owned(id: u32) -> Self {
        Self {
            id,
            local: PhantomData,
        }
    }
    /// # Safety
    /// The ABI slot must remain live until this operation returns.
    pub unsafe fn from_borrowed(id: u32) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { Self::from_owned(clone(id)) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = id;
            panic!("host references require a packaged Wasm module")
        }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn into_raw(self) -> u32 {
        std::mem::ManuallyDrop::new(self).id
    }
}
impl Clone for Ref {
    fn clone(&self) -> Self {
        unsafe { Self::from_borrowed(self.id) }
    }
}
impl Drop for Ref {
    fn drop(&mut self) {
        #[cfg(target_arch = "wasm32")]
        unsafe {
            drop(self.id);
        }
    }
}

/// Implemented by generated imported host types. Safe Rust cannot forge a root.
pub trait Value: Sized {
    fn reference(&self) -> &Ref;
    fn into_reference(self) -> Ref;
    fn from_reference(reference: Ref) -> Self;
}
pub struct Direct<T>(PhantomData<T>);
impl<T: Value> crate::abi::Argument<T> for Direct<T> {
    type Adopted = ();
    type A0 = u32;
    type A1 = ();
    type A2 = ();
    type A3 = ();
    type A4 = ();
    type A5 = ();
    type A6 = ();
    type A7 = ();
    unsafe fn lift(
        _: (),
        id: u32,
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
    ) -> BoundaryResult<T> {
        Ok(T::from_reference(unsafe { Ref::from_borrowed(id) }))
    }
}
impl<T: Value> crate::abi::ReturnValue<T> for Direct<T> {
    fn complete(value: T) -> BoundaryResult<crate::abi::Completion> {
        Ok(crate::abi::Completion::Direct(
            value.into_reference().into_raw(),
        ))
    }
    fn lower(value: T) -> BoundaryResult<u32> {
        Ok(value.into_reference().into_raw())
    }
}
pub fn encode<T: Value>(value: T, out: &mut Encoder) {
    out.opaque(value.into_reference());
}
pub fn decode<T: Value>(_: &mut crate::Decoder<'_>) -> BoundaryResult<T> {
    Err(BoundaryError("host values must be direct input parameters"))
}

/// Borrow dispatch is chosen statically, keeping Rust object guards separate.
pub trait Borrowable {
    type A1: Copy;
    type A2: Copy;
    type Adopted: Default;
    unsafe fn adopt(_: usize, _: Self::A1, _: Self::A2) -> Self::Adopted {
        Self::Adopted::default()
    }
    type Anchor: Deref<Target = Self>;
    unsafe fn anchor(
        id: usize,
        len: Self::A1,
        capacity: Self::A2,
        adopted: Self::Adopted,
    ) -> BoundaryResult<Self::Anchor>;
    unsafe fn with_shared<R>(
        id: usize,
        len: Self::A1,
        capacity: Self::A2,
        adopted: Self::Adopted,
        f: impl FnOnce(&Self) -> R,
    ) -> BoundaryResult<R>;
}
pub struct Anchor<T>(T);
impl<T> Deref for Anchor<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
pub fn anchor<T: Value>(id: u32) -> BoundaryResult<Anchor<T>> {
    Ok(Anchor(T::from_reference(unsafe { Ref::from_borrowed(id) })))
}
pub fn with_shared<T: Value, R>(id: u32, f: impl FnOnce(&T) -> R) -> BoundaryResult<R> {
    let value = std::mem::ManuallyDrop::new(T::from_reference(unsafe { Ref::from_owned(id) }));
    Ok(f(&value))
}

/// A caught JavaScript thrown value. Identity is preserved, including primitives.
pub struct HostError(Ref);
impl Clone for HostError {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Borrowable for HostError {
    type A1 = ();
    type A2 = ();
    type Adopted = ();
    type Anchor = Anchor<Self>;
    unsafe fn anchor(id: usize, _: (), _: (), _: ()) -> BoundaryResult<Self::Anchor> {
        anchor(id as u32)
    }
    unsafe fn with_shared<R>(
        id: usize,
        _: (),
        _: (),
        _: (),
        f: impl FnOnce(&Self) -> R,
    ) -> BoundaryResult<R> {
        with_shared(id as u32, f)
    }
}
impl Value for HostError {
    fn reference(&self) -> &Ref {
        &self.0
    }
    fn into_reference(self) -> Ref {
        self.0
    }
    fn from_reference(value: Ref) -> Self {
        Self(value)
    }
}
impl Encode for HostError {
    type Return = Direct<Self>;
    fn encode(self, out: &mut Encoder) {
        encode(self, out);
    }
}
impl Decode for HostError {
    type Abi = Direct<Self>;
    fn decode(d: &mut crate::Decoder<'_>) -> BoundaryResult<Self> {
        decode(d)
    }
}
impl std::fmt::Debug for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostError(JavaScript thrown value)")
    }
}
impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JavaScript operation failed")
    }
}
impl std::error::Error for HostError {}

type Settlement = Result<Ref, HostError>;
#[derive(Default)]
struct PromiseState {
    result: Option<Settlement>,
    waker: Option<std::task::Waker>,
}
struct PromiseChannel(std::rc::Rc<RefCell<PromiseState>>);
crate::opaque_storage!(PromiseChannel);
crate::opaque_storage!(Ref);
/// An existing JavaScript Promise. Await it directly from ordinary Rust async.
pub struct Promise<T: Value> {
    reference: Option<Ref>,
    channel: Option<(u32, std::rc::Rc<RefCell<PromiseState>>)>,
    marker: PhantomData<T>,
}
impl<T: Value> Unpin for Promise<T> {}
impl<T: Value> Value for Promise<T> {
    fn reference(&self) -> &Ref {
        self.reference.as_ref().unwrap()
    }
    fn into_reference(mut self) -> Ref {
        self.reference.take().unwrap()
    }
    fn from_reference(value: Ref) -> Self {
        Self {
            reference: Some(value),
            channel: None,
            marker: PhantomData,
        }
    }
}
impl<T: Value> Clone for Promise<T> {
    fn clone(&self) -> Self {
        Self::from_reference(self.reference().clone())
    }
}
impl<T: Value> Encode for Promise<T> {
    type Return = Direct<Self>;
    fn encode(self, out: &mut Encoder) {
        encode(self, out)
    }
}
impl<T: Value> Decode for Promise<T> {
    type Abi = Direct<Self>;
    fn decode(d: &mut crate::Decoder<'_>) -> BoundaryResult<Self> {
        decode(d)
    }
}
impl<T: Value> Borrowable for Promise<T> {
    type A1 = ();
    type A2 = ();
    type Adopted = ();
    type Anchor = Anchor<Self>;
    unsafe fn anchor(id: usize, _: (), _: (), _: ()) -> BoundaryResult<Self::Anchor> {
        anchor(id as u32)
    }
    unsafe fn with_shared<R>(
        id: usize,
        _: (),
        _: (),
        _: (),
        f: impl FnOnce(&Self) -> R,
    ) -> BoundaryResult<R> {
        with_shared(id as u32, f)
    }
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs")]
unsafe extern "C" {
    fn host_promise_subscribe(promise: u32, channel: u32);
    fn host_promise_unsubscribe(channel: u32);
}
impl<T: Value> std::future::Future for Promise<T> {
    type Output = Result<T, HostError>;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        if this.channel.is_none() {
            let state = std::rc::Rc::new(RefCell::new(PromiseState::default()));
            let id = crate::insert(PromiseChannel(state.clone()));
            this.channel = Some((id, state));
            #[cfg(target_arch = "wasm32")]
            unsafe {
                host_promise_subscribe(this.reference().id(), id);
            }
            #[cfg(not(target_arch = "wasm32"))]
            panic!("JavaScript promises require packaged Wasm");
        }
        let mut state = this.channel.as_ref().unwrap().1.borrow_mut();
        if let Some(result) = state.result.take() {
            std::task::Poll::Ready(result.map(T::from_reference))
        } else {
            state.waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
}
impl<T: Value> Drop for Promise<T> {
    fn drop(&mut self) {
        if let Some((id, _)) = self.channel.take() {
            #[cfg(target_arch = "wasm32")]
            unsafe {
                host_promise_unsubscribe(id);
            }
            let _ = unsafe { crate::dispose(id) };
        }
    }
}
pub unsafe fn settle(channel: u32, value: u32, ok: u32) {
    let Ok(state) = (unsafe { crate::with_ref::<PromiseChannel, _>(channel, |c| c.0.clone()) })
    else {
        return;
    };
    let wake = {
        let mut state = state.borrow_mut();
        if state.result.is_some() {
            return;
        }
        let value = unsafe { Ref::from_borrowed(value) };
        state.result = Some(if ok != 0 {
            Ok(value)
        } else {
            Err(HostError(value))
        });
        state.waker.take()
    };
    if let Some(wake) = wake {
        wake.wake();
    }
}

/// The output owns this root until the decoder closes its packet. The returned
/// externref is independently kept alive by the host GC after packet cleanup.
pub fn output(output: u32, id: u32) -> u32 {
    crate::boundary((|| {
        if !crate::output_owns(output, id) {
            return Err(BoundaryError("unowned host output"));
        }
        unsafe { crate::with_ref::<Ref, _>(id, Ref::id) }
    })())
}

/// A bounded copy could not complete. Host exceptions preserve their JS identity.
#[derive(Debug)]
pub enum CopyError {
    TooLarge,
    Host(HostError),
}
impl std::fmt::Display for CopyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => f.write_str("host value exceeds copy limit"),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for CopyError {}
macro_rules! native_value {
    ($name:ident) => {
        impl Value for $name {
            fn reference(&self) -> &Ref {
                &self.0
            }
            fn into_reference(self) -> Ref {
                self.0
            }
            fn from_reference(value: Ref) -> Self {
                Self(value)
            }
        }
        impl Clone for $name {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }
        impl Encode for $name {
            type Return = Direct<Self>;
            fn encode(self, out: &mut Encoder) {
                encode(self, out)
            }
        }
        impl Decode for $name {
            type Abi = Direct<Self>;
            fn decode(d: &mut crate::Decoder<'_>) -> BoundaryResult<Self> {
                decode(d)
            }
        }
        impl Borrowable for $name {
            type A1 = ();
            type A2 = ();
            type Adopted = ();
            type Anchor = Anchor<Self>;
            unsafe fn anchor(id: usize, _: (), _: (), _: ()) -> BoundaryResult<Self::Anchor> {
                anchor(id as u32)
            }
            unsafe fn with_shared<R>(
                id: usize,
                _: (),
                _: (),
                _: (),
                f: impl FnOnce(&Self) -> R,
            ) -> BoundaryResult<R> {
                with_shared(id as u32, f)
            }
        }
    };
}
/// A native JavaScript string, with explicit bounded UTF-8 conversion for Rust.
pub struct JsString(Ref);
native_value!(JsString);
/// A native pixel byte array. Copies cross address spaces once, in bulk.
pub struct Uint8ClampedArray(Ref);
native_value!(Uint8ClampedArray);

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs")]
unsafe extern "C" {
    fn host_string_length(value: u32, error: usize) -> u32;
    fn host_string_copy(value: u32, ptr: usize, len: usize, error: usize) -> u32;
    fn host_bytes_new(ptr: usize, len: usize, error: usize) -> u32;
    fn host_bytes_prepare(value: u32, max: usize, len: usize, error: usize) -> u32;
    fn host_bytes_copy(value: u32, ptr: usize, len: usize, error: usize);
}
#[cfg(target_arch = "wasm32")]
fn caught<T>(call: impl FnOnce(usize) -> T) -> Result<T, CopyError> {
    let mut error = 0u32;
    let value = call(&mut error as *mut u32 as usize);
    if error == 0 {
        Ok(value)
    } else {
        Err(CopyError::Host(HostError(unsafe {
            Ref::from_owned(error)
        })))
    }
}
impl JsString {
    /// Encode into bounded Rust storage without allocating an intermediate JS
    /// byte buffer. Lone JS surrogates follow TextEncoder replacement semantics.
    pub fn to_rust_string(&self, max_bytes: usize) -> Result<String, CopyError> {
        #[cfg(target_arch = "wasm32")]
        {
            let max_bytes = max_bytes.min(crate::MAX_BYTES);
            let units = caught(|error| unsafe { host_string_length(self.0.id(), error) })? as usize;
            if units > max_bytes {
                return Err(CopyError::TooLarge);
            }
            let capacity = units.saturating_mul(3).min(max_bytes);
            let mut bytes = Vec::<u8>::with_capacity(capacity);
            let written = caught(|error| unsafe {
                host_string_copy(self.0.id(), bytes.as_mut_ptr() as usize, capacity, error)
            })?;
            if written == u32::MAX {
                return Err(CopyError::TooLarge);
            }
            assert!(written as usize <= capacity, "invalid UTF-8 copy length");
            unsafe { bytes.set_len(written as usize) };
            if crate::PARANOID {
                Ok(String::from_utf8(bytes).expect("TextEncoder produced invalid UTF-8"))
            } else {
                // The matching host helper uses TextEncoder, which always produces UTF-8.
                Ok(unsafe { String::from_utf8_unchecked(bytes) })
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = max_bytes;
            panic!("host strings require packaged Wasm")
        }
    }
}
impl Uint8ClampedArray {
    /// Copy bytes once into JS-owned storage; the JS array never aliases Rust.
    pub fn copy_from(bytes: &[u8]) -> Result<Self, CopyError> {
        if bytes.len() > crate::MAX_BYTES {
            return Err(CopyError::TooLarge);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let id = caught(|error| unsafe {
                host_bytes_new(bytes.as_ptr() as usize, bytes.len(), error)
            })?;
            Ok(Self(unsafe { Ref::from_owned(id) }))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            panic!("host arrays require packaged Wasm")
        }
    }
    /// Copy into independent Rust storage, rejecting the size before allocation.
    /// A view of this module's Wasm memory is snapshotted before Rust allocation
    /// can grow, detach or reuse its storage; that alias case needs an extra copy.
    pub fn to_vec(&self, max_bytes: usize) -> Result<Vec<u8>, CopyError> {
        #[cfg(target_arch = "wasm32")]
        {
            let max_bytes = max_bytes.min(crate::MAX_BYTES);
            let mut length = 0u32;
            let prepared = caught(|error| unsafe {
                host_bytes_prepare(
                    self.0.id(),
                    max_bytes,
                    &mut length as *mut u32 as usize,
                    error,
                )
            })?;
            let prepared = unsafe { Ref::from_owned(prepared) };
            let len = length as usize;
            if len > max_bytes.min(crate::MAX_BYTES) {
                return Err(CopyError::TooLarge);
            }
            let mut bytes = vec![0u8; len];
            caught(|error| unsafe {
                host_bytes_copy(prepared.id(), bytes.as_mut_ptr() as usize, len, error)
            })?;
            Ok(bytes)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = max_bytes;
            panic!("host arrays require packaged Wasm")
        }
    }
}
