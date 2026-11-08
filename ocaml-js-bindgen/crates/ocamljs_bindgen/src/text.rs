//! Owned UTF-8 argument storage. Initialized length and allocation capacity are distinct.
use crate::{BoundaryError, BoundaryResult};
use std::alloc::{Layout, handle_alloc_error, realloc};

pub struct Owned;
impl crate::abi::Argument<String> for Owned {
    type Adopted = Option<Vec<u8>>;
    type A0 = usize;
    type A1 = usize;
    type A2 = usize;
    type A3 = ();
    type A4 = ();
    type A5 = ();
    type A6 = ();
    type A7 = ();
    #[inline(always)]
    unsafe fn adopt(
        p: usize,
        n: usize,
        cap: usize,
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
    ) -> Self::Adopted {
        Some(unsafe { crate::numeric::take_vec_parts(p, n, cap) })
    }
    #[inline(always)]
    unsafe fn lift(
        data: Self::Adopted,
        _: usize,
        _: usize,
        _: usize,
        _: (),
        _: (),
        _: (),
        _: (),
        _: (),
    ) -> BoundaryResult<String> {
        string(data.unwrap())
    }
}
#[inline(always)]
fn string(data: Vec<u8>) -> BoundaryResult<String> {
    if crate::PARANOID {
        String::from_utf8(data).map_err(|_| BoundaryError("invalid UTF-8"))
    } else {
        Ok(unsafe { String::from_utf8_unchecked(data) })
    }
}
pub struct Anchor(String);
impl std::ops::Deref for Anchor {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}
impl crate::host::Borrowable for str {
    type A1 = usize;
    type A2 = usize;
    type Adopted = Option<Vec<u8>>;
    type Anchor = Anchor;
    #[inline(always)]
    unsafe fn adopt(p: usize, n: usize, cap: usize) -> Self::Adopted {
        Some(unsafe { crate::numeric::take_vec_parts(p, n, cap) })
    }
    unsafe fn anchor(_: usize, _: usize, _: usize, data: Self::Adopted) -> BoundaryResult<Anchor> {
        Ok(Anchor(string(data.unwrap())?))
    }
    unsafe fn with_shared<R>(
        _: usize,
        _: usize,
        _: usize,
        data: Self::Adopted,
        f: impl FnOnce(&str) -> R,
    ) -> BoundaryResult<R> {
        let value = string(data.unwrap())?;
        Ok(f(&value))
    }
}
fn layout(cap: usize) -> BoundaryResult<Layout> {
    if crate::PARANOID && cap > crate::MAX_BYTES {
        return Err(BoundaryError("string input size limit"));
    }
    Layout::array::<u8>(cap).map_err(|_| BoundaryError("string input size overflow"))
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_string_alloc(cap: usize) -> usize {
    crate::boundary(layout(cap).and_then(|_| crate::numeric::allocate_raw::<u8>(cap)))
}
/// # Safety
/// Pointer/capacity refer to an untransferred allocation from string_alloc.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_realloc(p: usize, old: usize, new: usize) -> usize {
    crate::boundary((|| {
        let new_layout = layout(new)?;
        let old_layout = layout(old)?;
        if new == old {
            return Ok(p);
        }
        // Generated encoders only grow/shrink nonempty allocations.
        if old == 0 || new == 0 {
            return Err(BoundaryError("invalid empty string reallocation"));
        }
        let pointer = unsafe { realloc(p as *mut u8, old_layout, new) };
        if pointer.is_null() {
            handle_alloc_error(new_layout);
        }
        Ok(pointer as usize)
    })())
}
/// # Safety
/// Release an allocation only before ownership has transferred, with its exact capacity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_free(p: usize, cap: usize) {
    // No uninitialized payload is read when releasing preparation storage.
    drop(unsafe { crate::numeric::take_vec_parts::<u8>(p, 0, cap) });
}

/// Root strings reuse their existing allocation, with no output packet.
pub struct Output;
thread_local! {static RESULT: std::cell::RefCell<[usize;4]> = const {std::cell::RefCell::new([0;4])};}
thread_local! {static RESULT_TAG: std::cell::RefCell<[usize;2]> = const {std::cell::RefCell::new([0;2])};}
impl crate::abi::ReturnValue<String> for Output {
    fn complete(value: String) -> BoundaryResult<crate::abi::Completion> {
        Ok(crate::abi::Completion::String(value))
    }
    fn complete_result<E: crate::Encode>(
        value: Result<String, E>,
    ) -> BoundaryResult<crate::abi::Completion> {
        match value {
            Ok(value) => Ok(crate::abi::Completion::StringResult(Ok(value))),
            Err(error) => error.complete_string_error(),
        }
    }
    fn lower_result<E: crate::Encode>(value: Result<String, E>) -> BoundaryResult<u32> {
        let tagged = match value {
            Ok(text) => [0, Self::lower(text)? as usize],
            Err(error) => [1, crate::publish(Err::<String, E>(error))? as usize],
        };
        Ok(RESULT_TAG.with_borrow_mut(|result| {
            *result = tagged;
            result.as_ptr() as usize as u32
        }))
    }
    fn lower(mut value: String) -> BoundaryResult<u32> {
        if crate::PARANOID && value.len() > crate::MAX_BYTES {
            return Err(BoundaryError("encoded value exceeds ABI limit"));
        }
        // SIMD classification selects the host constructor without UTF-8 decoding.
        let kind = if value.len() >= 256 {
            unsafe { __ocamljs_string_kind(value.as_ptr() as usize, value.len()) }
        } else {
            0
        };
        let info = [
            value.as_mut_ptr() as usize,
            value.len(),
            value.capacity(),
            kind as usize,
        ];
        std::mem::forget(value);
        crate::numeric::retained();
        Ok(RESULT.with_borrow_mut(|result| {
            *result = info;
            result.as_ptr() as usize as u32
        }))
    }
}

/// Constructor selection for native byte strings, not UTF-8 validation.
/// # Safety
/// Pointer/length describe initialized storage still owned by the string result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_ascii(p: usize, n: usize) -> u32 {
    let data = p as *const u8;
    let mut at = 0;
    while at + 16 <= n {
        let flags = unsafe {
            std::ptr::read_unaligned(data.add(at) as *const u32)
                | std::ptr::read_unaligned(data.add(at + 4) as *const u32)
                | std::ptr::read_unaligned(data.add(at + 8) as *const u32)
                | std::ptr::read_unaligned(data.add(at + 12) as *const u32)
        };
        if flags & 0x80808080 != 0 {
            return 0;
        }
        at += 16;
    }
    while at < n {
        if unsafe { *data.add(at) } & 128 != 0 {
            return 0;
        }
        at += 1;
    }
    1
}

/// Classify valid UTF-8 as ASCII (0), Latin-1 characters (1), or general Unicode (2).
/// This selects a constructor; it does not validate UTF-8.
/// # Safety
/// The initialized range belongs to a live valid UTF-8 string. A prefix may be
/// inspected; classes 0/1 then describe that prefix, not the uninspected suffix.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_kind(p: usize, n: usize) -> u32 {
    let data = p as *const u8;
    let mut at = 0;
    let mut non_ascii = false;
    while at + 16 <= n {
        let [a, b, c, d] = unsafe {
            [
                std::ptr::read_unaligned(data.add(at).cast::<u32>()),
                std::ptr::read_unaligned(data.add(at + 4).cast::<u32>()),
                std::ptr::read_unaligned(data.add(at + 8).cast::<u32>()),
                std::ptr::read_unaligned(data.add(at + 12).cast::<u32>()),
            ]
        };
        if (a | b | c | d) & 0x80808080 != 0 {
            // Each byte's low seven bits plus 60 cannot carry into its neighbour.
            // With its original high bit, a set high bit identifies byte >= 0xC4.
            let high = |word: u32| (word & 0x7f7f7f7f).wrapping_add(0x3c3c3c3c) & word;
            if (high(a) | high(b) | high(c) | high(d)) & 0x80808080 != 0 {
                return 2;
            }
            non_ascii = true;
        }
        at += 16;
    }
    while at < n {
        let byte = unsafe { *data.add(at) };
        if byte >= 0xc4 {
            return 2;
        }
        non_ascii |= byte >= 128;
        at += 1;
    }
    u32::from(non_ascii)
}

/// SIMD classification reduces byte maxima; valid UTF-8 needs no decoding here.
/// # Safety
/// The range is initialized storage from a live valid UTF-8 string.
#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_kind(p: usize, n: usize) -> u32 {
    use std::arch::wasm32::*;
    let data = p as *const u8;
    let mut at = 0;
    let (mut a, mut b, mut c, mut d) = (
        u8x16_splat(0),
        u8x16_splat(0),
        u8x16_splat(0),
        u8x16_splat(0),
    );
    while at + 64 <= n {
        unsafe {
            a = u8x16_max(a, v128_load(data.add(at).cast()));
            b = u8x16_max(b, v128_load(data.add(at + 16).cast()));
            c = u8x16_max(c, v128_load(data.add(at + 32).cast()));
            d = u8x16_max(d, v128_load(data.add(at + 48).cast()));
        }
        at += 64;
    }
    let mut maximum = u8x16_max(u8x16_max(a, b), u8x16_max(c, d));
    while at + 16 <= n {
        maximum = u8x16_max(maximum, unsafe { v128_load(data.add(at).cast()) });
        at += 16;
    }
    if v128_any_true(u8x16_ge(maximum, u8x16_splat(0xc4))) {
        return 2;
    }
    let mut non_ascii = i8x16_bitmask(maximum) != 0;
    while at < n {
        let byte = unsafe { *data.add(at) };
        if byte >= 0xc4 {
            return 2;
        }
        non_ascii |= byte >= 128;
        at += 1;
    }
    u32::from(non_ascii)
}

#[cfg(target_arch = "wasm32")]
#[repr(align(8))]
struct ShuffleTable([[u8; 8]; 256]);
#[cfg(target_arch = "wasm32")]
static LATIN1_SHUFFLE: ShuffleTable = ShuffleTable({
    let mut table = [[0xff; 8]; 256];
    let mut mask = 0;
    while mask < 256 {
        let (mut lane, mut output) = (0, 0);
        while lane < 8 {
            if mask & (1 << lane) == 0 {
                table[mask][output] = lane as u8;
                output += 1;
            }
            lane += 1;
        }
        mask += 1;
    }
    table
});

/// Compact TextEncoder output for byte-valued characters, in place.
/// # Safety
/// The initialized range is TextEncoder output for characters in 0..=255,
/// within a live, untransferred string allocation. The range may be overwritten.
#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_compact_latin1(p: usize, n: usize) -> usize {
    use std::arch::wasm32::*;
    let data = p as *mut u8;
    let (mut read, mut write, mut carry) = (0, 0, 0u8);
    while read + 16 <= n {
        let value = unsafe { v128_load(data.add(read).cast()) };
        let previous = i8x16_shuffle::<16, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14>(
            value,
            i8x16_splat(0),
        );
        let previous = i8x16_replace_lane::<0>(previous, carry as i8);
        carry = u8x16_extract_lane::<15>(value);
        let decoded = v128_or(
            value,
            v128_and(i8x16_eq(previous, u8x16_splat(0xc3)), u8x16_splat(0x40)),
        );
        let leads = i8x16_bitmask(u8x16_ge(value, u8x16_splat(0xc0)));
        let low_mask = (leads & 255) as usize;
        let high_mask = (leads >> 8) as usize;
        let low_control = unsafe { v128_load64_zero(LATIN1_SHUFFLE.0[low_mask].as_ptr().cast()) };
        let high_control = unsafe { v128_load64_zero(LATIN1_SHUFFLE.0[high_mask].as_ptr().cast()) };
        let high =
            i8x16_shuffle::<8, 9, 10, 11, 12, 13, 14, 15, 0, 0, 0, 0, 0, 0, 0, 0>(decoded, decoded);
        let low = i8x16_swizzle(decoded, low_control);
        let high = i8x16_swizzle(high, high_control);
        let low_count = 8 - low_mask.count_ones() as usize;
        // Both stores stay within this loaded 16-byte range or the already
        // consumed prefix. The second overwrites unused trailing lanes of the first.
        unsafe {
            v128_store64_lane::<0>(low, data.add(write).cast());
            v128_store64_lane::<0>(high, data.add(write + low_count).cast());
        }
        read += 16;
        write += 16 - leads.count_ones() as usize;
    }
    if read < n && carry >= 0xc0 {
        let byte = unsafe { *data.add(read) } | if carry == 0xc3 { 0x40 } else { 0 };
        unsafe { *data.add(write) = byte };
        read += 1;
        write += 1;
    }
    while read < n {
        let first = unsafe { *data.add(read) };
        let byte = if first < 128 {
            read += 1;
            first
        } else {
            let second = unsafe { *data.add(read + 1) };
            read += 2;
            (first & 3) << 6 | second & 63
        };
        unsafe { *data.add(write) = byte };
        write += 1;
    }
    write
}

#[cfg(not(target_arch = "wasm32"))]
/// # Safety
/// The initialized range encodes byte-valued characters and may be overwritten.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_compact_latin1(p: usize, n: usize) -> usize {
    let data = p as *mut u8;
    let (mut read, mut write) = (0, 0);
    while read < n {
        let first = unsafe { *data.add(read) };
        let byte = if first < 128 {
            read += 1;
            first
        } else {
            let second = unsafe { *data.add(read + 1) };
            read += 2;
            (first & 3) << 6 | second & 63
        };
        unsafe { *data.add(write) = byte };
        write += 1;
    }
    write
}

/// Compact and shrink preparation storage without another JS round trip.
/// # Safety
/// The allocation belongs to string_alloc. Its exact capacity `cap` is twice
/// the byte-valued character count; `n` is their initialized TextEncoder output.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_string_finish_latin1(p: usize, n: usize, cap: usize) -> usize {
    let initialized = if n == cap / 2 {
        // Every character was ASCII; TextEncoder already copied the exact bytes.
        n
    } else {
        unsafe { __ocamljs_string_compact_latin1(p, n) }
    };
    unsafe { __ocamljs_string_realloc(p, cap, initialized) }
}

pub(crate) fn complete_result(value: Result<String, u32>) -> BoundaryResult<u32> {
    use crate::abi::ReturnValue;
    let tagged = match value {
        Ok(value) => [0, Output::lower(value)? as usize],
        Err(id) => [1, id as usize],
    };
    Ok(RESULT_TAG.with_borrow_mut(|result| {
        *result = tagged;
        result.as_ptr() as usize as u32
    }))
}

pub(crate) fn complete_scalar_error(pointer: u32) -> BoundaryResult<u32> {
    Ok(RESULT_TAG.with_borrow_mut(|result| {
        *result = [2, pointer as usize];
        result.as_ptr() as usize as u32
    }))
}
