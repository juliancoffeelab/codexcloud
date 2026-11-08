//! Boundary fixtures. All bindings call the same Rust model functions.
#[cfg(feature = "ours")]
use ocamljs_bindgen::ocamljs_bindgen;
use serde::{Deserialize, Serialize};
#[cfg(feature = "baseline")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "baseline")]
include!(concat!(env!("OUT_DIR"), "/reference.rs"));
#[cfg_attr(all(feature = "ours", not(feature = "js")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js", ocamljsify::ocamljsify(js_value))]
#[derive(Clone, Serialize, Deserialize)]
pub struct Flat {
    pub id: u32,
    pub weight: f64,
    pub offset: i32,
    pub active: bool,
}
#[cfg_attr(all(feature = "ours", not(feature = "js")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js", ocamljsify::ocamljsify(js_value))]
#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u32,
    pub weight: f64,
    pub label: String,
    pub children: Vec<Node>,
}
#[cfg_attr(all(feature = "ours", not(feature = "js")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js", ocamljsify::ocamljsify(js_value("typed_array")))]
pub type Numbers = Vec<f64>;
#[cfg_attr(all(feature = "ours", not(feature = "js")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js", ocamljsify::ocamljsify(js_value("typed_array")))]
pub type ByteBuffer = Vec<u8>;
#[cfg_attr(all(feature = "ours", not(feature = "js")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js", ocamljsify::ocamljsify(js_value("string")))]
pub type Text = String;
#[cfg_attr(all(feature = "ours", not(feature = "js-record")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js-record", ocamljsify::ocamljsify(js_value))]
#[derive(Clone, Serialize, Deserialize)]
pub struct TextEnvelope {
    pub revision: u32,
    pub body: Text,
    pub labels: Vec<Text>,
}
#[cfg_attr(feature = "ours", ocamljs_bindgen(opaque))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub struct Store {
    numbers: Numbers,
    bytes: ByteBuffer,
    flats: Vec<Flat>,
    tree: Node,
    text: Text,
    labels: Vec<Text>,
}
fn tree(depth: u32, width: u32, next: &mut u32) -> Node {
    let id = *next;
    *next += 1;
    Node {
        id,
        weight: id as f64 + 0.25,
        label: format!("node-{id}"),
        children: if depth == 0 {
            vec![]
        } else {
            (0..width).map(|_| tree(depth - 1, width, next)).collect()
        },
    }
}
fn model_store(n: u32, depth: u32, width: u32, text_units: u32, unicode: bool) -> Store {
    assert!(
        n <= 1_048_576
            && depth <= 32
            && width <= 64
            && (width <= 1 || depth <= 3)
            && text_units <= 1_048_576
    );
    Store {
        numbers: (0..n).map(|i| i as f64 + 0.25).collect(),
        bytes: (0..n).map(|i| i as u8).collect(),
        flats: (0..n)
            .map(|i| Flat {
                id: i,
                weight: i as f64 + 0.25,
                offset: -(i as i32),
                active: i % 2 == 0,
            })
            .collect(),
        tree: tree(depth, width, &mut 0),
        text: (if unicode { "λ🦀" } else { "a" }).repeat(text_units as usize),
        labels: vec!["unit_archer_0123".into(), "café".into()],
    }
}
fn model_tree_sum(node: &Node) -> u32 {
    node.id + node.children.iter().map(model_tree_sum).sum::<u32>()
}
#[cfg_attr(feature = "ours", ocamljs_bindgen(opaque))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub struct Counter {
    value: u32,
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn noop() -> u32 {
    7
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn eight(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 {
    a + b + c + d + e + f + g + h
}
#[cfg_attr(feature="ours",ocamljs_bindgen(module = Store))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn store_new(n: u32, depth: u32, width: u32, text_units: u32, unicode: bool) -> Store {
    model_store(n, depth, width, text_units, unicode)
}
#[cfg_attr(feature="ours",ocamljs_bindgen(module = Counter))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn counter_new(value: u32) -> Counter {
    Counter { value }
}
#[cfg_attr(feature="ours",ocamljs_bindgen(module = Counter))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn counter_get(counter: &Counter) -> u32 {
    counter.value
}
#[cfg_attr(feature="ours",ocamljs_bindgen(module = Counter))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn counter_add(counter: &mut Counter, value: u32) -> u32 {
    counter.value = counter.value.wrapping_add(value);
    counter.value
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature="js"),ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="js",ocamljs_bindgen(module = Store))]
pub fn numbers_out(store: &Store) -> Numbers {
    store.numbers.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn numbers_out(store: &Store) -> Numbers {
    store.numbers.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numbers_out(store: &Store) -> JsValue {
    let result: Numbers = store.numbers.clone();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature="js"),ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="js",ocamljs_bindgen(module = Store))]
pub fn bytes_out(store: &Store) -> ByteBuffer {
    store.bytes.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn bytes_out(store: &Store) -> ByteBuffer {
    store.bytes.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_bytes_out(store: &Store) -> JsValue {
    let result: ByteBuffer = store.bytes.clone();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature="js"),ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="js",ocamljs_bindgen(module = Store, js_value))]
pub fn flats_out(store: &Store) -> Vec<Flat> {
    store.flats.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn flats_out(store: &Store) -> JsValue {
    let result: Vec<Flat> = store.flats.clone();
    serde_wasm_bindgen::to_value(&result).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_flats_out(store: &Store) -> JsValue {
    let result: Vec<Flat> = store.flats.clone();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature="js"),ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="js",ocamljs_bindgen(module = Store))]
pub fn tree_out(store: &Store) -> Node {
    store.tree.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn tree_out(store: &Store) -> JsValue {
    let result: Node = store.tree.clone();
    serde_wasm_bindgen::to_value(&result).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_tree_out(store: &Store) -> JsValue {
    let result: Node = store.tree.clone();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature="js"),ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="js",ocamljs_bindgen(module = Store))]
pub fn text_out(store: &Store) -> Text {
    store.text.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn text_out(store: &Store) -> Text {
    store.text.clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_text_out(store: &Store) -> JsValue {
    let result: Text = store.text.clone();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn numbers_in(value: Numbers) -> f64 {
    value.iter().sum()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn numbers_in(value: Numbers) -> f64 {
    value.iter().sum()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numbers_in(value: JsValue) -> JsValue {
    let value: Numbers = serde_wasm_bindgen::from_value(value).unwrap();
    let result: f64 = value.iter().sum();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn bytes_in(value: ByteBuffer) -> u32 {
    value.iter().map(|v| *v as u32).sum()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn bytes_in(value: ByteBuffer) -> u32 {
    value.iter().map(|v| *v as u32).sum()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_bytes_in(value: JsValue) -> JsValue {
    let value: ByteBuffer = serde_wasm_bindgen::from_value(value).unwrap();
    let result: u32 = value.iter().map(|v| *v as u32).sum();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn flats_in(value: Vec<Flat>) -> f64 {
    value.iter().map(|v| v.weight).sum()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn flats_in(value: JsValue) -> f64 {
    let value: Vec<Flat> = serde_wasm_bindgen::from_value(value).unwrap();
    value.iter().map(|v| v.weight).sum()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_flats_in(value: JsValue) -> JsValue {
    let value: Vec<Flat> = serde_wasm_bindgen::from_value(value).unwrap();
    let result: f64 = value.iter().map(|v| v.weight).sum();
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn tree_in(value: Node) -> u32 {
    model_tree_sum(&value)
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn tree_in(value: JsValue) -> u32 {
    let value: Node = serde_wasm_bindgen::from_value(value).unwrap();
    model_tree_sum(&value)
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_tree_in(value: JsValue) -> JsValue {
    let value: Node = serde_wasm_bindgen::from_value(value).unwrap();
    let result: u32 = model_tree_sum(&value);
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn text_in(value: Text) -> u32 {
    value.len() as u32
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn text_in(value: Text) -> u32 {
    value.len() as u32
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_text_in(value: JsValue) -> JsValue {
    let value: Text = serde_wasm_bindgen::from_value(value).unwrap();
    let result: u32 = value.len() as u32;
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn numbers_echo(value: Numbers) -> Numbers {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn numbers_echo(value: Numbers) -> Numbers {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numbers_echo(value: JsValue) -> JsValue {
    let value: Numbers = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Numbers = value;
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen(js_value))]
pub fn flats_echo(value: Vec<Flat>) -> Vec<Flat> {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn flats_echo(value: JsValue) -> JsValue {
    let value: Vec<Flat> = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Vec<Flat> = value;
    serde_wasm_bindgen::to_value(&result).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_flats_echo(value: JsValue) -> JsValue {
    let value: Vec<Flat> = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Vec<Flat> = value;
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn tree_echo(value: Node) -> Node {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn tree_echo(value: JsValue) -> JsValue {
    let value: Node = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Node = value;
    serde_wasm_bindgen::to_value(&result).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_tree_echo(value: JsValue) -> JsValue {
    let value: Node = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Node = value;
    upstream_wire::value(&result).unwrap()
}
#[cfg(feature = "ours")]
#[cfg_attr(not(feature = "js"), ocamljs_bindgen)]
#[cfg_attr(feature = "js", ocamljs_bindgen)]
pub fn text_echo(value: Text) -> Text {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn text_echo(value: Text) -> Text {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_text_echo(value: JsValue) -> JsValue {
    let value: Text = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Text = value;
    upstream_wire::value(&result).unwrap()
}

#[cfg(feature = "ours")]
#[ocamljs_bindgen(module = Store)]
pub fn flat_out(store: &Store) -> Flat {
    store.flats[0].clone()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn flat_out(store: &Store) -> JsValue {
    serde_wasm_bindgen::to_value(&(store.flats[0].clone())).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_flat_out(store: &Store) -> JsValue {
    let result: Flat = store.flats[0].clone();
    upstream_wire::value(&result).unwrap()
}

#[cfg(feature = "ours")]
#[ocamljs_bindgen]
pub fn flat_in(value: Flat) -> f64 {
    value.weight
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn flat_in(value: JsValue) -> f64 {
    let value: Flat = serde_wasm_bindgen::from_value(value).unwrap();
    value.weight
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_flat_in(value: JsValue) -> JsValue {
    let value: Flat = serde_wasm_bindgen::from_value(value).unwrap();
    let result: f64 = value.weight;
    upstream_wire::value(&result).unwrap()
}

#[cfg(feature = "ours")]
#[ocamljs_bindgen]
pub fn flat_echo(value: Flat) -> Flat {
    value
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn flat_echo(value: JsValue) -> JsValue {
    let value: Flat = serde_wasm_bindgen::from_value(value).unwrap();
    serde_wasm_bindgen::to_value(&(value)).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_flat_echo(value: JsValue) -> JsValue {
    let value: Flat = serde_wasm_bindgen::from_value(value).unwrap();
    let result: Flat = value;
    upstream_wire::value(&result).unwrap()
}

#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_noop() -> JsValue {
    upstream_wire::value(&(noop())).unwrap()
}

#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_add(a: u32, b: u32) -> JsValue {
    upstream_wire::value(&(add(a, b))).unwrap()
}

#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_eight(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> JsValue {
    upstream_wire::value(&(eight(a, b, c, d, e, f, g, h))).unwrap()
}

#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_counter_get(counter: &Counter) -> JsValue {
    upstream_wire::value(&(counter_get(counter))).unwrap()
}

#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_counter_add(counter: &mut Counter, value: u32) -> JsValue {
    upstream_wire::value(&(counter_add(counter, value))).unwrap()
}

#[cfg(any(feature = "baseline", feature = "direct-numeric"))]
#[cfg_attr(all(feature = "ours", not(feature = "js")), ocamljsify::ocamljsify)]
#[cfg_attr(feature = "js", ocamljsify::ocamljsify(js_value("typed_array")))]
pub type BorrowedNumbers = [f64];
#[cfg(any(feature = "baseline", feature = "direct-numeric"))]
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_slice(values: &BorrowedNumbers) -> f64 {
    values.iter().sum()
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_pair(a: Numbers, b: Numbers) -> f64 {
    a.iter().sum::<f64>() + 2.0 * b.iter().sum::<f64>()
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_mixed(a: Numbers, bias: f64, b: Numbers, scale: f64) -> f64 {
    (a.iter().sum::<f64>() + bias + b.iter().sum::<f64>()) * scale
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_eight(
    a: Numbers,
    b: Numbers,
    c: Numbers,
    d: Numbers,
    e: Numbers,
    f: Numbers,
    g: Numbers,
    h: Numbers,
) -> f64 {
    [a, b, c, d, e, f, g, h]
        .iter()
        .enumerate()
        .map(|(i, v)| (i + 1) as f64 * v.iter().sum::<f64>())
        .sum()
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_independent(mut a: Numbers, b: Numbers) -> bool {
    if a.is_empty() {
        return b.is_empty();
    }
    let previous = b[0];
    a[0] += 1.0;
    a.as_ptr() != b.as_ptr() && b[0] == previous
}
#[cfg_attr(feature="ours",ocamljs_bindgen(module=Store))]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_keep(values: Numbers) -> Store {
    let mut store = model_store(0, 0, 0, 0, false);
    store.numbers = values;
    store
}
#[cfg(feature = "ours")]
#[ocamljs_bindgen]
pub fn numeric_preflight(a: Numbers, flag: bool, b: Numbers) -> f64 {
    if flag { numeric_pair(a, b) } else { 0.0 }
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numeric_slice(a: JsValue) -> JsValue {
    let a: Numbers = serde_wasm_bindgen::from_value(a).unwrap();
    upstream_wire::value(&numeric_slice(&a)).unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numeric_pair(a: JsValue, b: JsValue) -> JsValue {
    upstream_wire::value(&numeric_pair(
        serde_wasm_bindgen::from_value(a).unwrap(),
        serde_wasm_bindgen::from_value(b).unwrap(),
    ))
    .unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numeric_mixed(a: JsValue, bias: f64, b: JsValue, scale: f64) -> JsValue {
    upstream_wire::value(&numeric_mixed(
        serde_wasm_bindgen::from_value(a).unwrap(),
        bias,
        serde_wasm_bindgen::from_value(b).unwrap(),
        scale,
    ))
    .unwrap()
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numeric_eight(
    a: JsValue,
    b: JsValue,
    c: JsValue,
    d: JsValue,
    e: JsValue,
    f: JsValue,
    g: JsValue,
    h: JsValue,
) -> JsValue {
    upstream_wire::value(&numeric_eight(
        serde_wasm_bindgen::from_value(a).unwrap(),
        serde_wasm_bindgen::from_value(b).unwrap(),
        serde_wasm_bindgen::from_value(c).unwrap(),
        serde_wasm_bindgen::from_value(d).unwrap(),
        serde_wasm_bindgen::from_value(e).unwrap(),
        serde_wasm_bindgen::from_value(f).unwrap(),
        serde_wasm_bindgen::from_value(g).unwrap(),
        serde_wasm_bindgen::from_value(h).unwrap(),
    ))
    .unwrap()
}
#[cfg(all(feature = "ours", feature = "direct-numeric"))]
#[ocamljs_bindgen]
pub fn numeric_borrow_preflight(a: &[f64], flag: bool, b: Numbers) -> f64 {
    if flag {
        a.iter().sum::<f64>() + b.iter().sum::<f64>()
    } else {
        0.0
    }
}
#[cfg_attr(feature = "ours", ocamljs_bindgen)]
#[cfg_attr(feature = "baseline", wasm_bindgen)]
pub fn numeric_heterogeneous(a: Numbers, b: ByteBuffer, scale: f64) -> f64 {
    (a.iter().sum::<f64>() + b.iter().map(|v| *v as f64).sum::<f64>()) * scale
}
#[cfg(feature = "baseline")]
#[wasm_bindgen]
pub fn wire_numeric_heterogeneous(a: JsValue, b: JsValue, scale: f64) -> JsValue {
    upstream_wire::value(&numeric_heterogeneous(
        serde_wasm_bindgen::from_value(a).unwrap(),
        serde_wasm_bindgen::from_value(b).unwrap(),
        scale,
    ))
    .unwrap()
}

// String transport controls. Native literal String/str deliberately remain native
// in both SDK variants; Text selects Unicode JS strings in the js feature.
#[cfg_attr(feature="ours", ocamljs_bindgen)]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn string_literal_in(value: String) -> u32 { value.len() as u32 }
#[cfg_attr(feature="ours", ocamljs_bindgen)]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn string_borrowed_in(value: &str) -> u32 { value.len() as u32 }
#[cfg_attr(feature="ours", ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn string_store_new(value: Text) -> Store {
    let mut store = model_store(0,0,0,0,false);
    store.text = value;
    store
}
#[cfg_attr(feature="ours", ocamljs_bindgen)]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn string_mixed(a: String, b: &str, c: Text, bias: u32) -> u32 {
    a.len() as u32 + 2*b.len() as u32 + 3*c.len() as u32 + bias
}

// Mirrors Hex's Result<String, Error> save boundary: borrowed owner plus scalar
// identity/revision arguments. Both toolchains execute the identical Rust body.
#[cfg_attr(feature="ours", ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn text_result_out(store: &Store, id: u32, revision: u32) -> Result<Text, u32> {
    if id == 7 && revision == 0 { Ok(store.text.clone()) } else { Err(1) }
}
// Same Rust work, but an explicitly throwing API with a Unicode JS success value.
#[cfg_attr(feature="ours", ocamljs_bindgen(module = Store, throws, js_value))]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn text_throws_out(store: &Store, id: u32, revision: u32) -> Result<String, u32> {
    if id == 7 && revision == 0 { Ok(store.text.clone()) } else { Err(1) }
}
#[cfg_attr(feature="ours", ocamljs_bindgen(module = Store, throws))]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn text_throws_native_out(store: &Store, id: u32, revision: u32) -> Result<String, u32> {
    if id == 7 && revision == 0 { Ok(store.text.clone()) } else { Err(1) }
}
fn model_envelope(store: &Store) -> TextEnvelope {
    TextEnvelope { revision: 7, body: store.text.clone(), labels: store.labels.clone() }
}
#[cfg_attr(feature="ours", ocamljs_bindgen(module = Store))]
#[cfg_attr(feature="baseline", wasm_bindgen)]
pub fn string_envelope_store_new(value: Text, count: u32, unicode: bool) -> Store {
    assert!(count <= 4096);
    let mut store = string_store_new(value);
    store.labels = (0..count).map(|id| if unicode { format!("弓兵_{id}_café") } else { format!("unit_{id}") }).collect();
    store
}
#[cfg(feature="ours")]
#[ocamljs_bindgen(module = Store)]
pub fn text_envelope_out(store: &Store) -> TextEnvelope { model_envelope(store) }
#[cfg(feature="baseline")]
#[wasm_bindgen]
pub fn text_envelope_out(store: &Store) -> JsValue {
    serde_wasm_bindgen::to_value(&model_envelope(store)).unwrap()
}
fn model_envelope_in(value: TextEnvelope) -> u32 {
    value.revision + value.body.len() as u32 + value.labels.iter().map(|s|s.len() as u32).sum::<u32>()
}
#[cfg(feature="ours")]
#[ocamljs_bindgen]
pub fn text_envelope_in(value: TextEnvelope) -> u32 { model_envelope_in(value) }
#[cfg(feature="baseline")]
#[wasm_bindgen]
pub fn text_envelope_in(value: JsValue) -> u32 {
    model_envelope_in(serde_wasm_bindgen::from_value(value).unwrap())
}
#[cfg(feature="ours")]
#[ocamljs_bindgen]
pub fn text_envelope_echo(value: TextEnvelope) -> TextEnvelope { value }
#[cfg(feature="baseline")]
#[wasm_bindgen]
pub fn text_envelope_echo(value: JsValue) -> JsValue {
    let value: TextEnvelope = serde_wasm_bindgen::from_value(value).unwrap();
    serde_wasm_bindgen::to_value(&value).unwrap()
}

// Ownership probes for specialized success and buffered/error transport.
#[cfg(all(feature="ours", feature="lifecycle-probes"))]
#[ocamljs_bindgen(module = Store)]
pub fn text_result_notify(store: &Store, notify: ocamljs_bindgen::Callback<Flat>, fail: bool) -> Result<Text, u32> {
    let _ = notify.emit(Flat { id: 7, weight: 1.0, offset: 0, active: true });
    if fail { Err(1) } else { Ok(store.text.clone()) }
}
#[cfg(all(feature="ours", feature="lifecycle-probes"))]
#[ocamljs_bindgen(module = Store)]
pub fn text_encoded_result_out(store: &Store) -> ocamljs_bindgen::Encoded<Result<Text, u32>> {
    ocamljs_bindgen::Encoded::new(Ok::<Text, u32>(store.text.clone()))
}
