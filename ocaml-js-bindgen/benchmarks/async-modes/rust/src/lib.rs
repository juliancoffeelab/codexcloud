use ocamljs_bindgen::ocamljs_bindgen;

#[ocamljs_bindgen]
pub fn add(value: u32) -> u32 {
    value + 1
}

#[cfg(feature = "host")]
#[ocamljs_bindgen(host)]
pub struct HostObject;

#[cfg(feature = "host")]
#[ocamljs_bindgen]
pub fn host_identity(value: HostObject) -> HostObject {
    value
}

#[cfg(feature = "async")]
#[ocamljs_bindgen]
pub async fn ready(value: u32) -> u32 {
    value + 1
}

#[cfg(feature = "async")]
#[ocamljs_bindgen]
pub async fn suspended(value: u32) -> u32 {
    ocamljs_bindgen::yield_now().await;
    value + 1
}

#[cfg(feature = "async")]
#[ocamljs_bindgen]
pub async fn pending() -> u32 {
    std::future::pending().await
}

#[cfg(feature = "opaque")]
#[ocamljs_bindgen(opaque)]
pub struct Counter {
    value: u32,
}

#[cfg(feature = "opaque")]
#[ocamljs_bindgen(module = Counter)]
pub fn counter(value: u32) -> Counter {
    Counter { value }
}

#[cfg(all(feature = "opaque", feature = "async"))]
#[ocamljs_bindgen(module = Counter)]
pub async fn borrowed(value: &Counter) -> u32 {
    ocamljs_bindgen::yield_now().await;
    value.value + 1
}

#[cfg(feature = "owned-result")]
#[ocamljs_bindgen(module = Counter)]
pub async fn owned(value: u32) -> Counter {
    ocamljs_bindgen::yield_now().await;
    Counter { value: value + 1 }
}

#[cfg(feature = "owned-result")]
#[ocamljsify::ocamljsify]
pub struct Nested {
    pub value: u32,
    pub owner: Counter,
}

#[cfg(feature = "owned-result")]
#[ocamljs_bindgen(module = Counter)]
pub async fn nested(value: u32) -> Result<Nested, Counter> {
    ocamljs_bindgen::yield_now().await;
    Ok(Nested {
        value: value + 1,
        owner: Counter { value },
    })
}

#[cfg(feature = "completion")]
#[ocamljs_bindgen]
pub async fn text_native() -> String { "héllo 🦀".repeat(512) }
#[cfg(feature = "completion")]
#[ocamljs_bindgen(js_value)]
pub async fn text_js() -> String { "héllo 🦀".repeat(512) }
#[cfg(feature = "completion")]
#[ocamljsify::ocamljsify]
pub struct Point { pub x: f64, pub y: f64 }
#[cfg(feature = "completion")]
#[ocamljs_bindgen]
pub async fn point() -> Point { Point { x: 7.5, y: 8.5 } }
#[cfg(feature = "completion")]
#[ocamljs_bindgen(js_value)]
pub async fn text_result() -> Result<String,u32> { Ok("héllo 🦀".repeat(512)) }
#[cfg(feature = "completion")]
#[ocamljs_bindgen(js_value)]
pub async fn text_error() -> Result<String,u32> { Err(42) }
#[cfg(feature = "completion")]
#[ocamljs_bindgen]
pub async fn boolean() -> bool { true }
#[cfg(feature = "completion")]
#[ocamljs_bindgen]
pub async fn wide() -> u64 { u64::MAX }
#[cfg(feature = "completion")]
#[ocamljs_bindgen]
pub async fn unit() {}
#[cfg(feature = "completion")]
#[ocamljs_bindgen]
pub async fn host_echo(value: HostObject) -> HostObject { value }

#[cfg(feature="callbacks")]
#[ocamljsify::ocamljsify]
pub struct Event { pub value:u32 }
#[cfg(feature="callbacks")]
#[ocamljs_bindgen]
pub fn callback_once(value:u32, callback:ocamljs_bindgen::Callback<Event>) -> u32 {
    callback.emit(Event { value:value+1 }).unwrap(); value+1
}
