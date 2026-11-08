//! Typed input lowering. Unit lanes disappear from the wasm32 C ABI.
use crate::{BoundaryResult, Decode};
use std::marker::PhantomData;
pub trait Argument<T> {
    type Adopted: Default;
    type A0: Copy;
    type A1: Copy;
    type A2: Copy;
    type A3: Copy;
    type A4: Copy;
    type A5: Copy;
    type A6: Copy;
    type A7: Copy;
    /// # Safety
    /// All lanes obey the generated schema. Called for every parameter before
    /// any fallible lift/reservation, transferring owned numeric allocations.
    unsafe fn adopt(
        _: Self::A0,
        _: Self::A1,
        _: Self::A2,
        _: Self::A3,
        _: Self::A4,
        _: Self::A5,
        _: Self::A6,
        _: Self::A7,
    ) -> Self::Adopted {
        Self::Adopted::default()
    }
    /// # Safety
    /// Lanes and encoded buffers must obey the matching generated ABI contract.
    unsafe fn lift(
        adopted: Self::Adopted,
        a0: Self::A0,
        a1: Self::A1,
        a2: Self::A2,
        a3: Self::A3,
        a4: Self::A4,
        a5: Self::A5,
        a6: Self::A6,
        a7: Self::A7,
    ) -> BoundaryResult<T>;
}
pub struct Buffer<T>(PhantomData<T>);
pub struct Flat<T>(PhantomData<T>);
pub struct Opaque<T>(PhantomData<T>);
impl<T: Decode> Argument<T> for Buffer<T> {
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
        unsafe { crate::input_trusted(id) }
    }
}

pub struct Vector<T>(PhantomData<T>);
impl<T: Decode> Argument<Vec<T>> for Vector<T> {
    type Adopted = Option<Vec<T>>;
    type A0 = usize;
    type A1 = usize;
    type A2 = ();
    type A3 = ();
    type A4 = ();
    type A5 = ();
    type A6 = ();
    type A7 = ();
    #[inline(always)]
    unsafe fn adopt(
        pointer: usize,
        len: usize,
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
    ) -> Self::Adopted {
        unsafe { T::adopt_numeric_sequence(pointer, len) }
    }
    #[inline(always)]
    unsafe fn lift(
        guard: Self::Adopted,
        id: usize,
        _: usize,
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
    ) -> BoundaryResult<Vec<T>> {
        match guard {
            Some(value) => Ok(value),
            None => unsafe { crate::input_trusted(id as u32) },
        }
    }
}

/// Structural returns normally use a bounded owning output buffer.
pub trait ReturnValue<T> {
    fn lower(value: T) -> BoundaryResult<u32>;
    fn complete(value: T) -> BoundaryResult<Completion>
    where
        T: crate::Encode,
    {
        crate::publish(value).map(Completion::Direct)
    }
    fn complete_result<E: crate::Encode>(value: Result<T, E>) -> BoundaryResult<Completion>
    where
        T: crate::Encode,
    {
        crate::publish(value).map(Completion::Direct)
    }
    fn lower_result<E: crate::Encode>(value: Result<T, E>) -> BoundaryResult<u32>
    where
        T: crate::Encode,
    {
        crate::publish(value)
    }
}
pub struct ResultOutput<T, E>(PhantomData<(T, E)>);
impl<T: crate::Encode, E: crate::Encode> ReturnValue<Result<T, E>> for ResultOutput<T, E> {
    fn lower(value: Result<T, E>) -> BoundaryResult<u32> {
        T::Return::lower_result(value)
    }
    fn complete(value: Result<T, E>) -> BoundaryResult<Completion> {
        T::Return::complete_result(value)
    }
}
impl<T: crate::Encode> ReturnValue<T> for Buffer<T> {
    fn lower(value: T) -> BoundaryResult<u32> {
        crate::publish(value)
    }
}
impl<T: crate::objects::Opaque> ReturnValue<T> for Opaque<T> {
    fn complete(value: T) -> BoundaryResult<Completion> {
        Self::lower(value).map(Completion::Direct)
    }
    fn lower(value: T) -> BoundaryResult<u32> {
        Ok(crate::insert(value))
    }
}
pub fn lower_output<T: crate::Encode>(value: T) -> BoundaryResult<u32> {
    T::Return::lower(value)
}
thread_local! {static RETURN_BYTES: std::cell::RefCell<[u8;64]> = const {std::cell::RefCell::new([0;64])};}
/// Fixed numeric returns occupy a single reusable ABI scratch area. The JS stub
/// copies all fields before returning to user code; no view or Rust reference escapes.
/// Async flat results hold an owned snapshot until the future and anchors drop.
/// Callbacks retain their ordinary owning output buffers.
pub fn flat_output(write: impl FnOnce(&mut [u8; 64])) -> u32 {
    RETURN_BYTES.with_borrow_mut(|bytes| {
        write(bytes);
        bytes.as_ptr() as usize as u32
    })
}

/// Task completion keeps scratch values detached until the future and its
/// anchors have been dropped. Only then may polling publish the shared slot.
/// Direct values are owning output IDs/pointers interpreted by the generated
/// result-specific completion adapter; they are never a pending sentinel.
pub enum Completion {
    Direct(u32),
    Scalar(u64),
    Scratch([u8; 64]),
    String(String),
    StringResult(Result<String, u32>),
    StringResultScalar(u64),
}
impl Completion {
    pub fn bytes(value: &[u8]) -> Self {
        let mut bytes = [0; 8];
        bytes[..value.len()].copy_from_slice(value);
        Self::Scalar(u64::from_le_bytes(bytes))
    }
    pub fn string_error(value: &[u8]) -> Self {
        let mut bytes = [0; 8];
        bytes[..value.len()].copy_from_slice(value);
        Self::StringResultScalar(u64::from_le_bytes(bytes))
    }
    pub fn write(write: impl FnOnce(&mut [u8; 64])) -> Self {
        let mut bytes = [0; 64];
        write(&mut bytes);
        Self::Scratch(bytes)
    }
    pub(crate) fn publish(self) -> BoundaryResult<u32> {
        match self {
            Self::Direct(id) => Ok(id),
            Self::Scalar(value) => Ok(flat_output(|bytes| {
                bytes[..8].copy_from_slice(&value.to_le_bytes())
            })),
            Self::Scratch(value) => Ok(flat_output(|bytes| *bytes = value)),
            Self::String(value) => crate::text::Output::lower(value),
            Self::StringResult(value) => crate::text::complete_result(value),
            Self::StringResultScalar(value) => {
                let pointer = flat_output(|bytes| bytes[..8].copy_from_slice(&value.to_le_bytes()));
                crate::text::complete_scalar_error(pointer)
            }
        }
    }
}
pub fn complete<T: crate::Encode>(value: T) -> BoundaryResult<Completion> {
    value.complete()
}
