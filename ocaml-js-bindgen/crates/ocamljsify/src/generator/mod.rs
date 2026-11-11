//! Read the compiler-emitted API schema and generate compiler-native OCaml codecs.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::Path};
mod abi;
mod bulk;
mod calls;
mod collections;
mod documentation;
mod enum_modules;
mod initialization;
mod ml;
mod modules;
mod native_strings;
mod packaging;
mod projection;
mod references;
pub use packaging::with_schema;
mod validation;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ty {
    pub kind: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub qualified: bool,
    #[serde(default)]
    pub args: Vec<Ty>,
    pub inner: Option<Box<Ty>>,
    pub len: Option<usize>,
    #[serde(default)]
    pub mutable: bool,
    #[serde(default)]
    pub map: String,
}
impl Ty {
    pub fn unit(&self) -> bool {
        self.kind == "tuple" && self.args.is_empty()
    }
    pub fn scalar(&self) -> bool {
        self.unit()
            || self.kind == "named"
                && matches!(
                    self.name.as_str(),
                    "bool"
                        | "u8"
                        | "i8"
                        | "u16"
                        | "i16"
                        | "u32"
                        | "i32"
                        | "usize"
                        | "isize"
                        | "u64"
                        | "i64"
                        | "f32"
                        | "f64"
                )
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
    #[serde(default, skip_serializing_if = "is_false")]
    pub js_value: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub js_value_rec: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub js_selector: String,
    #[serde(default)]
    pub docs: String,
    #[serde(default, rename = "serde")]
    pub serde_attrs: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Body {
    pub style: String,
    pub fields: Vec<Field>,
}
fn is_false(value: &bool) -> bool {
    !*value
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variant {
    #[serde(default, skip_serializing_if = "is_false")]
    pub rust_default: bool,
    pub name: String,
    #[serde(default)]
    pub serde_name: Option<String>,
    pub body: Body,
    #[serde(default)]
    pub docs: String,
    #[serde(default, rename = "serde")]
    pub serde_attrs: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub module: String,
    #[serde(default)]
    pub member: String,
    #[serde(default)]
    pub rust_path: String,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub flat: bool,
    #[serde(default)]
    pub key_eq: bool,
    #[serde(default)]
    pub key_ord: bool,
    #[serde(default)]
    pub params: Vec<Value>,
    pub body: Option<Body>,
    #[serde(default)]
    pub variants: Vec<Variant>,
    pub ty: Option<Ty>,
    pub result: Option<Ty>,
    #[serde(default)]
    pub throws: bool,
    #[serde(default)]
    pub asynchronous: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub buffered: bool,
    /// Select JavaScript representation at a type declaration or exported occurrence.
    #[serde(default, skip_serializing_if = "is_false")]
    pub js_value: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub js_value_rec: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub js_selector: String,
    #[serde(default)]
    pub docs: String,
    #[serde(default, rename = "serde")]
    pub serde_attrs: String,
    #[serde(default)]
    pub js_name: String,
    #[serde(default)]
    pub js_namespace: String,
    #[serde(default)]
    pub js_module: String,
    #[serde(default)]
    pub operation: String,
    #[serde(default)]
    pub reference_support: String,
}
impl Entry {
    pub fn is_type(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "struct" | "enum" | "alias" | "opaque" | "host"
        )
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Backend {
    Melange,
    JsOfOcaml,
    Rescript,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AsyncRuntime {
    #[default]
    Promise,
    Lwt,
}
#[derive(Clone)]
pub struct Options {
    pub name: String,
    pub backend: Backend,
    pub async_runtime: AsyncRuntime,
    /// Retain Rust's schema, names, producers and debug sections in the Wasm.
    pub debug: bool,
    /// Emit checked codecs instead of trusting the generated value contract.
    pub paranoid: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            name: "bindings".into(),
            backend: Backend::Melange,
            async_runtime: AsyncRuntime::Promise,
            debug: false,
            paranoid: false,
        }
    }
}
impl Options {
    pub fn uses_lwt(&self) -> bool {
        matches!(self.backend, Backend::JsOfOcaml) && self.async_runtime == AsyncRuntime::Lwt
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Schema {
    pub entries: BTreeMap<String, Entry>,
    pub fingerprint: String,
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub fn read(wasm: &[u8]) -> Result<Schema, Error> {
    wasmparser::Validator::new().validate_all(wasm)?;
    let mut raw = BTreeMap::new();

    for payload in wasmparser::Parser::new(0).parse_all(wasm) {
        if let wasmparser::Payload::CustomSection(c) = payload? {
            if c.name() == "ocamljs.schema" {
                let text = std::str::from_utf8(c.data())?;

                for line in text.lines().filter(|l| !l.trim().is_empty()) {
                    let value: Value = serde_json::from_str(line)?;
                    if value.get("constructor").is_some() {
                        return Err(
                            "constructor metadata was removed; export ordinary named functions"
                                .into(),
                        );
                    }
                    let e: Entry = serde_json::from_value(value)?;
                    if raw
                        .insert(e.name.clone(), serde_json::from_str::<Value>(line)?)
                        .is_some()
                    {
                        return Err(
                            format!("ambiguous ABI name {}; export unique names", e.name).into(),
                        );
                    }
                }
            }
        }
    }
    if raw.is_empty() {
        return Err(
            "no ocamljs.schema section; mark exported Rust functions with #[ocamljs_bindgen]"
                .into(),
        );
    }
    // Fingerprint the emitted contract, before resolving source-level paths.
    let canonical = serde_json::to_vec(&raw)?;
    let fingerprint = format!("{:x}", Sha256::digest(&canonical));
    fn promise(v: &Value) -> bool {
        match v {
            Value::Object(v) => {
                v.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|n| matches!(n, "Promise" | "HostError"))
                    || v.values().any(promise)
            }
            Value::Array(v) => v.iter().any(promise),
            _ => false,
        }
    }
    if raw.values().any(|e| e["kind"] == "import" || promise(e)) {
        raw.entry("HostError".into()).or_insert_with(||serde_json::json!({"kind":"host","name":"HostError","docs":"A caught JavaScript thrown value. Its identity is preserved, including primitive values."}));
    }
    fn mentions(v: &Value, name: &str) -> bool {
        match v {
            Value::Object(v) => {
                v.get("name").and_then(Value::as_str) == Some(name)
                    || v.values().any(|v| mentions(v, name))
            }
            Value::Array(v) => v.iter().any(|v| mentions(v, name)),
            _ => false,
        }
    }
    for (name, docs) in [
        ("JsString", "A native JavaScript string."),
        (
            "Uint8ClampedArray",
            "A native JavaScript pixel byte array; host GC owns its storage.",
        ),
    ] {
        if raw.values().any(|e| mentions(e, name)) {
            raw.entry(name.into())
                .or_insert_with(|| serde_json::json!({"kind":"host","name":name,"docs":docs}));
        }
    }
    let paths = raw
        .values()
        .filter(|e| e["kind"] != "function" && e["kind"] != "module")
        .filter_map(|e| {
            let module = e["module"].as_str()?;
            let descriptor = raw
                .values()
                .find(|e| e["kind"] == "module" && e["member"] == module)?;
            Some((
                format!(
                    "{}::{}",
                    descriptor["rust_path"].as_str()?,
                    e["member"].as_str()?
                ),
                e["name"].as_str()?.to_owned(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    fn resolve(value: &mut Value, paths: &BTreeMap<String, String>) {
        if value["kind"] == "named" {
            if let Some(path) = value["path"].as_str() {
                let mut path = path.trim_start_matches("crate::");
                while let Some(rest) = path.strip_prefix("super::") {
                    path = rest;
                }
                if let Some(name) = paths.get(path) {
                    value["name"] = Value::String(name.clone());
                }
            }
        }
        match value {
            Value::Object(fields) => {
                for value in fields.values_mut() {
                    resolve(value, paths);
                }
            }
            Value::Array(values) => {
                for value in values {
                    resolve(value, paths);
                }
            }
            _ => {}
        }
    }
    for value in raw.values_mut() {
        resolve(value, &paths);
    }
    let entries = raw
        .iter()
        .map(|(name, value)| Ok((name.clone(), serde_json::from_value(value.clone())?)))
        .collect::<Result<BTreeMap<String, Entry>, serde_json::Error>>()?;
    let schema = Schema {
        entries,
        fingerprint,
    };
    schema.validate()?;

    validation::wasm_exports(&schema, wasm)?;
    Ok(schema)
}
impl Schema {
    fn validate(&self) -> Result<(), Error> {
        validation::structure(self)?;
        for e in self.entries.values().filter(|e| e.kind != "module") {
            if e.kind == "import" {
                references::validate_import(self, e)?;
                continue;
            }
            if e.kind == "function" {
                for p in &e.params {
                    let t: Ty = serde_json::from_value(p["ty"].clone())?;
                    if t.name != "Callback"
                        && !references::host(self, &t)
                        && references::contains_host(
                            self,
                            &t,
                            &mut std::collections::BTreeSet::new(),
                        )
                    {
                        return Err("host values must be direct parameters; nested host inputs are not serialized".into());
                    }
                    let moving = p["mode"] == "move";
                    if moving
                        && (t.kind != "named"
                            || self.entries.get(&t.name).is_none_or(|e| e.kind != "opaque")
                            || !t.args.is_empty())
                    {
                        return Err("consume requires a direct by-value opaque argument".into());
                    }
                    if !p["mode"].is_null() && p["mode"] != "move" && p["mode"] != "value" {
                        return Err("invalid argument mode".into());
                    }
                    self.check_ty(&t, &[], !moving)?;
                    validation::instantiated(
                        self,
                        &t,
                        !moving,
                        &mut std::collections::BTreeSet::new(),
                    )?;
                }
                let r = e.result.as_ref().unwrap();
                if e.asynchronous && references::resolved(self, r).name == "Promise" {
                    return Err("await a host Promise inside Rust async; returning a nested Promise has different native flattening semantics".into());
                }
                self.check_ty(r, &[], false)?;
                validation::instantiated(self, r, false, &mut std::collections::BTreeSet::new())?;
                if e.throws && !(r.kind == "named" && r.name == "Result") {
                    return Err(format!("{}: throws requires Result<T,E>", e.name).into());
                }
            } else {
                let params = e
                    .params
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                if e.kind != "opaque" {
                    if let Some(b) = &e.body {
                        for f in &b.fields {
                            self.check_ty(&f.ty, &params, false)?;
                        }
                    }
                    for v in &e.variants {
                        for f in &v.body.fields {
                            self.check_ty(&f.ty, &params, false)?;
                        }
                    }
                    if let Some(t) = &e.ty {
                        self.check_ty(t, &params, false)?;
                    }
                }
            }
        }
        modules::validate(self)?;
        collections::validate(self)?;
        Ok(())
    }
    fn check_ty(&self, t: &Ty, params: &[String], input: bool) -> Result<(), Error> {
        if t.kind == "ref" {
            let i = t.inner.as_ref().ok_or("reference missing inner type")?;
            if abi::text_input(self, t).is_some() || abi::numeric_input(self, t).is_some() {
                if t.mutable {
                    return Err("mutable numeric slices are unsupported".into());
                }
                if !input {
                    return Err("references are call-scoped input parameters only".into());
                }
                return Ok(());
            }
            if !references::host(self, i)
                && self
                    .entries
                    .get(&i.name)
                    .is_none_or(|e| !matches!(e.kind.as_str(), "opaque" | "host"))
            {
                return Err(format!("&{} requires #[ocamljs_bindgen(opaque)]", i.name).into());
            }
            if references::host(self, i) && t.mutable {
                return Err("host references use shared Rust borrows; host mutation is explicit in imported operations".into());
            }
            if !input {
                return Err("returned/nested references are unsupported".into());
            }
            return Ok(());
        }
        if t.name == "str" {
            return Ok(());
        }
        if t.kind == "slice" {
            let element = t.inner.as_ref().ok_or("slice missing element")?;
            if !bulk::numeric(&abi::resolve(self, element)) && !params.contains(&element.name) {
                return Err("numeric slices require a primitive numeric element".into());
            }
            return self.check_ty(element, params, input);
        }
        if t.kind == "array" {
            if t.len.is_none_or(|n| n > 1_000_000) {
                return Err("invalid fixed array length".into());
            }
            return self.check_ty(
                t.inner.as_ref().ok_or("missing array element")?,
                params,
                input,
            );
        }
        if t.kind == "tuple" {
            for a in &t.args {
                self.check_ty(a, params, input)?;
            }
            return Ok(());
        }
        if t.kind != "named" {
            return Err(format!("unsupported type kind {}", t.kind).into());
        }
        if (t.scalar() || t.name == "String" || params.contains(&t.name) && !t.qualified)
            && !t.args.is_empty()
        {
            return Err("primitive and generic parameter types cannot have arguments".into());
        }
        if t.name == "Callback" {
            if !input
                || t.args.len() != 1
                || validation::contains_opaque(
                    self,
                    &t.args[0],
                    &mut std::collections::BTreeSet::new(),
                )
            {
                return Err(
                    "Callback requires one owned structural input payload without opaque objects"
                        .into(),
                );
            }
            let payload = &t.args[0];
            if payload.kind != "named"
                || !payload.args.is_empty()
                || self.entries.get(&payload.name).is_none_or(|e| {
                    !matches!(e.kind.as_str(), "struct" | "enum") || !e.params.is_empty()
                })
            {
                return Err("Callback payload must be a concrete nongeneric described struct or enum; wrap other payloads in a record".into());
            }
            self.check_ty(payload, params, false)?;
            return Ok(());
        }
        let expected = match t.name.as_str() {
            "Vec" | "LinkedList" | "Option" | "Box" | "Rc" | "Arc" | "Promise" => Some(1),
            "Result" | "BTreeMap" | "HashMap" => Some(2),
            _ => None,
        };
        if let Some(n) = expected {
            if t.args.len() != n {
                return Err(format!("{} requires {n} type arguments", t.name).into());
            }
        } else if !t.scalar() && t.name != "String" && !(params.contains(&t.name) && !t.qualified) {
            let e = self.entries.get(&t.name).ok_or_else(|| {
                format!(
                    "{} has no ABI description; annotate the type or an explicit type alias",
                    t.name
                )
            })?;
            if !e.is_type() {
                return Err("a function or module cannot be used as a value type".into());
            }
            if input && e.kind == "opaque" {
                return Err("opaque inputs must be borrowed as &T or &mut T".into());
            }
            if e.params.len() != t.args.len() {
                return Err(format!("wrong generic arity for {}", t.name).into());
            }
        }
        if t.name == "Promise" && references::resolved(self, &t.args[0]).name == "Promise" {
            return Err("nested Promise<Promise<T>> is unsupported because native promises flatten; await explicitly in Rust".into());
        }
        if t.name == "Promise" && !references::host(self, &t.args[0]) {
            return Err("Promise<T> requires a host value payload; decode native Rust data explicitly in Rust".into());
        }
        if matches!(t.name.as_str(), "Rc" | "Arc")
            && validation::contains_opaque(self, &t.args[0], &mut std::collections::BTreeSet::new())
        {
            return Err("Rc<opaque> cannot preserve identity; return an owned opaque object or call-scoped reference".into());
        }
        for a in &t.args {
            self.check_ty(a, params, input)?;
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
struct BindingInput {
    schema: Schema,
    imports: Vec<String>,
}

fn import_names(wasm: &[u8]) -> Result<Vec<String>, Error> {
    let mut names = std::collections::BTreeSet::new();
    for payload in wasmparser::Parser::new(0).parse_all(wasm) {
        if let wasmparser::Payload::ImportSection(section) = payload? {
            for import in section {
                let import = import?;
                // Reference lowering removes only this private slot-management module.
                // All runtime capability imports retain their names.
                if import.module != "ocamljs_ref" {
                    names.insert(import.name.to_owned());
                }
            }
        }
    }
    Ok(names.into_iter().collect())
}

/// Canonical code-generation input, independent of function bodies and layout.
pub fn binding_input(wasm: &[u8]) -> Result<Vec<u8>, Error> {
    let schema = read(wasm)?;
    Ok(serde_json::to_vec(&BindingInput {
        schema,
        imports: import_names(wasm)?,
    })?)
}

/// Generate compiler-native bindings without parsing or optimizing implementation code.
pub fn generate_bindings(input: &[u8], out: &Path, options: &Options) -> Result<String, Error> {
    let BindingInput { schema, imports } = serde_json::from_slice(input)?;
    let schema = &schema;
    schema.validate()?;
    if options.async_runtime == AsyncRuntime::Lwt && !matches!(options.backend, Backend::JsOfOcaml)
    {
        return Err("Lwt generation requires the js_of_ocaml backend".into());
    }
    if options.name.is_empty()
        || !options
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("binding name must be an identifier".into());
    }
    std::fs::create_dir_all(out)?;
    let effective = projection::resolve(schema)?;
    let (implementation, interface, js_codecs) = ml::generate(&effective, options)?;
    std::fs::write(out.join("Bindings.ml"), &implementation)?;
    std::fs::write(out.join("Bindings.mli"), interface)?;
    let mut wrappers = calls::generate(schema, options, &implementation)?;
    wrappers.push_str(&bulk::generate(schema, &implementation, options)?);
    wrappers.push_str(&js_codecs);
    let mut capabilities = std::collections::BTreeSet::new();
    capabilities.insert(if options.paranoid {
        "paranoid"
    } else {
        "trusted"
    });
    if projection::active(&effective) {
        capabilities.insert("js_projection");
    } else {
        capabilities.insert("no_js_projection");
    }
    if references::used(schema) {
        capabilities.insert("references");
    }
    if references::promises(schema) {
        capabilities.insert("host_promises");
    }
    if schema.entries.values().any(|e| e.kind == "opaque") {
        capabilities.insert("owners");
    }
    let asynchronous = schema.entries.values().any(|e| e.asynchronous);
    capabilities.insert(if asynchronous { "async" } else { "sync" });
    if schema
        .entries
        .values()
        .any(|e| abi::async_direct(schema, e))
    {
        capabilities.insert("async_direct");
    }
    if wrappers.contains("completeOpaque(") {
        capabilities.insert("async_opaque");
    }
    if wrappers.contains("completeStringResult(") {
        capabilities.insert("async_string_result");
    }
    if wrappers.contains("discardHost") {
        capabilities.insert("async_host");
    }
    capabilities.insert(if options.uses_lwt() { "lwt" } else { "promise" });
    let async_owners = schema.entries.values().any(|entry| {
        entry.asynchronous
            && entry
                .result
                .as_ref()
                .is_some_and(|ty| abi::owns_result(schema, ty))
    });
    capabilities.insert(if async_owners {
        "async_owners"
    } else {
        "no_async_owners"
    });
    // Only js_of_ocaml adapters expose receiver callbacks that can decline a
    // result (Lwt cancellation, or adapter delivery failure). Native JS Promise
    // resolvers always accept it and need no acknowledgment/abandonment layer.
    if matches!(options.backend, Backend::JsOfOcaml) {
        capabilities.insert("delivery_ack");
        if async_owners {
            capabilities.insert("delivery_owners");
        }
    } else {
        capabilities.insert("direct_promise");
    }

    for name in &imports {
        for (prefix, capability) in [
            ("callback_", "callbacks"),
            ("host_promise_", "host_promises"),
            ("host_string_", "host_text"),
            ("host_bytes_", "host_bytes"),
            ("fetch_", "fetch"),
        ] {
            if name.starts_with(prefix) {
                capabilities.insert(capability);
            }
        }
    }
    if !capabilities.contains("callbacks") {
        capabilities.insert("no_callbacks");
    }
    let mut codec_methods = std::collections::BTreeSet::new();
    for name in [
        "writer",
        "finish",
        "close",
        "enter",
        "leave",
        "r_budget",
        "w_budget",
        "r_length",
        "w_length",
        "r_handle",
        "r_span",
        "r_pool_string",
        "r_js_string",
        "r_string",
        "w_string",
        "w_js_string",
        "r_bool",
        "w_bool",
        "r_u8",
        "w_u8",
        "r_i8",
        "w_i8",
        "r_u16",
        "w_u16",
        "r_i16",
        "w_i16",
        "r_u32",
        "w_u32",
        "r_i32",
        "w_i32",
        "r_f32",
        "w_f32",
        "r_f64",
        "w_f64",
        "r_i64",
        "w_i64",
        "r_u64",
        "w_u64",
        "r_host",
    ] {
        if implementation.contains(&format!("\"{name}\""))
            || wrappers.contains(&format!("methods.{name}("))
        {
            codec_methods.insert(name);
        }
    }
    for (name, dep) in [
        ("r_js_string", "r_u32"),
        ("r_js_string", "r_pool_string"),
        ("r_string", "r_u32"),
        ("r_string", "r_pool_string"),
        ("w_js_string", "w_string"),
        ("w_js_string", "w_u32"),
        ("w_string", "w_u32"),
        ("r_handle", "r_u32"),
        ("r_host", "r_u32"),
        ("r_length", "r_u32"),
        ("w_length", "w_u32"),
        ("r_bool", "r_u8"),
        ("w_bool", "w_u8"),
    ] {
        if codec_methods.contains(name) {
            codec_methods.insert(dep);
        }
    }
    capabilities.extend(codec_methods.iter().copied());
    capabilities.insert(
        if options.paranoid || codec_methods.contains("r_pool_string") {
            "pool_metadata"
        } else {
            "no_pool_metadata"
        },
    );
    if codec_methods.contains("w_string")
        || codec_methods.contains("w_js_string")
        || wrappers.contains("writeText(")
    {
        capabilities.insert("text_writer");
    }
    if wrappers.contains("readTyped(") {
        capabilities.insert("typed_reader");
    }
    if wrappers.contains("writeTyped(") {
        capabilities.insert("typed_writer");
    }

    if schema.entries.values().any(|e| {
        e.kind == "function"
            && (e.asynchronous && !abi::async_direct(schema, e)
                || !e.asynchronous
                    && !e.result.as_ref().unwrap().scalar()
                    && !references::host(schema, e.result.as_ref().unwrap())
                    && abi::opaque(schema, e.result.as_ref().unwrap()).is_none()
                    && !abi::text_output(schema, e.result.as_ref().unwrap())
                    && abi::direct_result(schema, e.result.as_ref().unwrap()).is_none())
    }) || capabilities.contains("callbacks")
    {
        capabilities.insert("reader");
    }
    let buffered_string_error = schema.entries.values().any(|e| {
        if e.kind != "function" || e.buffered {
            return false;
        }
        let Some(result) = &e.result else {
            return false;
        };
        if abi::text_result(schema, result).is_none() {
            return false;
        }
        let result = abi::resolve(schema, result);
        !e.asynchronous || !abi::resolve(schema, &result.args[1]).scalar()
    });
    capabilities.insert(if buffered_string_error {
        "buffered_string_error"
    } else {
        "no_buffered_string_error"
    });
    if buffered_string_error {
        capabilities.insert("reader");
    }
    if (asynchronous || capabilities.contains("callbacks")) && capabilities.contains("reader") {
        capabilities.insert("close");
    }
    let mut numeric = String::new();
    for (name, size, get, set) in bulk::NUMBERS {
        if codec_methods.contains(format!("r_{name}").as_str()) {
            numeric.push_str(&format!(
                "methods.r_{name}=r=>{{const p=r.take({size});return r.view().{get}(p,true);}};\n"
            ));
        }
        if codec_methods.contains(format!("w_{name}").as_str()) {
            numeric.push_str(&format!("methods.w_{name}=(w,v)=>{{{};const p=w.reserve({size});w.data.{set}(p,v,true);}};\n",calls::scalar_checked(options.paranoid, &Ty{kind:"named".into(),name:name.into(),qualified:false,args:vec![],inner:None,len:None,mutable:false,map:String::new()},"v")));
        }
    }
    for name in ["i64", "u64"] {
        if codec_methods.contains(format!("r_{name}").as_str()) {
            numeric.push_str(&format!("methods.r_{name}=r=>{{const p=r.take(8);return r.view().getBigInt64(p,true).toString();}};\n"));
        }
        if codec_methods.contains(format!("w_{name}").as_str()) {
            numeric.push_str(&format!("methods.w_{name}=(w,v)=>{{const n=wideInteger(v,false),p=w.reserve(8);w.data.setBigInt64(p,n,true);}};\n"));
        }
    }
    if wrappers.contains("copyInput(") || wrappers.contains("encodeInput(") {
        capabilities.insert("input_buffer");
    }
    if wrappers.contains("encodeInput(") {
        capabilities.insert("writer");
    }
    if capabilities.contains("input_buffer") || capabilities.contains("fetch") {
        capabilities.insert("input");
    }
    if matches!(options.backend, Backend::JsOfOcaml) && references::promises(schema) {
        capabilities.insert("jsoo_promise");
    }
    if wrappers.contains("completionView(") {
        capabilities.insert("completion_view");
    }
    if wrappers.contains("readStringResultOutput(") || wrappers.contains("completeStringResult(") {
        capabilities.insert("string_result");
    }
    if implementation.contains("\"string_result_wrap_error\"") {
        capabilities.insert("string_result_js_error");
    }
    if wrappers.contains("stringView(")
        || wrappers.contains("readUnicodeStringOutput")
        || wrappers.contains("readNativeStringOutput")
        || wrappers.contains("prepareUnicodeString(")
        || wrappers.contains("prepareNativeByteString(")
    {
        capabilities.insert("string_memory");
    }
    if wrappers.contains("prepareUnicodeString(") {
        capabilities.insert("unicode_prepare");
    }
    if wrappers.contains("stringPrefix(") {
        capabilities.insert("string_prefix");
    }
    if wrappers.contains("prepareNativeByteString(")
        || wrappers.contains("encodeInput(")
            && !matches!(options.backend, Backend::Rescript)
            && (implementation.contains("\"w_string\"") || wrappers.contains("methods.w_string("))
    {
        capabilities.insert("native_string_input");
    }
    if !matches!(options.backend, Backend::Rescript)
        && (wrappers.contains("readNativeStringOutput")
            || !options.paranoid && capabilities.contains("r_string"))
    {
        capabilities.insert("native_string_output");
    }
    if capabilities.contains("string_memory") || capabilities.contains("r_pool_string") {
        capabilities.insert("unicode_decoder");
    }
    if capabilities.contains("string_memory")
        || capabilities.contains("w_string")
        || capabilities.contains("w_js_string")
    {
        capabilities.insert("wide_string_scan");
    }
    if wrappers.contains("readUnicodeStringOutput") {
        capabilities.insert("unicode_string_output");
    }
    if !options.paranoid
        && !matches!(options.backend, Backend::Rescript)
        && (wrappers.contains("prepareNativeByteString(")
            || capabilities.contains("native_string_output")
            || capabilities.contains("native_string_input"))
    {
        capabilities.insert("native_string_host");
    }
    if wrappers.contains("prepareNativeByteString(") || capabilities.contains("native_string_host")
    {
        capabilities.insert("native_prepare");
    }
    if capabilities.contains("unicode_prepare") || capabilities.contains("native_prepare") {
        capabilities.insert("string_prepare");
    }
    if options.paranoid
        || references::used(schema)
        || codec_methods.contains("w_i64")
        || codec_methods.contains("w_u64")
    {
        capabilities.insert("scalar_helpers");
    }

    if wrappers.contains("writeNumericInput_") || implementation.contains("numeric_set_0") {
        capabilities.insert("numeric_input");
        for kind in 0..10 {
            for copy in ["js", "native", "fill"] {
                if wrappers.contains(&format!("writeNumericInput_{kind}_{copy}(")) {
                    numeric.push_str(&bulk::numeric_input_writer(kind, copy, options.paranoid));
                }
            }
            if implementation.contains(&format!("numeric_set_{kind}")) {
                let element = [
                    "u8", "i8", "u16", "i16", "u32", "i32", "f32", "f64", "u32", "i32",
                ][kind];
                let (_, size, _, set) = bulk::NUMBERS.iter().find(|(n, ..)| *n == element).unwrap();
                let check = if options.paranoid {
                    format!("numericValue({kind},value);")
                } else {
                    String::new()
                };
                numeric.push_str(&format!("methods.numeric_set_{kind}=(view,index,value)=>{{{check}if(littleEndian||{size}===1)view[index]=value;else view.{set}(index*{size},value,true);}};\n"));
            }
        }
    }
    if capabilities.contains("reader")
        || capabilities.contains("callbacks")
        || capabilities.contains("buffered_string_error")
    {
        capabilities.insert("free_output");
    }
    let template = runtime_features(include_str!("runtime.mjs"), &capabilities);
    let global = format!("Ocamljs_{}", options.name);
    let host_imports = references::javascript(schema)?;
    let host_modules = references::javascript_modules(schema);
    let runtime = render_template(
        &template,
        &[
            (
                "__BYTE_STRINGS__",
                if !matches!(options.backend, Backend::Rescript) {
                    "true"
                } else {
                    "false"
                },
            ),
            ("__STAMP__", &packaging::stamp(schema)),
            (
                "__PARANOID__",
                if options.paranoid { "true" } else { "false" },
            ),
            ("__GLOBAL__", &global),
            ("__HOST_IMPORTS__", &host_imports),
            ("__HOST_MODULES__", &host_modules),
            ("__FAST_WRAPPERS__", &wrappers),
            ("__NUMERIC_CODECS__", &numeric),
            (
                "__NATIVE_STRING_COPY__",
                &serde_json::to_string(&native_strings::copier())?,
            ),
            (
                "__NATIVE_STRING_READ__",
                &serde_json::to_string(&native_strings::reader())?,
            ),
        ],
    );
    std::fs::write(out.join("bindings.runtime.mjs"), &runtime)?;
    Ok(runtime)
}

pub fn package_runtime(wasm: &[u8], runtime: &str, debug: bool) -> Result<Vec<u8>, Error> {
    let schema = read(wasm)?;
    let packaged = packaging::package(&schema, wasm, debug)?;
    packaging::prune_runtime(&packaged, runtime, debug)
}

pub fn schema_sidecar(wasm: &[u8]) -> Result<Vec<u8>, Error> {
    read(wasm)?;
    packaging::sidecar(wasm)
}

pub fn generate(schema: &Schema, wasm: &[u8], out: &Path, options: &Options) -> Result<(), Error> {
    // Validate caller-supplied metadata against the artifact before emitting code.
    let packaged = packaging::package(schema, wasm, options.debug)?;
    let input = serde_json::to_vec(&BindingInput {
        schema: schema.clone(),
        imports: import_names(&packaged)?,
    })?;
    let runtime = generate_bindings(&input, out, options)?;
    let output = packaging::prune_runtime(&packaged, &runtime, options.debug)?;
    std::fs::write(out.join("backend.wasm"), output)?;
    std::fs::write(out.join("schema.json"), packaging::sidecar(wasm)?)?;
    Ok(())
}
pub fn snake(name: &str) -> String {
    let mut s = String::new();
    let chars = name.chars().collect::<Vec<_>>();
    for (i, c) in chars.iter().enumerate() {
        if c.is_uppercase()
            && i > 0
            && (chars[i - 1].is_lowercase() || chars.get(i + 1).is_some_and(|c| c.is_lowercase()))
        {
            s.push('_');
        }
        s.extend(c.to_lowercase());
    }
    s
}
pub fn label(name: &str) -> String {
    let n = name.trim_start_matches("r#");
    if matches!(
        n,
        "type"
            | "method"
            | "object"
            | "class"
            | "open"
            | "default"
            | "val"
            | "mutable"
            | "done"
            | "end"
            | "begin"
            | "effect"
            | "match"
            | "to"
            | "from"
            | "as"
            | "assert"
            | "and"
            | "constraint"
            | "do"
            | "downto"
            | "else"
            | "exception"
            | "external"
            | "false"
            | "for"
            | "fun"
            | "function"
            | "if"
            | "in"
            | "include"
            | "inherit"
            | "initializer"
            | "lazy"
            | "let"
            | "module"
            | "new"
            | "nonrec"
            | "of"
            | "or"
            | "rec"
            | "sig"
            | "struct"
            | "then"
            | "true"
            | "try"
            | "virtual"
            | "when"
            | "while"
            | "with"
            | "mod"
            | "land"
            | "lor"
            | "lxor"
            | "lsl"
            | "lsr"
            | "asr"
    ) {
        format!("{n}_")
    } else {
        n.into()
    }
}

fn runtime_features(template: &str, features: &std::collections::BTreeSet<&str>) -> String {
    let mut enabled = vec![true];
    let mut output = String::new();
    for line in template.lines() {
        if let Some(feature) = line.trim().strip_prefix("// #if ") {
            enabled.push(*enabled.last().unwrap() && features.contains(feature));
        } else if line.trim() == "// #endif" {
            assert!(enabled.len() > 1, "unbalanced runtime feature block");
            enabled.pop();
        } else if *enabled.last().unwrap() {
            output.push_str(line);
            output.push('\n');
        }
    }
    assert_eq!(enabled.len(), 1, "unterminated runtime feature block");
    output
}

fn render_template(template: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = String::new();
    let mut remaining = template;
    while let Some((at, key, value)) = values
        .iter()
        .filter_map(|(key, value)| remaining.find(key).map(|at| (at, *key, *value)))
        .min_by_key(|v| v.0)
    {
        rendered.push_str(&remaining[..at]);
        rendered.push_str(value);
        remaining = &remaining[at + key.len()..];
    }
    rendered.push_str(remaining);
    rendered
}
