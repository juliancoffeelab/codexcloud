//! First-principles WebAssembly ABI for OCaml JavaScript compilers.
//! Values use a strict little-endian codec. Scalar calls and opaque references
//! bypass the codec. Opaque pointers are accepted only under the matching generated ownership contract.
pub use ocamljs_bindgen_macro::{EncodeProjection, ocamljs_bindgen};
#[doc(hidden)]
pub mod abi;
#[doc(hidden)]
pub mod numeric;
#[doc(hidden)]
pub mod objects;
pub mod text;
pub use objects::{dispose, insert, with_mut, with_ref};
mod callback;
pub mod fetch;
pub mod host;
pub use host::HostError;
pub mod scoped;
pub mod tasks;
pub mod views;
pub use callback::Callback;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};
pub use tasks::{spawn_scoped, yield_now};

pub const ABI_VERSION: u32 = 13;
/// Optional resource probes; independent of boundary validation and JS counters.
pub const RESOURCE_DIAGNOSTICS: bool = cfg!(any(
    not(target_arch = "wasm32"),
    feature = "resource-diagnostics",
    ocamljs_resource_diagnostics
));
/// Generated boundary checking; public byte parsers always remain checked.
pub const PARANOID: bool = cfg!(feature = "paranoid") || cfg!(ocamljs_paranoid);
pub const MAX_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ITEMS: usize = 1_000_000;
pub const MAX_DEPTH: usize = 128;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryError(pub &'static str);
pub type BoundaryResult<T> = Result<T, BoundaryError>;
impl std::fmt::Display for BoundaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for BoundaryError {}

pub struct Encoder {
    /// Complete inline data for `Encoder::new()`, or the structural body for a
    /// publication encoder. Use `is_publication()` when implementing custom codecs;
    /// encode strings through `Encode` rather than embedding inline packets.
    pub bytes: Vec<u8>,
    strings: Option<Vec<u8>>,
    handles: Option<HashSet<u32>>,
    error: Option<BoundaryError>,
    items: usize,
    depth: usize,
    checked: bool,
}
impl Encoder {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            strings: None,
            handles: None,
            error: None,
            items: MAX_ITEMS,
            depth: 0,
            checked: true,
        }
    }
    fn publication() -> Self {
        let mut encoder = Self::new();
        encoder.strings = Some(Vec::new());
        encoder.checked = PARANOID;
        encoder
    }
    /// Publication encoders place UTF-8 strings in a separate owned segment.
    /// Their `bytes` field alone is not an inline `decode` packet.
    pub fn is_publication(&self) -> bool {
        self.strings.is_some()
    }
    fn byte_len(&self) -> usize {
        self.bytes
            .len()
            .saturating_add(self.strings.as_ref().map_or(0, Vec::len))
    }
    fn string(&mut self, value: &str) {
        (value.len() as u32).encode(self);
        if self.strings.is_none() {
            self.push(value.as_bytes());
        } else if self.error.is_none() {
            if self.checked && self.byte_len().saturating_add(value.len()) > MAX_BYTES {
                self.fail("encoded value exceeds ABI limit");
            } else {
                self.strings
                    .as_mut()
                    .unwrap()
                    .extend_from_slice(value.as_bytes());
            }
        }
    }
    pub fn nested(&mut self, f: impl FnOnce(&mut Self)) {
        if !self.checked {
            return f(self);
        }
        if self.depth >= MAX_DEPTH {
            self.fail("nesting limit exceeded");
            return;
        }
        self.depth += 1;
        f(self);
        self.depth -= 1;
    }
    pub fn fail(&mut self, message: &'static str) {
        self.error.get_or_insert(BoundaryError(message));
    }
    pub fn items(&mut self, n: usize) -> bool {
        if !self.checked {
            return true;
        }
        if n > self.items {
            self.fail("collection budget exceeded");
            false
        } else {
            self.items -= n;
            true
        }
    }
    pub fn push(&mut self, bytes: &[u8]) {
        if self.error.is_some() {
            return;
        }
        if self.checked && self.byte_len().saturating_add(bytes.len()) > MAX_BYTES {
            self.fail("encoded value exceeds ABI limit");
            return;
        }
        self.bytes.extend_from_slice(bytes);
    }
    pub fn opaque<T: objects::Opaque>(&mut self, value: T) {
        if self.error.is_some() {
            return;
        }
        let id = insert(value);
        self.handles.get_or_insert_with(HashSet::new).insert(id);
        id.encode(self);
    }
}
impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for Encoder {
    fn drop(&mut self) {
        for id in self.handles.take().into_iter().flatten() {
            let _ = unsafe { dispose(id) };
        }
    }
}
pub struct Decoder<'a> {
    bytes: &'a [u8],
    strings: Option<&'a [u8]>,
    string_position: usize,
    position: usize,
    depth: usize,
    remaining_items: usize,
    checked: bool,
}
impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> BoundaryResult<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(BoundaryError("input exceeds ABI limit"));
        }
        Ok(Self {
            bytes,
            strings: None,
            string_position: 0,
            position: 0,
            depth: 0,
            remaining_items: MAX_ITEMS,
            checked: true,
        })
    }
    /// Decode ABI4 buffered output segments. Inline input remains `new(bytes)`.
    pub fn with_pool(bytes: &'a [u8], strings: &'a [u8]) -> BoundaryResult<Self> {
        if bytes.len().saturating_add(strings.len()) > MAX_BYTES {
            return Err(BoundaryError("input exceeds ABI limit"));
        }
        std::str::from_utf8(strings).map_err(|_| BoundaryError("invalid UTF-8"))?;
        let mut decoder = Self::new(bytes)?;
        decoder.strings = Some(strings);
        Ok(decoder)
    }
    fn string(&mut self, count: usize) -> BoundaryResult<&'a [u8]> {
        if let Some(strings) = self.strings {
            let end = self
                .string_position
                .checked_add(count)
                .ok_or(BoundaryError("length overflow"))?;
            let value = strings
                .get(self.string_position..end)
                .ok_or(BoundaryError("truncated string pool"))?;
            std::str::from_utf8(value).map_err(|_| BoundaryError("invalid UTF-8 boundary"))?;
            self.string_position = end;
            Ok(value)
        } else {
            self.take(count)
        }
    }
    pub fn take(&mut self, count: usize) -> BoundaryResult<&'a [u8]> {
        if !self.checked {
            let start = self.position;
            self.position += count;
            // Trusted generated writer and matching schema establish this range.
            return Ok(unsafe { self.bytes.get_unchecked(start..self.position) });
        }
        let end = self
            .position
            .checked_add(count)
            .ok_or(BoundaryError("length overflow"))?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(BoundaryError("truncated value"))?;
        self.position = end;
        Ok(value)
    }
    pub fn items(&mut self, n: usize) -> BoundaryResult<()> {
        if !self.checked {
            return Ok(());
        }
        if n > self.remaining_items {
            return Err(BoundaryError("collection budget exceeded"));
        }
        self.remaining_items -= n;
        Ok(())
    }
    pub fn length(&mut self) -> BoundaryResult<usize> {
        let n = u32::decode(self)? as usize;
        self.items(n)?;
        Ok(n)
    }
    pub fn nested<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> BoundaryResult<T>,
    ) -> BoundaryResult<T> {
        if !self.checked {
            return f(self);
        }
        if self.depth >= MAX_DEPTH {
            return Err(BoundaryError("nesting limit exceeded"));
        }
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        r
    }
    pub fn finish(self) -> BoundaryResult<()> {
        if !self.checked {
            return Ok(());
        }
        if self.position != self.bytes.len()
            || self
                .strings
                .is_some_and(|s| self.string_position != s.len())
        {
            Err(BoundaryError("trailing bytes"))
        } else {
            Ok(())
        }
    }
}
pub trait Encode: Sized {
    #[doc(hidden)]
    type Return: abi::ReturnValue<Self>;
    fn encode(self, out: &mut Encoder);
    #[doc(hidden)]
    fn complete(self) -> BoundaryResult<abi::Completion> {
        <Self::Return as abi::ReturnValue<Self>>::complete(self)
    }
    #[doc(hidden)]
    fn complete_string_error(self) -> BoundaryResult<abi::Completion> {
        publish(Err::<String, Self>(self)).map(|id| abi::Completion::StringResult(Err(id)))
    }
    #[doc(hidden)]
    fn encode_sequence(values: Vec<Self>, out: &mut Encoder) {
        for value in values {
            value.encode(out);
        }
    }
    #[doc(hidden)]
    fn encode_array<const N: usize>(values: [Self; N], out: &mut Encoder) {
        for value in values {
            value.encode(out);
        }
    }
}
/// Encode structural data without transferring or cloning its ownership.
/// Generate this codec explicitly with `#[ocamljs_bindgen(borrow)]` on a
/// structural record or enum. Every nested structural field must also opt in.
/// Opaque handles and host references retain their owning codecs; they cannot
/// opt in to borrowed data encoding. Callbacks keep their existing exported
/// argument acquisition path and may be retained inside opaque Rust objects.
pub trait EncodeRef {
    fn encode_ref(&self, out: &mut Encoder);
    #[doc(hidden)]
    fn encode_slice(values: &[Self], out: &mut Encoder)
    where
        Self: Sized,
    {
        for value in values {
            value.encode_ref(out);
        }
    }
}
impl<T: EncodeRef + ?Sized> Encode for &T {
    type Return = abi::Buffer<Self>;
    fn encode(self, out: &mut Encoder) {
        self.encode_ref(out);
    }
}
/// A compiler-checked projection of a public wire type.
pub trait EncodeAs<T> {
    fn encode_as(self, out: &mut Encoder);
}
/// A borrowed/exact-size iterator with the ordinary Vec<T> wire representation.
/// Indices can therefore project authoritative records directly, without an
/// intermediate collection of cloned values or borrowed references.
pub struct SequenceProjection<I>(pub I);
impl<T, I> EncodeAs<Vec<T>> for SequenceProjection<I>
where
    I: ExactSizeIterator,
    I::Item: EncodeAs<T>,
{
    fn encode_as(self, out: &mut Encoder) {
        if !out.items(self.0.len()) {
            return;
        }
        (self.0.len() as u32).encode(out);
        for value in self.0 {
            value.encode_as(out);
        }
    }
}
#[doc(hidden)]
pub struct EncodedReturn<T>(std::marker::PhantomData<fn() -> T>);
impl<T> abi::ReturnValue<Encoded<T>> for EncodedReturn<T> {
    fn lower(value: Encoded<T>) -> BoundaryResult<u32> {
        publish_encoder(value.encoder)
    }
}
/// Project a successful value while preserving the native Result error encoding.
pub struct ResultProjection<T, E>(pub Result<T, E>);
impl<T, P: EncodeAs<T>, E: Encode> EncodeAs<Result<T, E>> for ResultProjection<P, E> {
    fn encode_as(self, out: &mut Encoder) {
        match self.0 {
            Ok(v) => {
                0u8.encode(out);
                v.encode_as(out);
            }
            Err(e) => {
                1u8.encode(out);
                e.encode(out);
            }
        }
    }
}
/// An eagerly encoded, detached return value. Construction finishes all borrowed
/// reads; lowering transfers its body and UTF-8 pool into the ABI arena without
/// copying either segment. `Encoder::new()` remains an inline codec; publication
/// encoders use the segmented ABI4 output format.
/// Generated exports support only a complete synchronous
/// `Encoded<Result<Value, Error>>` return. Nested use fails encoding rather than
/// bypassing the surrounding encoder's depth or collection limits.
/// This value stays on its originating thread because transferred opaque handles
/// belong to that thread's arena.
pub struct Encoded<T> {
    encoder: Encoder,
    marker: std::marker::PhantomData<(fn() -> T, Rc<()>)>,
}
impl<T> Encoded<T> {
    /// Encode the typed projection before its borrowed sources leave scope.
    pub fn new(value: impl EncodeAs<T>) -> Self {
        let mut encoder = Encoder::publication();
        value.encode_as(&mut encoder);
        Self {
            encoder,
            marker: std::marker::PhantomData,
        }
    }
}
impl<T> Encode for Encoded<T> {
    type Return = EncodedReturn<T>;
    fn encode(self, out: &mut Encoder) {
        out.fail("Encoded must be the complete export return type");
    }
}
impl<T: Encode> EncodeAs<T> for T {
    fn encode_as(self, out: &mut Encoder) {
        self.encode(out);
    }
}
impl<T: EncodeRef> EncodeAs<T> for &T {
    fn encode_as(self, out: &mut Encoder) {
        self.encode_ref(out);
    }
}
pub trait Decode: Sized {
    #[doc(hidden)]
    type Abi: abi::Argument<Self>;
    /// # Safety
    /// The matching generated ABI must provide a fully initialized primitive
    /// allocation with exactly this element type, length and capacity. Only
    /// primitive implementations opt in; structural elements keep the codec.
    #[doc(hidden)]
    unsafe fn adopt_numeric_sequence(_: usize, _: usize) -> Option<Vec<Self>> {
        None
    }
    fn decode(input: &mut Decoder<'_>) -> BoundaryResult<Self>;
    #[doc(hidden)]
    fn decode_sequence(n: usize, input: &mut Decoder<'_>) -> BoundaryResult<Vec<Self>> {
        decode_elements(n, input)
    }
}
pub fn decode<T: Decode>(bytes: &[u8]) -> BoundaryResult<T> {
    let mut d = Decoder::new(bytes)?;
    let value = T::decode(&mut d)?;
    d.finish()?;
    Ok(value)
}
/// Decode both segments of an ABI4 buffered publication without assembling a copy.
pub fn decode_segments<T: Decode>(body: &[u8], strings: &[u8]) -> BoundaryResult<T> {
    let mut decoder = Decoder::with_pool(body, strings)?;
    let value = T::decode(&mut decoder)?;
    decoder.finish()?;
    Ok(value)
}
macro_rules! number { ($direct:tt; $($t:ty),*) => {$ (
 impl Encode for $t {
  type Return = abi::Buffer<Self>;
  fn encode(self, out: &mut Encoder) { out.push(&self.to_le_bytes()); }
  fn complete(self) -> BoundaryResult<abi::Completion> { Ok(abi::Completion::bytes(&self.to_le_bytes())) }
  fn complete_string_error(self) -> BoundaryResult<abi::Completion> { Ok(abi::Completion::string_error(&self.to_le_bytes())) }
  fn encode_sequence(values:Vec<Self>,out:&mut Encoder){ Self::encode_slice(&values,out); }
  fn encode_array<const N:usize>(values:[Self;N],out:&mut Encoder){Self::encode_slice(&values,out);}
 }
 impl EncodeRef for $t {
  fn encode_ref(&self,out:&mut Encoder){(*self).encode(out);}
  fn encode_slice(values:&[Self],out:&mut Encoder){
   if cfg!(target_endian="little") { unsafe { encode_numeric(values,out); } }
   else {for value in values {value.encode_ref(out);}}
  }
 }
 impl Decode for $t {
  type Abi=abi::Buffer<Self>;
  numeric_adopt!($direct);
  fn decode(input:&mut Decoder<'_>)->BoundaryResult<Self>{Ok(Self::from_le_bytes(input.take(size_of::<Self>())?.try_into().unwrap()))}
  fn decode_sequence(n:usize,input:&mut Decoder<'_>)->BoundaryResult<Vec<Self>>{
   if cfg!(target_endian="little") { unsafe {decode_numeric(n,input)} }else{decode_elements(n,input)}
  }
 }
)*}; }
macro_rules! numeric_adopt {
    (true) => {
        #[inline(always)]
        unsafe fn adopt_numeric_sequence(pointer: usize, len: usize) -> Option<Vec<Self>> {
            Some(unsafe { numeric::take_vec::<Self>(pointer, len) })
        }
    };
    (false) => {};
}
number!(true; u8, i8, u16, i16, u32, i32, f32, f64);
number!(false; u64, i64);
impl Encode for usize {
    type Return = abi::Buffer<Self>;
    fn complete(self) -> BoundaryResult<abi::Completion> {
        (u32::try_from(self).expect("usize exceeds wasm32")).complete()
    }
    fn complete_string_error(self) -> BoundaryResult<abi::Completion> {
        (u32::try_from(self).expect("usize exceeds wasm32")).complete_string_error()
    }
    fn encode(self, out: &mut Encoder) {
        u32::try_from(self)
            .expect("usize exceeds wasm32")
            .encode(out);
    }
}
impl Decode for usize {
    numeric_adopt!(true);
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        Ok(u32::decode(d)? as usize)
    }
}
impl Encode for isize {
    type Return = abi::Buffer<Self>;
    fn complete(self) -> BoundaryResult<abi::Completion> {
        (i32::try_from(self).expect("isize exceeds wasm32")).complete()
    }
    fn complete_string_error(self) -> BoundaryResult<abi::Completion> {
        (i32::try_from(self).expect("isize exceeds wasm32")).complete_string_error()
    }
    fn encode(self, out: &mut Encoder) {
        i32::try_from(self)
            .expect("isize exceeds wasm32")
            .encode(out);
    }
}
impl Decode for isize {
    numeric_adopt!(true);
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        Ok(i32::decode(d)? as isize)
    }
}
impl Encode for bool {
    type Return = abi::Buffer<Self>;
    fn complete(self) -> BoundaryResult<abi::Completion> {
        (u8::from(self)).complete()
    }
    fn complete_string_error(self) -> BoundaryResult<abi::Completion> {
        (u8::from(self)).complete_string_error()
    }
    fn encode(self, o: &mut Encoder) {
        u8::from(self).encode(o);
    }
}
impl Decode for bool {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        match u8::decode(d)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BoundaryError("invalid bool")),
        }
    }
}
impl Encode for () {
    type Return = abi::Buffer<Self>;
    fn complete(self) -> BoundaryResult<abi::Completion> {
        Ok(abi::Completion::bytes(&[]))
    }
    fn complete_string_error(self) -> BoundaryResult<abi::Completion> {
        Ok(abi::Completion::string_error(&[]))
    }
    fn encode(self, _: &mut Encoder) {}
}
impl Decode for () {
    type Abi = abi::Buffer<Self>;
    fn decode(_: &mut Decoder<'_>) -> BoundaryResult<Self> {
        Ok(())
    }
}
impl Encode for String {
    type Return = crate::text::Output;
    fn encode(self, o: &mut Encoder) {
        if o.strings.as_ref().is_some_and(Vec::is_empty)
            && self.capacity() <= self.len().saturating_mul(2)
            && self.capacity() <= MAX_BYTES
        {
            (self.len() as u32).encode(o);
            if o.error.is_none() {
                if o.checked && o.byte_len().saturating_add(self.len()) > MAX_BYTES {
                    o.fail("encoded value exceeds ABI limit");
                } else {
                    o.strings = Some(self.into_bytes());
                }
            }
        } else {
            o.string(&self);
        }
    }
}
impl EncodeRef for str {
    fn encode_ref(&self, out: &mut Encoder) {
        out.string(self);
    }
}
impl EncodeRef for String {
    fn encode_ref(&self, out: &mut Encoder) {
        self.as_str().encode_ref(out);
    }
}
macro_rules! borrowed_scalar { ($($t:ty),*) => { $(impl EncodeRef for $t { fn encode_ref(&self, out: &mut Encoder) { (*self).encode(out); } })* }; }
borrowed_scalar!(usize, isize, bool, ());
impl<T: EncodeRef> EncodeRef for Option<T> {
    fn encode_ref(&self, out: &mut Encoder) {
        match self {
            None => 0u8.encode(out),
            Some(v) => {
                1u8.encode(out);
                v.encode_ref(out);
            }
        }
    }
}
impl<T: EncodeRef, E: EncodeRef> EncodeRef for Result<T, E> {
    fn encode_ref(&self, out: &mut Encoder) {
        match self {
            Ok(v) => {
                0u8.encode(out);
                v.encode_ref(out);
            }
            Err(v) => {
                1u8.encode(out);
                v.encode_ref(out);
            }
        }
    }
}
impl<T: EncodeRef> EncodeRef for [T] {
    fn encode_ref(&self, out: &mut Encoder) {
        if !out.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(out);
        T::encode_slice(self, out);
    }
}
impl<T: EncodeRef> EncodeRef for Vec<T> {
    fn encode_ref(&self, out: &mut Encoder) {
        self.as_slice().encode_ref(out);
    }
}
impl<T: EncodeRef> EncodeRef for std::collections::LinkedList<T> {
    fn encode_ref(&self, out: &mut Encoder) {
        if !out.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(out);
        for value in self {
            value.encode_ref(out);
        }
    }
}
impl<T: Encode> Encode for std::collections::LinkedList<T> {
    type Return = abi::Buffer<Self>;
    fn encode(self, out: &mut Encoder) {
        if !out.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(out);
        for value in self {
            value.encode(out);
        }
    }
}
impl<T: Decode> Decode for std::collections::LinkedList<T> {
    type Abi = abi::Buffer<Self>;
    fn decode(decoder: &mut Decoder<'_>) -> BoundaryResult<Self> {
        decoder.nested(|decoder| {
            let length = decoder.length()?;
            let mut values = Self::new();
            for _ in 0..length {
                values.push_back(T::decode(decoder)?);
            }
            Ok(values)
        })
    }
}
impl<T: EncodeRef, const N: usize> EncodeRef for [T; N] {
    fn encode_ref(&self, out: &mut Encoder) {
        if !out.items(N) {
            return;
        }
        T::encode_slice(self, out);
    }
}
impl<T: EncodeRef + ?Sized> EncodeRef for &T {
    fn encode_ref(&self, out: &mut Encoder) {
        (**self).encode_ref(out);
    }
}
impl<T: EncodeRef + ?Sized> EncodeRef for Box<T> {
    fn encode_ref(&self, out: &mut Encoder) {
        (**self).encode_ref(out);
    }
}
impl<T: EncodeRef + ?Sized> EncodeRef for Rc<T> {
    fn encode_ref(&self, out: &mut Encoder) {
        (**self).encode_ref(out);
    }
}
impl<T: EncodeRef + ?Sized> EncodeRef for std::sync::Arc<T> {
    fn encode_ref(&self, out: &mut Encoder) {
        (**self).encode_ref(out);
    }
}
impl<T: EncodeRef> Encode for std::sync::Arc<T> {
    type Return = abi::Buffer<Self>;
    fn encode(self, out: &mut Encoder) {
        self.encode_ref(out);
    }
}
impl<T: Decode> Decode for std::sync::Arc<T> {
    type Abi = abi::Buffer<Self>;
    fn decode(input: &mut Decoder<'_>) -> BoundaryResult<Self> {
        Ok(Self::new(T::decode(input)?))
    }
}
impl<K: EncodeRef, V: EncodeRef> EncodeRef for BTreeMap<K, V> {
    fn encode_ref(&self, out: &mut Encoder) {
        if !out.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(out);
        for (k, v) in self {
            k.encode_ref(out);
            v.encode_ref(out);
        }
    }
}
impl<K: EncodeRef, V: EncodeRef, S> EncodeRef for HashMap<K, V, S> {
    fn encode_ref(&self, out: &mut Encoder) {
        if !out.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(out);
        for (k, v) in self {
            k.encode_ref(out);
            v.encode_ref(out);
        }
    }
}
impl Decode for String {
    type Abi = crate::text::Owned;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        let n = u32::decode(d)? as usize;
        let bytes = d.string(n)?.to_vec();
        if d.checked {
            String::from_utf8(bytes).map_err(|_| BoundaryError("invalid UTF-8"))
        } else {
            Ok(unsafe { String::from_utf8_unchecked(bytes) })
        }
    }
}
impl<T: Encode> Encode for Option<T> {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        match self {
            None => 0u8.encode(o),
            Some(v) => {
                1u8.encode(o);
                v.encode(o);
            }
        }
    }
}
impl<T: Decode> Decode for Option<T> {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| match u8::decode(d)? {
            0 => Ok(None),
            1 => Ok(Some(T::decode(d)?)),
            _ => Err(BoundaryError("invalid option tag")),
        })
    }
}
impl<T: Encode, E: Encode> Encode for Result<T, E> {
    type Return = abi::ResultOutput<T, E>;
    fn encode(self, o: &mut Encoder) {
        match self {
            Ok(v) => {
                0u8.encode(o);
                v.encode(o);
            }
            Err(e) => {
                1u8.encode(o);
                e.encode(o);
            }
        }
    }
}
impl<T: Decode, E: Decode> Decode for Result<T, E> {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| match u8::decode(d)? {
            0 => Ok(Ok(T::decode(d)?)),
            1 => Ok(Err(E::decode(d)?)),
            _ => Err(BoundaryError("invalid result tag")),
        })
    }
}
// Only the built-in numeric implementations above call these private helpers.
// Their types have no padding, all bit patterns are valid, and the target is LE.
unsafe fn encode_numeric<T>(values: &[T], out: &mut Encoder) {
    out.push(unsafe {
        std::slice::from_raw_parts(values.as_ptr().cast(), std::mem::size_of_val(values))
    });
}
unsafe fn decode_numeric<T>(n: usize, input: &mut Decoder<'_>) -> BoundaryResult<Vec<T>> {
    let bytes = input.take(
        n.checked_mul(size_of::<T>())
            .ok_or(BoundaryError("length overflow"))?,
    )?;
    let mut values = Vec::<T>::with_capacity(n);
    // Byte copying supports unaligned wire spans and returns owned aligned storage.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), values.as_mut_ptr().cast(), bytes.len());
        values.set_len(n);
    }
    Ok(values)
}
fn decode_elements<T: Decode>(n: usize, input: &mut Decoder<'_>) -> BoundaryResult<Vec<T>> {
    let mut values = Vec::with_capacity(if input.checked { n.min(4096) } else { n });
    for _ in 0..n {
        values.push(T::decode(input)?);
    }
    Ok(values)
}
impl<T: Encode> Encode for Vec<T> {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        if !o.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(o);
        T::encode_sequence(self, o);
    }
}
impl<T: Decode> Decode for Vec<T> {
    type Abi = abi::Vector<T>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| {
            let n = d.length()?;
            T::decode_sequence(n, d)
        })
    }
}
impl<T: Encode, const N: usize> Encode for [T; N] {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        if !o.items(N) {
            return;
        }
        T::encode_array(self, o);
    }
}
impl<T: Decode, const N: usize> Decode for [T; N] {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| {
            d.items(N)?;
            T::decode_sequence(N, d)?
                .try_into()
                .map_err(|_| BoundaryError("invalid array length"))
        })
    }
}
impl<T: Encode> Encode for Box<T> {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        (*self).encode(o);
    }
}
impl<T: Decode> Decode for Box<T> {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| Ok(Box::new(T::decode(d)?)))
    }
}
impl<T: Encode + Clone> Encode for Rc<T> {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        match Rc::try_unwrap(self) {
            Ok(v) => v.encode(o),
            Err(v) => (*v).clone().encode(o),
        }
    }
}
impl<T: Decode> Decode for Rc<T> {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        Ok(Rc::new(T::decode(d)?))
    }
}
macro_rules! tuple { ($($t:ident:$i:tt),*) => {
 impl<$($t: Encode),*> Encode for ($($t,)*) { type Return = abi::Buffer<Self>; fn encode(self,o:&mut Encoder) { $(self.$i.encode(o);)* } }
 impl<$($t: EncodeRef),*> EncodeRef for ($($t,)*) { fn encode_ref(&self,o:&mut Encoder) { $(self.$i.encode_ref(o);)* } }
 impl<$($t: Decode),*> Decode for ($($t,)*) { type Abi = abi::Buffer<Self>; fn decode(d:&mut Decoder<'_>)->BoundaryResult<Self> { d.nested(|d| Ok(($($t::decode(d)?,)*))) } }
}; }
tuple!(A:0);
tuple!(A:0,B:1);
tuple!(A:0,B:1,C:2);
tuple!(A:0,B:1,C:2,D:3);
impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        if !o.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(o);
        for (k, v) in self {
            k.encode(o);
            v.encode(o);
        }
    }
}
impl<K: Decode + Ord, V: Decode> Decode for BTreeMap<K, V> {
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| {
            let n = d.length()?;
            let mut m = Self::new();
            for _ in 0..n {
                let k = K::decode(d)?;
                let v = V::decode(d)?;
                if m.insert(k, v).is_some() && d.checked {
                    return Err(BoundaryError("duplicate map key"));
                }
            }
            Ok(m)
        })
    }
}
impl<K: Encode, V: Encode, S> Encode for HashMap<K, V, S> {
    type Return = abi::Buffer<Self>;
    fn encode(self, o: &mut Encoder) {
        if !o.items(self.len()) {
            return;
        }
        (self.len() as u32).encode(o);
        for (k, v) in self {
            k.encode(o);
            v.encode(o);
        }
    }
}
impl<K: Decode + Eq + std::hash::Hash, V: Decode, S: std::hash::BuildHasher + Default> Decode
    for HashMap<K, V, S>
{
    type Abi = abi::Buffer<Self>;
    fn decode(d: &mut Decoder<'_>) -> BoundaryResult<Self> {
        d.nested(|d| {
            let n = d.length()?;
            let mut m = Self::default();
            for _ in 0..n {
                let k = K::decode(d)?;
                let v = V::decode(d)?;
                if m.insert(k, v).is_some() && d.checked {
                    return Err(BoundaryError("duplicate map key"));
                }
            }
            Ok(m)
        })
    }
}

// Transient buffer tokens never escape generated call scopes. Recycle slots so
// data traffic does not consume object generations or a monotonically growing ID.
struct Pool<T> {
    slots: Vec<Option<T>>,
    free: Vec<usize>,
}
impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self {
            slots: vec![],
            free: vec![],
        }
    }
}
impl<T> Pool<T> {
    fn put(&mut self, value: T) -> u32 {
        let i = self.free.pop().unwrap_or_else(|| {
            self.slots.push(None);
            self.slots.len() - 1
        });
        self.slots[i] = Some(value);
        u32::try_from(i + 1).expect("buffer arena exhausted")
    }
    fn get(&self, id: u32) -> Option<&T> {
        self.slots.get(id.checked_sub(1)? as usize)?.as_ref()
    }
    fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        self.slots.get_mut(id.checked_sub(1)? as usize)?.as_mut()
    }
    fn remove(&mut self, id: u32) -> Option<T> {
        let i = id.checked_sub(1)? as usize;
        let v = self.slots.get_mut(i)?.take()?;
        self.free.push(i);
        Some(v)
    }
    fn len(&self) -> usize {
        self.slots.len() - self.free.len()
    }
}
struct Output {
    bytes: Vec<u8>,
    strings: Vec<u8>,
    info: [usize; 4],
    handles: Option<HashSet<u32>>,
}
#[derive(Default)]
struct Arena {
    inputs: Pool<Rc<Vec<u8>>>,
    outputs: Pool<Output>,
}
thread_local! { static ARENA: RefCell<Arena> = RefCell::new(Arena::default()); }
pub(crate) fn input_owner(id: u32) -> BoundaryResult<Rc<Vec<u8>>> {
    ARENA.with_borrow(|a| {
        a.inputs
            .get(id)
            .cloned()
            .ok_or(BoundaryError("invalid input buffer"))
    })
}
#[cfg(any(not(target_arch = "wasm32"), feature = "paranoid", ocamljs_paranoid))]
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_contains(id: u32) -> u32 {
    objects::contains(id) as u32
}
#[doc(hidden)]
pub unsafe fn with_shared<T: host::Borrowable<A1 = (), A2 = (), Adopted = ()>, R>(
    id: u32,
    f: impl FnOnce(&T) -> R,
) -> BoundaryResult<R> {
    unsafe { T::with_shared(id as usize, (), (), (), f) }
}
pub(crate) fn output_owns(output: u32, id: u32) -> bool {
    ARENA.with_borrow(|a| {
        a.outputs.get(output).is_some_and(|o| {
            o.handles
                .as_ref()
                .is_some_and(|handles| handles.contains(&id))
        })
    })
}
pub fn input<T: Decode>(id: u32) -> BoundaryResult<T> {
    with_bytes(id, decode)?
}
/// Generated ABI only: bytes must have been written by the matching typed codec.
/// # Safety
/// The input must obey the schema, including UTF-8, tags, lengths and ranges.
#[doc(hidden)]
pub unsafe fn input_trusted<T: Decode>(id: u32) -> BoundaryResult<T> {
    with_bytes(id, |bytes| {
        let mut decoder = if PARANOID {
            Decoder::new(bytes)?
        } else {
            Decoder {
                bytes,
                strings: None,
                string_position: 0,
                position: 0,
                depth: 0,
                remaining_items: usize::MAX,
                checked: false,
            }
        };
        let result = T::decode(&mut decoder)?;
        decoder.finish()?;
        Ok(result)
    })?
}
/// # Safety
/// The bytes must be canonical UTF-8 from the generated string writer.
#[doc(hidden)]
pub unsafe fn boundary_str(bytes: &[u8]) -> BoundaryResult<&str> {
    if PARANOID {
        std::str::from_utf8(bytes).map_err(|_| BoundaryError("invalid UTF-8"))
    } else {
        Ok(unsafe { std::str::from_utf8_unchecked(bytes) })
    }
}
pub fn with_bytes<R>(id: u32, f: impl FnOnce(&[u8]) -> R) -> BoundaryResult<R> {
    let bytes = ARENA.with_borrow(|a| {
        a.inputs
            .get(id)
            .cloned()
            .ok_or(BoundaryError("invalid input buffer"))
    })?;
    Ok(f(&bytes))
}
pub fn publish<T: Encode>(v: T) -> BoundaryResult<u32> {
    let mut e = Encoder::publication();
    v.encode(&mut e);
    publish_encoder(e)
}
fn publish_encoder(mut e: Encoder) -> BoundaryResult<u32> {
    if let Some(error) = e.error.take() {
        return Err(error);
    }
    let bytes = std::mem::take(&mut e.bytes);
    let strings = e.strings.take().unwrap_or_default();
    let info = [
        bytes.as_ptr() as usize,
        bytes.len(),
        strings.as_ptr() as usize,
        strings.len(),
    ];
    let handles = std::mem::take(&mut e.handles);
    Ok(ARENA.with_borrow_mut(|a| {
        a.outputs.put(Output {
            bytes,
            strings,
            info,
            handles,
        })
    }))
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "ocamljs")]
unsafe extern "C" {
    fn boundary_error(ptr: usize, len: usize);
}
pub fn boundary<T>(v: BoundaryResult<T>) -> T {
    match v {
        Ok(v) => v,
        Err(error) => {
            #[cfg(target_arch = "wasm32")]
            unsafe {
                boundary_error(error.0.as_ptr() as usize, error.0.len());
            }
            panic!("OCamlJS boundary: {error}")
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_abi() -> u32 {
    ABI_VERSION
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_paranoid() -> u32 {
    PARANOID as u32
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_alloc(len: u32) -> u32 {
    if PARANOID {
        assert!(len as usize <= MAX_BYTES);
    }
    ARENA.with_borrow_mut(|a| a.inputs.put(Rc::new(vec![0; len as usize])))
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_input_ptr(id: u32) -> usize {
    ARENA.with_borrow(|a| a.inputs.get(id).expect("invalid input buffer").as_ptr() as usize)
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_input_truncate(id: u32, len: u32) {
    ARENA.with_borrow_mut(|arena| {
        let value = Rc::get_mut(arena.inputs.get_mut(id).expect("invalid input buffer"))
            .expect("input preparation buffer is shared");
        if PARANOID {
            assert!(len as usize <= value.len());
        }
        value.truncate(len as usize);
    });
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_input_free(id: u32) {
    ARENA.with_borrow_mut(|a| {
        a.inputs.remove(id);
    });
}
#[unsafe(no_mangle)]
/// Borrow four native usize words: body pointer/length, pool pointer/length.
/// Copy all four words before another Rust call: output-slab growth can move
/// this inline array even though the two segment allocations remain stable.
pub extern "C" fn __ocamljs_output_info(id: u32) -> usize {
    ARENA.with_borrow(|a| {
        a.outputs
            .get(id)
            .expect("invalid output buffer")
            .info
            .as_ptr() as usize
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_output_ptr(id: u32) -> usize {
    ARENA.with_borrow(|a| {
        a.outputs
            .get(id)
            .expect("invalid output buffer")
            .bytes
            .as_ptr() as usize
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_output_len(id: u32) -> usize {
    ARENA.with_borrow(|a| {
        a.outputs
            .get(id)
            .expect("invalid output buffer")
            .bytes
            .len()
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_output_pool_ptr(id: u32) -> usize {
    ARENA.with_borrow(|a| {
        a.outputs
            .get(id)
            .expect("invalid output buffer")
            .strings
            .as_ptr() as usize
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_output_pool_len(id: u32) -> usize {
    ARENA.with_borrow(|a| {
        a.outputs
            .get(id)
            .expect("invalid output buffer")
            .strings
            .len()
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_claim(output: u32, handle: u32) -> u32 {
    ARENA.with_borrow_mut(|a| {
        let o = a.outputs.get_mut(output).expect("invalid output buffer");
        assert!(
            o.handles
                .as_mut()
                .is_some_and(|handles| handles.remove(&handle)),
            "unowned output handle"
        );
        handle
    })
}
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_output_free(id: u32) {
    let output = ARENA.with_borrow_mut(|a| a.outputs.remove(id));
    if let Some(o) = output {
        for h in o.handles.into_iter().flatten() {
            let _ = unsafe { dispose(h) };
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn __ocamljs_dispose(id: u32) -> u32 {
    u32::from(unsafe { dispose(id) }.is_ok())
}
#[cfg(any(
    not(target_arch = "wasm32"),
    feature = "resource-diagnostics",
    ocamljs_resource_diagnostics
))]
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_resource_diagnostics() -> u32 {
    1
}
#[cfg(any(
    not(target_arch = "wasm32"),
    feature = "resource-diagnostics",
    ocamljs_resource_diagnostics
))]
#[unsafe(no_mangle)]
pub extern "C" fn __ocamljs_live() -> usize {
    ARENA.with_borrow(|a| a.inputs.len() + a.outputs.len() + objects::live() + numeric::pending())
}
