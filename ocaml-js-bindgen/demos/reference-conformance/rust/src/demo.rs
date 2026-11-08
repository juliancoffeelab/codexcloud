use ocamljs_bindgen::{Callback, HostError, host::Promise, ocamljs_bindgen};

#[ocamljs_bindgen(host)]
/// A JavaScript object whose identity is preserved at the Wasm boundary.
pub struct Object;

#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_namespace = "Object", js_name = "is")]
    pub fn object_is(a: &Object, b: &Object) -> Result<bool, HostError>;
    #[ocamljs_bindgen(js_namespace = "JSON", js_name = "parse")]
    pub fn parse(text: &str) -> Result<Object, HostError>;
}
#[ocamljs_bindgen]
pub fn identity(value: Object) -> Object {
    value
}
#[ocamljs_bindgen]
pub fn same(a: &Object, b: &Object) -> Result<bool, HostError> {
    object_is(a, b)
}
#[ocamljs_bindgen]
pub fn parse_json(text: &str) -> Result<Object, HostError> {
    parse(text)
}

#[ocamljs_bindgen(opaque)]
pub struct Holder {
    value: Object,
}
#[ocamljs_bindgen]
impl Holder {
    pub fn create(value: Object) -> Self {
        Self { value }
    }
    pub fn value(&self) -> Object {
        self.value.clone()
    }
}

#[ocamljs_bindgen(host)]
pub struct ArrayBuffer;
#[ocamljs_bindgen(host)]
pub struct ByteArray;
#[ocamljs_bindgen(host)]
pub struct LibraryCounter;
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(constructor, js_name = "ArrayBuffer")]
    fn new_buffer(size: u32) -> Result<ArrayBuffer, HostError>;
    #[ocamljs_bindgen(constructor, js_name = "Uint8Array")]
    fn new_bytes(buffer: &ArrayBuffer) -> Result<ByteArray, HostError>;
    #[ocamljs_bindgen(method, js_name = "fill")]
    fn fill(bytes: &ByteArray, value: u8) -> Result<ByteArray, HostError>;
    #[ocamljs_bindgen(indexing_getter)]
    fn byte_at(bytes: &ByteArray, index: u32) -> Result<u8, HostError>;
    #[ocamljs_bindgen(js_namespace = "Promise", js_name = "resolve")]
    fn resolve(value: &Object) -> Result<Promise<Object>, HostError>;
}
#[ocamljs_bindgen(js_module = "./third-party.mjs")]
unsafe extern "C" {
    #[ocamljs_bindgen(constructor, js_name = "Counter")]
    fn new_counter(value: f64) -> Result<LibraryCounter, HostError>;
    #[ocamljs_bindgen(method, js_name = "add")]
    fn counter_add(counter: &LibraryCounter, value: f64) -> Result<f64, HostError>;
    #[ocamljs_bindgen(js_name = "failure")]
    fn failure() -> Result<Object, HostError>;
}
#[ocamljs_bindgen]
pub fn buffer(size: u32) -> Result<ArrayBuffer, HostError> {
    new_buffer(size)
}
#[ocamljs_bindgen]
pub fn bytes(buffer: &ArrayBuffer) -> Result<ByteArray, HostError> {
    new_bytes(buffer)
}
#[ocamljs_bindgen]
pub fn paint_bytes(bytes: &ByteArray, value: u8) -> Result<ByteArray, HostError> {
    fill(bytes, value)
}
#[ocamljs_bindgen]
pub fn read_byte(bytes: &ByteArray, index: u32) -> Result<u8, HostError> {
    byte_at(bytes, index)
}
#[ocamljs_bindgen]
pub fn library_counter(value: f64) -> Result<LibraryCounter, HostError> {
    new_counter(value)
}
#[ocamljs_bindgen]
pub fn increment(counter: &LibraryCounter, value: f64) -> Result<f64, HostError> {
    counter_add(counter, value)
}
#[ocamljs_bindgen]
pub fn fail() -> Result<Object, HostError> {
    failure()
}
#[ocamljs_bindgen]
pub fn native_promise(value: &Object) -> Result<Promise<Object>, HostError> {
    resolve(value)
}
#[ocamljs_bindgen]
pub fn promise_identity(value: Promise<Object>) -> Promise<Object> {
    value
}
#[ocamljs_bindgen]
pub async fn await_object(value: Promise<Object>) -> Result<Object, HostError> {
    value.await
}
#[ocamljs_bindgen]
pub async fn await_borrowed(value: &Promise<Object>) -> Result<Object, HostError> {
    value.clone().await
}
#[ocamljsify::ocamljsify]
pub struct HostEvent {
    pub value: Object,
    pub label: String,
    pub sequence: u32,
}
#[ocamljs_bindgen]
pub fn notify(value: &Object, handler: Callback<HostEvent>) {
    let _ = handler.emit(HostEvent {
        value: value.clone(),
        label: "\u{feff}Host λ🦀".into(),
        sequence: 1,
    });
}
#[ocamljs_bindgen]
pub async fn notify_after(
    value: Promise<Object>,
    handler: Callback<HostEvent>,
) -> Result<Object, HostError> {
    let value = value.await?;
    let _ = handler.emit(HostEvent {
        value: value.clone(),
        label: "\u{feff}Host λ🦀".into(),
        sequence: 2,
    });
    Ok(value)
}

#[ocamljs_bindgen]
pub type ObjectAlias = Object;
#[ocamljs_bindgen]
pub type Pending = Promise<Object>;
#[ocamljs_bindgen]
pub fn alias_identity(value: ObjectAlias) -> ObjectAlias {
    value
}
#[ocamljs_bindgen]
pub fn alias_promise(value: Pending) -> Pending {
    value
}
#[ocamljs_bindgen(js_module = "./third-party.mjs")]
unsafe extern "C" {
    #[ocamljs_bindgen(js_name = "unsigned")]
    fn check_unsigned(a: u32, b: u64) -> Result<u64, HostError>;
}
#[ocamljs_bindgen]
pub fn unsigned() -> Result<u64, HostError> {
    check_unsigned(u32::MAX, u64::MAX)
}

#[ocamljs_bindgen]
pub async fn await_delayed(value: Promise<Object>) -> Result<Object, HostError> {
    ocamljs_bindgen::yield_now().await;
    value.await
}

#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_namespace = "Promise", js_name = "reject")]
    fn reject(value: &Object) -> Result<Promise<Object>, HostError>;
    #[ocamljs_bindgen(js_namespace = "Object", js_name = "is")]
    fn equal_error(error: &HostError, value: &Object) -> Result<bool, HostError>;
}
#[ocamljs_bindgen]
pub fn rejected(value: &Object) -> Result<Promise<Object>, HostError> {
    reject(value)
}
#[ocamljs_bindgen]
pub fn error_matches(error: &HostError, value: &Object) -> Result<bool, HostError> {
    equal_error(error, value)
}

#[ocamljsify::ocamljsify]
pub enum CopyFailure {
    TooLarge,
    Host(HostError),
}
fn copy_failure(error: ocamljs_bindgen::host::CopyError) -> CopyFailure {
    match error {
        ocamljs_bindgen::host::CopyError::TooLarge => CopyFailure::TooLarge,
        ocamljs_bindgen::host::CopyError::Host(error) => CopyFailure::Host(error),
    }
}
#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(js_name = "String")]
    fn native_text(text: &str) -> Result<ocamljs_bindgen::host::JsString, HostError>;
}
#[ocamljs_bindgen]
pub fn text(text: &str) -> Result<ocamljs_bindgen::host::JsString, HostError> {
    native_text(text)
}
#[ocamljs_bindgen]
pub fn copy_text(
    text: &ocamljs_bindgen::host::JsString,
    limit: u32,
) -> Result<String, CopyFailure> {
    text.to_rust_string(limit as usize).map_err(copy_failure)
}
#[ocamljs_bindgen]
pub fn copy_pixels(
    bytes: &ocamljs_bindgen::host::Uint8ClampedArray,
    limit: u32,
) -> Result<ocamljs_bindgen::host::Uint8ClampedArray, CopyFailure> {
    let bytes = bytes.to_vec(limit as usize).map_err(copy_failure)?;
    ocamljs_bindgen::host::Uint8ClampedArray::copy_from(&bytes).map_err(copy_failure)
}

/// Yield once and return the exact host object, including thenable objects.
#[ocamljs_bindgen]
pub async fn async_identity(value: Object) -> Object {
    ocamljs_bindgen::yield_now().await;
    value
}
