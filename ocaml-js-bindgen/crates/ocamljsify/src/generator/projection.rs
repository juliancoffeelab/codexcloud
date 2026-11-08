//! Explicit, globally consistent JavaScript representations for owned values.
use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
};

pub(super) fn named(name: &str) -> Ty {
    Ty {
        kind: "named".into(),
        name: name.into(),
        qualified: true,
        args: vec![],
        inner: None,
        len: None,
        mutable: false,
        map: String::new(),
    }
}
fn select(t: &mut Ty, selector: &str) {
    if !selector.is_empty() {
        t.map = format!("js:{selector}");
    }
}
fn selector(t: &Ty) -> &str {
    t.map.strip_prefix("js:").unwrap_or("")
}
fn plain(t: &Ty) -> Ty {
    let mut value = t.clone();
    value.map.clear();
    value
}
fn numeric(t: &Ty) -> bool {
    t.scalar() && !t.unit() && t.name != "bool"
}
fn integer_type(t: &Ty) -> bool {
    numeric(t) && !matches!(t.name.as_str(), "f32" | "f64")
}
fn sequence_element(t: &Ty) -> Option<&Ty> {
    if matches!(t.kind.as_str(), "array" | "slice") {
        t.inner.as_deref()
    } else if matches!(t.name.as_str(), "Vec" | "LinkedList") {
        t.args.first()
    } else {
        None
    }
}
fn typed_constructor(t: &Ty) -> Option<&'static str> {
    Some(match sequence_element(t)?.name.as_str() {
        "i8" => "Int8Array",
        "u8" => "Uint8Array",
        "i16" => "Int16Array",
        "u16" => "Uint16Array",
        "i32" | "isize" => "Int32Array",
        "u32" | "usize" => "Uint32Array",
        "f32" => "Float32Array",
        "f64" => "Float64Array",
        "i64" => "BigInt64Array",
        "u64" => "BigUint64Array",
        _ => return None,
    })
}
pub(super) fn raw_target(s: &Schema, t: &Ty) -> Option<Ty> {
    if t.scalar() {
        return Some(t.clone());
    }
    let entry = s.entries.get(&t.name)?;
    if entry.kind != "alias" || !entry.js_value {
        return None;
    }
    raw_target(s, entry.ty.as_ref()?)
}
fn raw_type(t: &Ty) -> &'static str {
    if t.unit() {
        "unit"
    } else {
        match t.name.as_str() {
            "bool" => "bool",
            "i64" | "u64" => "string",
            _ => "float",
        }
    }
}
fn raw_js(t: &Ty, value: &str, output: bool) -> String {
    if t.unit() {
        return "undefined".into();
    }
    let wide = matches!(t.name.as_str(), "i64" | "u64");
    if output {
        let base = if wide {
            if t.name == "u64" {
                format!("BigInt.asUintN(64,BigInt({value}))")
            } else {
                format!("BigInt({value})")
            }
        } else {
            value.into()
        };
        if selector(t) == "number" && wide {
            format!("js_safe_number({base})")
        } else if selector(t) == "bigint" && !wide {
            format!("BigInt({base})")
        } else {
            base
        }
    } else if wide {
        let base = if selector(t) == "number" {
            format!("js_number_bigint({value})")
        } else {
            value.into()
        };
        let (low, high) = if t.name == "u64" {
            ("0n", "18446744073709551615n")
        } else {
            ("(-9223372036854775808n)", "9223372036854775807n")
        };
        format!(
            "(()=>{{const x={base};if(typeof x!=='bigint'||x<{low}||x>{high})fail('integer out of range');return {}.toString();}})()",
            if t.name == "u64" {
                "BigInt.asIntN(64,x)"
            } else {
                "x"
            }
        )
    } else if selector(t) == "bigint" {
        format!("js_bigint_number({value})")
    } else {
        value.into()
    }
}
pub(super) fn marked(field: &Field) -> bool {
    field.js_value || field.js_value_rec
}
pub(super) fn success(ret: &Ty) -> &Ty {
    if ret.kind == "named" && ret.name == "Result" {
        &ret.args[0]
    } else {
        ret
    }
}
/// Resolve the explicit recursive escape hatch without modifying packaged metadata.
pub(super) fn resolve(schema: &Schema) -> Result<Schema, Error> {
    fn deep(s: &Schema, t: &Ty, selected: &mut BTreeSet<String>) -> Result<(), Error> {
        if let Some(entry) = s.entries.get(&t.name) {
            if !entry.is_type() {
                return Ok(());
            }
            if !matches!(entry.kind.as_str(), "struct" | "enum" | "alias") {
                return Err(
                    format!("js_value_rec cannot select {} ({})", entry.name, entry.kind).into(),
                );
            }
            if selected.insert(entry.name.clone()) {
                for ty in collections::fields(entry) {
                    deep(s, ty, selected)?;
                }
            }
        }
        for arg in &t.args {
            deep(s, arg, selected)?;
        }
        if let Some(inner) = &t.inner {
            deep(s, inner, selected)?;
        }
        Ok(())
    }
    let mut selected = BTreeSet::new();
    for entry in schema.entries.values() {
        if entry.js_value && entry.js_value_rec {
            return Err(format!("{}: choose js_value or js_value_rec once", entry.name).into());
        }
        for body in entry
            .body
            .iter()
            .chain(entry.variants.iter().map(|v| &v.body))
        {
            if body
                .fields
                .iter()
                .any(|field| field.js_value && field.js_value_rec)
            {
                return Err(
                    format!("{}: choose each field representation once", entry.name).into(),
                );
            }
        }
        if entry.js_value_rec {
            if entry.is_type() {
                deep(schema, &named(&entry.name), &mut selected)?;
            } else if let Some(ret) = &entry.result {
                deep(schema, success(ret), &mut selected)?;
            }
        }
        for body in entry
            .body
            .iter()
            .chain(entry.variants.iter().map(|v| &v.body))
        {
            for field in &body.fields {
                if field.js_value_rec {
                    deep(schema, &field.ty, &mut selected)?;
                }
            }
        }
    }
    let mut effective = schema.clone();
    for entry in effective.entries.values_mut() {
        entry.js_value |= entry.js_value_rec || selected.contains(&entry.name);
        if entry.js_value && !entry.js_selector.is_empty() {
            if entry.kind == "alias" {
                select(entry.ty.as_mut().unwrap(), &entry.js_selector);
            } else if entry.kind == "function" {
                let ret = entry.result.as_mut().unwrap();
                if ret.name == "Result" {
                    select(&mut ret.args[0], &entry.js_selector);
                } else {
                    select(ret, &entry.js_selector);
                }
            } else {
                let expected = if entry.kind == "enum"
                    && entry.variants.iter().all(|v| v.body.fields.is_empty())
                {
                    "number"
                } else {
                    "object"
                };
                if entry.js_selector != expected {
                    return Err(format!(
                        "{}: js_value({:?}) is incompatible; expected {expected}",
                        entry.name, entry.js_selector
                    )
                    .into());
                }
            }
        }
        for body in entry
            .body
            .iter_mut()
            .chain(entry.variants.iter_mut().map(|v| &mut v.body))
        {
            for field in &mut body.fields {
                field.js_value |= field.js_value_rec;
                if field.js_value {
                    select(&mut field.ty, &field.js_selector);
                }
            }
        }
    }
    fn explicit(s: &Schema, t: &Ty, at: &str) -> Result<(), Error> {
        if let Some(entry) = s.entries.get(&t.name) {
            if entry.is_type()
                && (!entry.js_value || !selector(t).is_empty() && selector(t) != entry.js_selector)
            {
                return Err(format!("{at}: js_value directly names {}; annotate that type with js_value, or select a container that retains its declared representation",entry.name).into());
            }
        }
        if t.kind == "ref" || t.mutable || references::host(s, t) {
            return Err(format!("{at}: js_value requires an owned structural value").into());
        }
        Ok(())
    }
    for entry in effective.entries.values() {
        if entry.is_type()
            && entry.js_value
            && (!matches!(entry.kind.as_str(), "struct" | "enum" | "alias"))
        {
            return Err(format!(
                "js_value requires a concrete structural declaration: {} ({})",
                entry.name, entry.kind
            )
            .into());
        }
        if entry.kind == "function" && entry.js_value {
            explicit(
                &effective,
                success(entry.result.as_ref().unwrap()),
                &entry.name,
            )?;
        }
        if entry.js_value
            && entry.kind == "enum"
            && entry.variants.iter().any(|v| !v.body.fields.is_empty())
        {
            let mut names = BTreeSet::from(["tag".to_owned()]);
            for variant in &entry.variants {
                if !names.insert(label(&snake(&variant.name))) {
                    return Err(
                        format!("{}: JavaScript variant property collision", entry.name).into(),
                    );
                }
            }
        }
        for body in entry
            .body
            .iter()
            .chain(entry.variants.iter().map(|v| &v.body))
        {
            for field in &body.fields {
                if field.js_value {
                    explicit(
                        &effective,
                        &field.ty,
                        &format!("{}.{}", entry.name, field.name),
                    )?;
                }
            }
        }
    }
    Ok(effective)
}
pub(super) fn active(schema: &Schema) -> bool {
    schema.entries.values().any(|entry| {
        entry.js_value
            || entry
                .body
                .iter()
                .chain(entry.variants.iter().map(|v| &v.body))
                .any(|body| body.fields.iter().any(marked))
    })
}
pub(super) fn key(t: &Ty) -> String {
    if t.kind == "named" && !t.name.is_empty() && t.args.is_empty() && selector(t).is_empty() {
        format!("named_{}", label(&snake(&t.name)))
    } else {
        format!(
            "value_{:x}",
            Sha256::digest(collections::identity(t).as_bytes())
        )
    }
}
/// Only native named boundaries become compiler callbacks. Anonymous builtin
/// structure stays JavaScript; a declaration's representation never depends on context.
fn generic(schema: &Schema, t: &Ty) -> bool {
    !t.qualified
        && schema
            .entries
            .values()
            .any(|e| e.params.iter().any(|p| p.as_str() == Some(&t.name)))
}
pub(super) fn template(entry: &Entry) -> Ty {
    let mut t = named(&entry.name);
    t.args = entry
        .params
        .iter()
        .filter_map(Value::as_str)
        .map(|name| {
            let mut t = named(name);
            t.qualified = false;
            t
        })
        .collect();
    t
}
pub(super) fn dependencies(schema: &Schema, t: &Ty) -> Vec<Ty> {
    fn walk(
        s: &Schema,
        t: &Ty,
        bindings: &BTreeMap<String, Ty>,
        seen: &mut BTreeSet<String>,
        out: &mut BTreeMap<String, Ty>,
    ) {
        if let Some(bound) = bindings.get(&t.name).filter(|_| !t.qualified) {
            out.entry(collections::identity(bound))
                .or_insert_with(|| bound.clone());
            return;
        }
        if generic(s, t) {
            out.entry(collections::identity(t))
                .or_insert_with(|| t.clone());
            return;
        }
        if let Some(entry) = s.entries.get(&t.name) {
            if !entry.js_value {
                let native = collections::substitute(t, bindings);
                out.entry(collections::identity(&native)).or_insert(native);
                return;
            }
            if !seen.insert(collections::identity(t)) {
                return;
            }
            let own = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(
                    t.args
                        .iter()
                        .map(|arg| collections::substitute(arg, bindings)),
                )
                .collect();
            for child in collections::fields(entry) {
                walk(s, child, &own, seen, out);
            }
            return;
        }
        for child in &t.args {
            walk(s, child, bindings, seen, out);
        }
        if let Some(inner) = &t.inner {
            walk(s, inner, bindings, seen, out);
        }
    }
    let mut out = BTreeMap::new();
    walk(schema, t, &BTreeMap::new(), &mut BTreeSet::new(), &mut out);
    out.into_values().collect()
}
#[derive(Default)]
pub(super) struct Projection {
    pub helpers: String,
    pub javascript: String,
}
struct Builder<'a> {
    schema: &'a Schema,
    options: &'a Options,
    maps: &'a collections::Maps,
    callbacks: Vec<Ty>,
    output: Projection,
}
impl Builder<'_> {
    fn jsoo(&self) -> bool {
        matches!(self.options.backend, Backend::JsOfOcaml)
    }
    fn number(&self) -> &'static str {
        if self.jsoo() {
            "Js_of_ocaml.Js.number_t"
        } else {
            "float"
        }
    }
    fn undefined(&self, ty: &str) -> String {
        format!(
            "({ty}) {}",
            if self.jsoo() {
                "Js_of_ocaml.Js.optdef"
            } else {
                "Js.Undefined.t"
            }
        )
    }
    fn property_name(&self, raw: &str) -> String {
        // Use each compiler's documented property escaping, retaining Rust keys.
        if matches!(self.options.backend, Backend::Rescript) {
            // bsc's OCaml syntax printer preserves method labels verbatim.
            // The packager restores these marked object-field tokens to raw
            // quoted keys in its authoritative ReScript source output.
            let mut name = String::from("__ocamljs_property_");
            for byte in raw.bytes() {
                write!(name, "{byte:02x}").unwrap();
            }
            return name;
        }
        let invalid_start = raw.starts_with(|c: char| !c.is_ascii_lowercase());
        if self.jsoo() {
            if raw.contains('_') {
                return format!("{}{raw}_", if invalid_start { "_" } else { "" });
            }
            if invalid_start || label(raw) != raw {
                return format!("_{raw}");
            }
        } else {
            if raw.contains("__") || raw.starts_with('_') {
                return format!("{raw}__");
            }
            if invalid_start || label(raw) != raw {
                return format!("_{raw}");
            }
        }
        raw.into()
    }
    fn object(&self, fields: Vec<(String, String)>) -> String {
        let fields = fields
            .into_iter()
            .map(|(name, ty)| {
                let name = self.property_name(&name);
                if self.jsoo() {
                    format!("{name} : ({ty}) Js_of_ocaml.Js.readonly_prop")
                } else {
                    format!("{name} : {ty}")
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!(
            "< {fields} > {}",
            if self.jsoo() {
                "Js_of_ocaml.Js.t"
            } else {
                "Js.t"
            }
        )
    }
    fn field_name(body: &Body, i: usize) -> String {
        if body.style == "named" {
            body.fields[i].name.clone()
        } else {
            i.to_string()
        }
    }
    fn body_type(&self, body: &Body) -> String {
        self.object(
            body.fields
                .iter()
                .enumerate()
                .map(|(i, f)| (Self::field_name(body, i), self.typ(&f.ty)))
                .collect(),
        )
    }
    fn compare(&self, t: &Ty, left: &str, right: &str) -> String {
        fn chain(parts: Vec<String>) -> String {
            parts
                .into_iter()
                .rev()
                .reduce(|rest, first| format!("({first}||{rest})"))
                .unwrap_or_else(|| "0".into())
        }
        if generic(self.schema, t) {
            return format!("c{}({left},{right})", self.callback(t));
        }
        if let Some(entry) = self.schema.entries.get(&t.name) {
            if !entry.js_value {
                return format!("c{}({left},{right})", self.callback(t));
            }
            let bindings = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(t.args.iter().cloned())
                .collect();
            if let Some(target) = &entry.ty {
                return self.compare(&collections::substitute(target, &bindings), left, right);
            }
            if let Some(body) = &entry.body {
                if entry.transparent {
                    return self.compare(
                        &collections::substitute(&body.fields[0].ty, &bindings),
                        left,
                        right,
                    );
                }
                return chain(
                    body.fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            self.compare(
                                &collections::substitute(&f.ty, &bindings),
                                &format!("js_field({left},{:?})", Self::field_name(body, i)),
                                &format!("js_field({right},{:?})", Self::field_name(body, i)),
                            )
                        })
                        .collect(),
                );
            }
            if entry.variants.iter().all(|v| v.body.fields.is_empty()) {
                return format!(
                    "(integer({left},0,{0})-integer({right},0,{0}))",
                    entry.variants.len().saturating_sub(1)
                );
            }
            let arms = entry
                .variants
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let name = label(&snake(&v.name));
                    let parts = v
                        .body
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(j, f)| {
                            self.compare(
                                &collections::substitute(&f.ty, &bindings),
                                &format!(
                                    "js_field(js_field({left},{name:?}),{:?})",
                                    Self::field_name(&v.body, j)
                                ),
                                &format!(
                                    "js_field(js_field({right},{name:?}),{:?})",
                                    Self::field_name(&v.body, j)
                                ),
                            )
                        })
                        .collect();
                    format!("case {i}:return {};", chain(parts))
                })
                .collect::<String>();
            return format!(
                "(()=>{{const a=integer(js_field({left},'tag'),0,{0}),b=integer(js_field({right},'tag'),0,{0});if(a!==b)return a-b;switch(a){{{arms}default:fail('invalid enum tag');}}}})()",
                entry.variants.len().saturating_sub(1)
            );
        }
        if t.unit() {
            return "0".into();
        }
        if t.name == "String" {
            return format!("js_compare_keys({left},{right})");
        }
        if t.scalar() {
            return format!("({left}<{right}?-1:{left}>{right}?1:0)");
        }
        if let Some(inner) = sequence_element(t) {
            return format!(
                "(()=>{{const a={left},b={right};for(let j=0;j<Math.min(a.length,b.length);j++){{const c={};if(c)return c;}}return a.length-b.length;}})()",
                self.compare(inner, "a[j]", "b[j]")
            );
        }
        if matches!(t.name.as_str(), "Box" | "Rc" | "Arc") {
            return self.compare(&t.args[0], left, right);
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.compare(&t.args[0], left, right);
            }
            return chain(
                t.args
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        self.compare(
                            t,
                            &format!("js_field({left},{i:?})"),
                            &format!("js_field({right},{i:?})"),
                        )
                    })
                    .collect(),
            );
        }
        if t.name == "Option" && matches!(selector(t), "nullable" | "optional") {
            let sentinel = if selector(t) == "nullable" {
                "null"
            } else {
                "undefined"
            };
            let mut inner = &t.args[0];
            while inner.name == "Option" {
                inner = &inner.args[0];
            }
            return format!(
                "({left}==={sentinel}?({right}==={sentinel}?0:-1):{right}==={sentinel}?1:{})",
                self.compare(inner, left, right)
            );
        }
        if matches!(t.name.as_str(), "Option" | "Result") {
            let first = if t.name == "Option" { "value" } else { "ok" };
            let second = if t.name == "Option" {
                "0".into()
            } else {
                self.compare(
                    &t.args[1],
                    &format!("js_field({left},'error')"),
                    &format!("js_field({right},'error')"),
                )
            };
            let payload = self.compare(
                &t.args[0],
                &format!("js_field({left},{first:?})"),
                &format!("js_field({right},{first:?})"),
            );
            return format!(
                "(()=>{{const a=integer(js_field({left},'tag'),0,1),b=integer(js_field({right},'tag'),0,1);if(a!==b)return a-b;return {};}})()",
                if t.name == "Option" {
                    format!("a===0?0:{payload}")
                } else {
                    format!("a===0?{payload}:{second}")
                }
            );
        }
        unreachable!("validated key")
    }
    fn map_mode(&self, t: &Ty) -> bool {
        selector(t) == "map"
            || (selector(t).is_empty() && !self.string_key(&t.args[0], &mut BTreeSet::new()))
    }
    fn bigint_type(&self) -> String {
        match self.options.backend {
            Backend::Melange => "Js.bigint".into(), Backend::Rescript => "bigint".into(),
            Backend::JsOfOcaml => "< toString : Js_of_ocaml.Js.js_string Js_of_ocaml.Js.t Js_of_ocaml.Js.meth > Js_of_ocaml.Js.t".into(),
        }
    }
    fn selected_type(&self, t: &Ty) -> Option<String> {
        Some(match selector(t) {
            "number" => self.number().into(),
            "bigint" => self.bigint_type(),
            "nullable" => {
                let mut inner = &t.args[0];
                while inner.name == "Option" {
                    inner = &inner.args[0];
                }
                format!(
                    "({}) {}",
                    self.typ(inner),
                    if self.jsoo() {
                        "Js_of_ocaml.Js.opt"
                    } else {
                        "Js.Nullable.t"
                    }
                )
            }
            "optional" => {
                let mut inner = &t.args[0];
                while inner.name == "Option" {
                    inner = &inner.args[0];
                }
                self.undefined(&self.typ(inner))
            }
            "typed_array" => {
                let constructor = typed_constructor(t).unwrap();
                if self.jsoo() {
                    if constructor.starts_with("Big") {
                        let element = self.bigint_type();
                        format!(
                            "< length : int Js_of_ocaml.Js.readonly_prop; at : int -> ({element}) Js_of_ocaml.Js.optdef Js_of_ocaml.Js.meth > Js_of_ocaml.Js.t"
                        )
                    } else {
                        let mut name = constructor.to_owned();
                        name[..1].make_ascii_lowercase();
                        format!("Js_of_ocaml.Typed_array.{name} Js_of_ocaml.Js.t")
                    }
                } else if constructor.starts_with("Big") {
                    self.object(vec![("length".into(), "int".into())])
                } else {
                    format!("Js.Typed_array.{constructor}.t")
                }
            }
            _ => return None,
        })
    }
    fn selected_decode(&self, t: &Ty) -> Option<String> {
        let original = plain(t);
        Some(match selector(t) {
            "number" if matches!(t.name.as_str(), "i64" | "u64") => {
                format!("js_safe_number({})", self.decode(&original))
            }
            "bigint" if !matches!(t.name.as_str(), "i64" | "u64") => {
                format!("BigInt({})", self.decode(&original))
            }
            "typed_array" => {
                let ctor = typed_constructor(t).unwrap();
                let inner = sequence_element(t).unwrap();
                let length = if t.kind == "array" {
                    if self.options.paranoid {
                        format!("const n={};methods.r_budget(r,n);", t.len.unwrap())
                    } else {
                        format!("const n={};", t.len.unwrap())
                    }
                } else {
                    "const n=methods.r_length(r);".into()
                };
                let get = match inner.name.as_str() {
                    "i8" => "getInt8",
                    "u8" => "getUint8",
                    "i16" => "getInt16",
                    "u16" => "getUint16",
                    "i32" | "isize" => "getInt32",
                    "u32" | "usize" => "getUint32",
                    "f32" => "getFloat32",
                    "f64" => "getFloat64",
                    "i64" => "getBigInt64",
                    "u64" => "getBigUint64",
                    _ => unreachable!(),
                };
                format!("(()=>{{{length}return readTyped(r,{ctor},n,{get:?});}})()")
            }
            "nullable" | "optional" => {
                let sentinel = if selector(t) == "nullable" {
                    "null"
                } else {
                    "undefined"
                };
                fn layers(b: &Builder<'_>, t: &Ty, sentinel: &str) -> String {
                    if t.name == "Option" {
                        format!(
                            "(()=>{{const tag=methods.r_u8(r);if(tag===0)return {sentinel};if(tag!==1)fail('invalid option tag');return {};}})()",
                            layers(b, &t.args[0], sentinel)
                        )
                    } else {
                        b.decode(t)
                    }
                }
                layers(self, &original, sentinel)
            }
            _ => return None,
        })
    }
    fn selected_encode(&self, t: &Ty, value: &str, depth: usize) -> Option<String> {
        let original = plain(t);
        Some(match selector(t) {
            "number" if matches!(t.name.as_str(), "i64" | "u64") => {
                let wide = format!("js_number_bigint({value})");
                self.encode_at(&original, &wide, depth + 1)
            }
            "bigint" if !matches!(t.name.as_str(), "i64" | "u64") => {
                let number = format!("js_bigint_number({value})");
                self.encode_at(&original, &number, depth + 1)
            }
            "typed_array" => {
                let ctor = typed_constructor(t).unwrap();
                let a = format!("a{depth}");

                let length = if t.kind == "array" {
                    format!(
                        "{}",
                        if self.options.paranoid {
                            format!(
                                "if({a}.length!=={})fail('fixed array length');methods.w_budget(w,{a}.length);",
                                t.len.unwrap()
                            )
                        } else {
                            String::new()
                        }
                    )
                } else {
                    format!("methods.w_length(w,{a}.length);")
                };
                let check = if self.options.paranoid {
                    format!("if(!({a} instanceof {ctor}))fail('expected {ctor}');")
                } else {
                    String::new()
                };
                let set = match sequence_element(t).unwrap().name.as_str() {
                    "i8" => "setInt8",
                    "u8" => "setUint8",
                    "i16" => "setInt16",
                    "u16" => "setUint16",
                    "i32" | "isize" => "setInt32",
                    "u32" | "usize" => "setUint32",
                    "f32" => "setFloat32",
                    "f64" => "setFloat64",
                    "i64" => "setBigInt64",
                    "u64" => "setBigUint64",
                    _ => unreachable!(),
                };
                format!("{{const {a}={value};{check}{length}writeTyped(w,{a},{set:?});}}")
            }
            "nullable" | "optional" => {
                let sentinel = if selector(t) == "nullable" {
                    "null"
                } else {
                    "undefined"
                };
                let mut inner = &original.args[0];
                let mut tags = String::from("methods.w_u8(w,1);");
                while inner.name == "Option" {
                    tags.push_str("methods.w_u8(w,1);");
                    inner = &inner.args[0];
                }
                format!(
                    "if({value}==={sentinel})methods.w_u8(w,0);else{{{tags}{}}}",
                    self.encode_at(inner, value, depth + 1)
                )
            }
            _ => return None,
        })
    }
    fn typ(&self, t: &Ty) -> String {
        if let Some(ty) = self.selected_type(t) {
            return ty;
        }
        if generic(self.schema, t) {
            return format!("'{}", label(&snake(&t.name)));
        }
        if t.unit() {
            return self.undefined("unit");
        }
        if t.kind == "array" || matches!(t.name.as_str(), "Vec" | "LinkedList") {
            let inner = if t.kind == "array" {
                t.inner.as_ref().unwrap()
            } else {
                &t.args[0]
            };
            return format!(
                "({}) {}",
                self.typ(inner),
                if self.jsoo() {
                    "Js_of_ocaml.Js.js_array Js_of_ocaml.Js.t"
                } else {
                    "array"
                }
            );
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.typ(&t.args[0]);
            }
            return self.object(
                t.args
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (i.to_string(), self.typ(t)))
                    .collect(),
            );
        }
        match t.name.as_str() {
            "String" | "str" => if self.jsoo() { "Js_of_ocaml.Js.js_string Js_of_ocaml.Js.t" } else { "string" }.into(),
            "bool" => if self.jsoo() { "bool Js_of_ocaml.Js.t" } else { "bool" }.into(),
            "i64" | "u64" => match self.options.backend {
                Backend::Melange => "Js.bigint",
                Backend::Rescript => "bigint",
                Backend::JsOfOcaml => "< toString : Js_of_ocaml.Js.js_string Js_of_ocaml.Js.t Js_of_ocaml.Js.meth > Js_of_ocaml.Js.t",
            }.into(),
            "BTreeMap" | "HashMap" if self.map_mode(t) => {
                let key = self.typ(&t.args[0]); let value = self.typ(&t.args[1]);
                if self.jsoo() {
                    format!("< size : int Js_of_ocaml.Js.readonly_prop; get : {key} -> ({value}) Js_of_ocaml.Js.optdef Js_of_ocaml.Js.meth; has : {key} -> bool Js_of_ocaml.Js.t Js_of_ocaml.Js.meth; set : {key} -> {value} -> < > Js_of_ocaml.Js.t Js_of_ocaml.Js.meth > Js_of_ocaml.Js.t")
                } else { format!("({key}, {value}) Js.Map.t") }
            }
            "BTreeMap" | "HashMap" => if self.jsoo() {
                "< > Js_of_ocaml.Js.t".into()
            } else { format!("({}) Js.Dict.t", self.typ(&t.args[1])) },
            "Option" => self.object(vec![("tag".into(), self.number().into()), ("value".into(), self.undefined(&self.typ(&t.args[0])))]),
            "Result" => self.object(vec![("tag".into(), self.number().into()), ("ok".into(), self.undefined(&self.typ(&t.args[0]))), ("error".into(), self.undefined(&self.typ(&t.args[1])))]),
            "Box" | "Rc" | "Arc" => self.typ(&t.args[0]),
            _ if t.scalar() => self.number().into(),
            _ => ml::type_name(self.schema,self.options,self.maps,t),
        }
    }
    fn string_key(&self, t: &Ty, seen: &mut BTreeSet<String>) -> bool {
        if t.kind != "named" {
            return false;
        }
        if t.name == "String" {
            return true;
        }
        if matches!(t.name.as_str(), "Box" | "Rc" | "Arc") {
            return self.string_key(&t.args[0], seen);
        }
        if !seen.insert(t.name.clone()) {
            return false;
        }
        self.schema.entries.get(&t.name).is_some_and(|e| {
            if e.kind == "alias" && e.params.is_empty() {
                return self.string_key(e.ty.as_ref().unwrap(), seen);
            }
            e.kind == "struct"
                && e.transparent
                && e.params.is_empty()
                && self.string_key(&e.body.as_ref().unwrap().fields[0].ty, seen)
        })
    }
    fn property(raw: &str) -> String {
        if raw == "__proto__" {
            format!("[{:?}]", raw)
        } else {
            format!("{:?}", raw)
        }
    }
    fn callback(&self, t: &Ty) -> usize {
        let identity = collections::identity(t);
        self.callbacks
            .iter()
            .position(|native| collections::identity(native) == identity)
            .expect("native boundary callback")
    }
    fn callback_args(&self, t: &Ty) -> String {
        let dependencies = if let Some(entry) = self
            .schema
            .entries
            .get(&t.name)
            .filter(|e| !e.params.is_empty())
        {
            let bindings = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(t.args.iter().cloned())
                .collect();
            dependencies(self.schema, &template(entry))
                .iter()
                .map(|native| collections::substitute(native, &bindings))
                .collect()
        } else {
            dependencies(self.schema, t)
        };
        dependencies
            .iter()
            .map(|native| format!(",c{}", self.callback(native)))
            .collect()
    }
    fn validate(&self, t: &Ty, seen: &mut BTreeSet<String>) -> Result<(), Error> {
        if let Some(entry) = self.schema.entries.get(&t.name) {
            if !entry.js_value {
                return Ok(());
            }
            if seen.insert(entry.name.clone()) {
                for field in collections::fields(entry) {
                    self.validate(field, seen)?;
                }
            }
            return Ok(());
        }
        if t.kind == "ref" || t.mutable || references::host(self.schema, t) {
            return Err("js_value requires owned values; select a named native boundary for borrowed/host data".into());
        }
        if selector(t) == "dict" && !self.string_key(&t.args[0], &mut BTreeSet::new()) {
            return Err("js_value(\"dict\") requires String keys".into());
        }
        let compatible = match selector(t) {
            "" => true,
            "number" => numeric(t),
            "bigint" => integer_type(t),
            "boolean" => t.name == "bool",
            "string" => matches!(t.name.as_str(), "String" | "str"),
            "array" => sequence_element(t).is_some(),
            "typed_array" => typed_constructor(t).is_some(),
            "dict" | "map" => matches!(t.name.as_str(), "BTreeMap" | "HashMap"),
            "nullable" | "optional" => t.name == "Option",
            "object" => t.kind == "tuple" && !t.args.is_empty(),
            _ => false,
        };
        if !compatible {
            return Err(format!(
                "js_value({:?}) is incompatible with {} {}",
                selector(t),
                t.kind,
                t.name
            )
            .into());
        }
        if matches!(t.name.as_str(), "BTreeMap" | "HashMap") {
            collections::key_check(
                self.schema,
                &t.args[0],
                t.name == "BTreeMap",
                &mut BTreeSet::new(),
            )?;
        }
        for arg in &t.args {
            self.validate(arg, seen)?;
        }
        if let Some(inner) = &t.inner {
            self.validate(inner, seen)?;
        }
        Ok(())
    }
    fn definition(&self, entry: &Entry) -> String {
        if entry.kind == "alias" {
            return self.typ(entry.ty.as_ref().unwrap());
        }
        if entry.kind == "struct" {
            if entry.transparent {
                return self.typ(&entry.body.as_ref().unwrap().fields[0].ty);
            }
            return self.body_type(entry.body.as_ref().unwrap());
        }
        if entry.variants.iter().all(|v| v.body.fields.is_empty()) {
            return self.number().into();
        }
        let mut fields = vec![("tag".into(), self.number().into())];
        for variant in &entry.variants {
            fields.push((
                label(&snake(&variant.name)),
                self.undefined(&self.body_type(&variant.body)),
            ));
        }
        self.object(fields)
    }
    // A span never crosses Wasm entry: selected graphs contain owned DTOs only.
    // Aliases add no depth; transparent structs retain their named boundary.
    fn span_read(
        &self,
        t: &Ty,
        offset: usize,
        start: &str,
        data: &str,
        seen: &mut BTreeSet<String>,
    ) -> Option<(usize, String)> {
        if !selector(t).is_empty() {
            return None;
        }
        let get = |method: &str| format!("{data}.{method}({start}+{offset},true)");
        if t.unit() {
            return Some((0, "undefined".into()));
        }
        let name = match t.name.as_str() {
            "usize" => "u32",
            "isize" => "i32",
            name => name,
        };
        match name {
            "String" | "str" => {
                return Some((
                    4,
                    format!("methods.r_pool_string(r,{},true)", get("getUint32")),
                ));
            }
            "bool" => return Some((1, format!("js_bool({})", get("getUint8")))),
            "i64" => return Some((8, get("getBigInt64"))),
            "u64" => return Some((8, get("getBigUint64"))),
            "Box" | "Rc" | "Arc" => return self.span_read(&t.args[0], offset, start, data, seen),
            _ => {}
        }
        if let Some((_, width, method, _)) = bulk::NUMBERS.iter().find(|(kind, ..)| *kind == name) {
            return Some((*width, get(method)));
        }
        if t.kind != "named" || !seen.insert(t.name.clone()) {
            return None;
        }
        let entry = self.schema.entries.get(&t.name)?;
        if !entry.js_value {
            return None;
        }
        let value = if entry.kind == "alias" {
            self.span_read(entry.ty.as_ref()?, offset, start, data, seen)
        } else if entry.transparent {
            self.span_read(&entry.body.as_ref()?.fields[0].ty, offset, start, data, seen).map(|(width,value)|
                (width, format!("(()=>{{methods.enter(r);try{{return {value};}}finally{{methods.leave(r);}}}})()")))
        } else {
            None
        };
        seen.remove(&t.name);
        value
    }
    fn body(&self, body: &Body) -> String {
        let mut statements = String::new();
        let mut index = 0;
        while index < body.fields.len() {
            let mut run = Vec::new();
            let mut width = 0;
            let start = format!("p{index}");
            let data = format!("d{index}");
            for field in &body.fields[index..] {
                let Some((size, value)) =
                    self.span_read(&field.ty, width, &start, &data, &mut BTreeSet::new())
                else {
                    break;
                };
                width += size;
                run.push(value);
            }
            if !run.is_empty() && width > 0 {
                write!(
                    statements,
                    "const {start}=methods.r_span(r,{width}),{data}=r.data;"
                )
                .unwrap();
                for (at, value) in run.iter().enumerate() {
                    write!(statements, "const f{}={value};", index + at).unwrap();
                }
                index += run.len();
            } else {
                write!(
                    statements,
                    "const f{index}={};",
                    self.decode(&body.fields[index].ty)
                )
                .unwrap();
                index += 1;
            }
        }
        let fields = body
            .fields
            .iter()
            .enumerate()
            .map(|(i, _)| format!("{}:f{i}", Self::property(&Self::field_name(body, i))))
            .collect::<Vec<_>>()
            .join(",");
        format!("(()=>{{{statements}return {{{fields}}};}})()")
    }
    fn sequence(&self, element: &Ty, length: &str) -> String {
        if let Some((width, value)) = self.span_read(element, 0, "at", "d", &mut BTreeSet::new()) {
            if width > 0 {
                return format!(
                    "(()=>{{{length}const a=new Array(n),p=methods.r_span(r,n*{width}),d=r.data;for(let j=0;j<n;j++){{const at=p+j*{width};a[j]={value};}}return a;}})()"
                );
            }
        }
        format!(
            "(()=>{{{length}const a=new Array(n);for(let j=0;j<n;j++)a[j]={};return a;}})()",
            self.decode(element)
        )
    }
    fn decode(&self, t: &Ty) -> String {
        if let Some(value) = self.selected_decode(t) {
            return value;
        }
        if generic(self.schema, t) {
            return format!("c{}(r)", self.callback(t));
        }
        if t.unit() {
            return "undefined".into();
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.decode(&t.args[0]);
            }
            return format!(
                "{{{}}}",
                t.args
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("\"{}\":{}", i, self.decode(t)))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        if t.kind == "array" {
            return self.sequence(
                t.inner.as_ref().unwrap(),
                &format!("const n={};methods.r_budget(r,n);", t.len.unwrap()),
            );
        }
        match t.name.as_str() {
            "String" | "str" => "methods.r_js_string(r)".into(),
            "usize" => "methods.r_u32(r)".into(),
            "isize" => "methods.r_i32(r)".into(),
            "i64" => "BigInt(methods.r_i64(r))".into(),
            "u64" => "BigInt.asUintN(64,BigInt(methods.r_u64(r)))".into(),
            _ if t.scalar() => format!("methods.r_{}(r)", t.name),
            "Vec" | "LinkedList" => self.sequence(&t.args[0], "const n=methods.r_length(r);"),
            "Option" => format!(
                "(()=>{{const tag=methods.r_u8(r);if(tag===0)return {{tag:0}};if(tag===1)return {{tag:1,value:{}}};fail('invalid option tag');}})()",
                self.decode(&t.args[0])
            ),
            "Result" => format!(
                "(()=>{{const tag=methods.r_u8(r);if(tag===0)return {{tag:0,ok:{}}};if(tag===1)return {{tag:1,error:{}}};fail('invalid result tag');}})()",
                self.decode(&t.args[0]),
                self.decode(&t.args[1])
            ),
            "BTreeMap" | "HashMap" if self.map_mode(t) => {
                format!(
                    "(()=>{{const n=methods.r_length(r),m=new Map();for(let j=0;j<n;j++){{const key={};if(m.has(key))fail('JS Map would collapse distinct Rust keys');const value={};m.set(key,value);}}return m;}})()",
                    self.decode(&t.args[0]),
                    self.decode(&t.args[1])
                )
            }
            "BTreeMap" | "HashMap" => {
                let ordering = if t.name == "BTreeMap" && self.options.paranoid {
                    "if(j>0&&js_compare_keys(previous,key)>=0)fail('map keys are not strictly increasing');previous=key;"
                } else {
                    ""
                };
                format!(
                    "(()=>{{const n=methods.r_length(r),d=Object.create(null);let previous;for(let j=0;j<n;j++){{const key={};{ordering}if(Object.hasOwn(d,key))fail('duplicate map key');d[key]={};}}return d;}})()",
                    self.decode_key(&t.args[0]),
                    self.decode(&t.args[1])
                )
            }
            "Box" | "Rc" | "Arc" => self.decode(&t.args[0]),
            _ => {
                let entry = &self.schema.entries[&t.name];
                if entry.js_value {
                    format!(
                        "js_read_type_{}(r{})",
                        label(&snake(&entry.name)),
                        self.callback_args(t)
                    )
                } else {
                    format!("c{}(r)", self.callback(t))
                }
            }
        }
    }
    fn read_entry(&self, entry: &Entry) -> String {
        match entry.kind.as_str() {
            "alias" => self.decode(entry.ty.as_ref().unwrap()),
            "struct" if entry.transparent => {
                self.decode(&entry.body.as_ref().unwrap().fields[0].ty)
            }
            "struct" => self.body(entry.body.as_ref().unwrap()),
            "enum" => {
                let fieldless = entry.variants.iter().all(|v| v.body.fields.is_empty());
                let branches = entry
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| {
                        let value = if fieldless {
                            index.to_string()
                        } else {
                            format!(
                                "{{tag:{index},{}:{}}}",
                                Self::property(&label(&snake(&variant.name))),
                                self.body(&variant.body)
                            )
                        };
                        format!("case {index}:return {value};")
                    })
                    .collect::<String>();
                format!(
                    "(()=>{{switch(methods.r_u32(r)){{{branches}default:fail('invalid enum tag');}}}})()"
                )
            }
            _ => unreachable!(),
        }
    }
    fn encode_key(&self, t: &Ty, value: &str) -> String {
        if t.name == "String" {
            return format!("methods.w_js_string(w,{value});");
        }
        if matches!(t.name.as_str(), "Box" | "Rc" | "Arc") {
            return self.encode_key(&t.args[0], value);
        }
        let entry = &self.schema.entries[&t.name];
        if entry.kind == "alias" {
            return self.encode_key(entry.ty.as_ref().unwrap(), value);
        }
        let body = self.encode_key(&entry.body.as_ref().unwrap().fields[0].ty, value);
        if self.options.paranoid {
            format!("methods.enter(w);try{{{body}}}finally{{methods.leave(w);}}")
        } else {
            body
        }
    }
    fn decode_key(&self, t: &Ty) -> String {
        if t.name == "String" {
            return "methods.r_js_string(r)".into();
        }
        if matches!(t.name.as_str(), "Box" | "Rc" | "Arc") {
            return self.decode_key(&t.args[0]);
        }
        let entry = &self.schema.entries[&t.name];
        if entry.kind == "alias" {
            return self.decode_key(entry.ty.as_ref().unwrap());
        }
        let value = self.decode_key(&entry.body.as_ref().unwrap().fields[0].ty);
        if self.options.paranoid {
            format!(
                "(()=>{{methods.enter(r);try{{return {value};}}finally{{methods.leave(r);}}}})()"
            )
        } else {
            value
        }
    }
    fn encode(&self, t: &Ty, value: &str) -> String {
        self.encode_at(t, value, 0)
    }
    fn encode_at(&self, t: &Ty, value: &str, depth: usize) -> String {
        if let Some(code) = self.selected_encode(t, value, depth) {
            return code;
        }
        if generic(self.schema, t) {
            return format!("c{}({value});", self.callback(t));
        }
        if let Some(entry) = self.schema.entries.get(&t.name) {
            return if entry.js_value {
                format!(
                    "js_write_type_{}(w,{value}{});",
                    label(&snake(&entry.name)),
                    self.callback_args(t)
                )
            } else {
                format!("c{}({value});", self.callback(t))
            };
        }
        if t.unit() {
            return if self.options.paranoid {
                format!("if({value}!==undefined)fail('expected JavaScript undefined');")
            } else {
                String::new()
            };
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.encode_at(&t.args[0], value, depth + 1);
            }
            return t
                .args
                .iter()
                .enumerate()
                .map(|(i, t)| self.encode_at(t, &format!("js_field({value},{i:?})"), depth + 1))
                .collect();
        }
        if t.kind == "array" || matches!(t.name.as_str(), "Vec" | "LinkedList") {
            let inner = if t.kind == "array" {
                t.inner.as_ref().unwrap()
            } else {
                &t.args[0]
            };
            let array = format!("a{depth}");
            let index = format!("j{depth}");
            let budget = if t.kind == "array" {
                format!(
                    "if({array}.length!=={})fail('fixed array length');methods.w_budget(w,{array}.length);",
                    t.len.unwrap()
                )
            } else {
                format!("methods.w_length(w,{array}.length);")
            };
            let guard = if self.options.paranoid {
                format!("if(!Array.isArray({array}))fail('expected JavaScript array');")
            } else {
                String::new()
            };
            return format!(
                "{{const {array}={value};{guard}{budget}for(let {index}=0;{index}<{array}.length;{index}++){{{}}}}}",
                self.encode_at(inner, &format!("{array}[{index}]"), depth + 1)
            );
        }
        match t.name.as_str() {
            "String" | "str" => format!("methods.w_js_string(w,{value});"),
            "i64" if !self.options.paranoid => format!("methods.w_i64(w,{value}.toString());"),
            "i64" => format!(
                "if(typeof {value}!=='bigint'||{value}<(-9223372036854775808n)||{value}>9223372036854775807n)fail('i64 out of range');methods.w_i64(w,{value}.toString());"
            ),
            "u64" if !self.options.paranoid => {
                format!("methods.w_u64(w,BigInt.asIntN(64,{value}).toString());")
            }
            "u64" => format!(
                "if(typeof {value}!=='bigint'||{value}<0n||{value}>18446744073709551615n)fail('u64 out of range');methods.w_u64(w,BigInt.asIntN(64,{value}).toString());"
            ),
            "usize" => format!("methods.w_u32(w,{value});"),
            "isize" => format!("methods.w_i32(w,{value});"),
            _ if t.scalar() => format!("methods.w_{}(w,{value});", t.name),
            "Box" | "Rc" | "Arc" => self.encode_at(&t.args[0], value, depth + 1),
            "Option" => {
                let object = format!("o{depth}");
                let tag = format!("tag{depth}");
                format!(
                    "{{const {object}={value},{tag}=js_field({object},'tag');if({tag}===0)methods.w_u8(w,0);else if({tag}===1){{methods.w_u8(w,1);{}}}else fail('invalid option tag');}}",
                    self.encode_at(
                        &t.args[0],
                        &format!("js_field({object},'value')"),
                        depth + 1
                    )
                )
            }
            "Result" => {
                let object = format!("o{depth}");
                let tag = format!("tag{depth}");
                format!(
                    "{{const {object}={value},{tag}=js_field({object},'tag');if({tag}===0){{methods.w_u8(w,0);{}}}else if({tag}===1){{methods.w_u8(w,1);{}}}else fail('invalid result tag');}}",
                    self.encode_at(&t.args[0], &format!("js_field({object},'ok')"), depth + 1),
                    self.encode_at(
                        &t.args[1],
                        &format!("js_field({object},'error')"),
                        depth + 1
                    )
                )
            }
            "BTreeMap" | "HashMap" if self.map_mode(t) => {
                let map = format!("m{depth}");
                let key = format!("k{depth}");
                let item = format!("v{depth}");
                format!(
                    "{{const {map}={value};if(!({map} instanceof Map))fail('expected JS Map');methods.w_length(w,{map}.size);for(const [{key},{item}] of Map.prototype.entries.call({map})){{{}{}}}}}",
                    self.encode_at(&t.args[0], &key, depth + 1),
                    self.encode_at(&t.args[1], &item, depth + 1)
                )
            }
            "BTreeMap" | "HashMap" => {
                let object = format!("o{depth}");
                let keys = format!("keys{depth}");
                let key = format!("key{depth}");
                let sort = if t.name == "BTreeMap" && self.options.paranoid {
                    format!("{keys}.sort(js_compare_keys);")
                } else {
                    String::new()
                };
                format!(
                    "{{const {object}={value};if({object}===null||typeof {object}!=='object'||Array.isArray({object}))fail('expected JavaScript dictionary');const {keys}=Object.keys({object});{sort}methods.w_length(w,{keys}.length);for(const {key} of {keys}){{{}{}}}}}",
                    self.encode_key(&t.args[0], &key),
                    self.encode_at(&t.args[1], &format!("{object}[{key}]"), depth + 1)
                )
            }
            _ => unreachable!(),
        }
    }
    fn write_body(&self, body: &Body, value: &str) -> String {
        body.fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                self.encode(
                    &field.ty,
                    &if self.options.paranoid {
                        format!("js_field({value},{:?})", Self::field_name(body, i))
                    } else {
                        format!("({value})[{:?}]", Self::field_name(body, i))
                    },
                )
            })
            .collect()
    }
    fn write_entry(&self, entry: &Entry) -> String {
        match entry.kind.as_str() {
            "alias" => self.encode(entry.ty.as_ref().unwrap(), "v"),
            "struct" if entry.transparent => {
                self.encode(&entry.body.as_ref().unwrap().fields[0].ty, "v")
            }
            "struct" => self.write_body(entry.body.as_ref().unwrap(), "v"),
            "enum" => {
                if entry.variants.iter().all(|v| v.body.fields.is_empty()) {
                    return format!(
                        "methods.w_u32(w,integer(v,0,{}));",
                        entry.variants.len().saturating_sub(1)
                    );
                }
                let branches = entry
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| {
                        format!(
                            "case {index}:{{{}break;}}",
                            self.write_body(
                                &variant.body,
                                &format!("js_field(v,{:?})", label(&snake(&variant.name)))
                            )
                        )
                    })
                    .collect::<String>();
                format!(
                    "const tag=js_field(v,'tag');methods.w_u32(w,integer(tag,0,{}));switch(tag){{{branches}default:fail('invalid enum tag');}}",
                    entry.variants.len().saturating_sub(1)
                )
            }
            _ => unreachable!(),
        }
    }
}

pub(super) fn js_type(
    schema: &Schema,
    options: &Options,
    maps: &collections::Maps,
    t: &Ty,
) -> String {
    Builder {
        schema,
        options,
        maps,
        callbacks: vec![],
        output: Projection::default(),
    }
    .typ(t)
}
pub(super) fn definition(
    schema: &Schema,
    options: &Options,
    maps: &collections::Maps,
    entry: &Entry,
) -> String {
    Builder {
        schema,
        options,
        maps,
        callbacks: vec![],
        output: Projection::default(),
    }
    .definition(entry)
}
pub(super) fn generate(
    schema: &Schema,
    options: &Options,
    maps: &collections::Maps,
) -> Result<Projection, Error> {
    let mut b = Builder {
        schema,
        options,
        maps,
        callbacks: vec![],
        output: Projection::default(),
    };
    if !active(schema) {
        return Ok(b.output);
    }
    b.output.javascript.push_str("function js_safe_number(v){const n=Number(v);if(!Number.isSafeInteger(n))fail('integer exceeds JS safe range');return n;}function js_number_bigint(v){if(typeof v!=='number'||!Number.isSafeInteger(v))fail('expected exact safe integer Number');return BigInt(v);}function js_bigint_number(v){if(typeof v!=='bigint')fail('expected BigInt');return js_safe_number(v); }\n");
    b.output.javascript.push_str("function js_field(o,k){if(o===null||typeof o!=='object'||!Object.hasOwn(o,k))fail('missing JavaScript property');return o[k];}function js_bool(value){if(value!==0&&value!==1)fail('invalid bool');return value===1;}\n");
    b.output.javascript.push_str("function js_compare_keys(a,b){let i=0,j=0;while(i<a.length&&j<b.length){const x=a.codePointAt(i),y=b.codePointAt(j);if(x!==y)return x<y?-1:1;i+=x>65535?2:1;j+=y>65535?2:1;}return i<a.length?1:j<b.length?-1:0;}\n");
    if !options.paranoid {
        b.output.javascript=b.output.javascript.replace("if(o===null||typeof o!=='object'||!Object.hasOwn(o,k))fail('missing JavaScript property');", "").replace("if(value!==0&&value!==1)fail('invalid bool');", "");
        b.output.javascript=b.output.javascript.replace("if(!Number.isSafeInteger(n))fail('integer exceeds JS safe range');", "").replace("if(typeof v!=='number'||!Number.isSafeInteger(v))fail('expected exact safe integer Number');", "").replace("if(typeof v!=='bigint')fail('expected BigInt');", "");
    }
    let mut roots = BTreeMap::new();
    for entry in schema.entries.values() {
        if entry.kind == "function" && entry.asynchronous && !entry.throws {
            let ret = entry.result.as_ref().unwrap();
            if abi::text_result(schema, ret).is_some() && abi::text_result_js(schema, ret) {
                let error = abi::resolve(schema, ret).args[1].clone();
                if raw_target(schema, &error).is_some() {
                    roots.insert(key(&error), error);
                }
            }
        }
        if entry.is_type() && entry.js_value {
            let t = template(entry);
            roots.insert(key(&t), t);
        }
        if entry.kind == "function" && entry.js_value {
            let t = success(entry.result.as_ref().unwrap()).clone();
            roots.insert(key(&t), t);
        }
        for body in entry
            .body
            .iter()
            .chain(entry.variants.iter().map(|v| &v.body))
        {
            for field in &body.fields {
                if field.js_value || entry.js_value {
                    roots.insert(key(&field.ty), field.ty.clone());
                }
            }
        }
    }
    for entry in schema
        .entries
        .values()
        .filter(|e| e.is_type() && e.js_value)
    {
        let t = template(entry);
        b.validate(&t, &mut BTreeSet::new())?;
        b.callbacks = dependencies(schema, &t);
        let args = (0..b.callbacks.len())
            .map(|i| format!(",c{i}"))
            .collect::<String>();
        let read = b.read_entry(entry);
        let write = b.write_entry(entry);
        let name = label(&snake(&entry.name));
        if entry.kind == "alias" || !options.paranoid {
            writeln!(
                b.output.javascript,
                "function js_read_type_{name}(r{args}){{return {read};}}function js_write_type_{name}(w,v{args}){{{write}}}"
            )?;
        } else {
            writeln!(
                b.output.javascript,
                "function js_read_type_{name}(r{args}){{methods.enter(r);try{{return {read};}}finally{{methods.leave(r);}}}}function js_write_type_{name}(w,v{args}){{methods.enter(w);try{{{write}}}finally{{methods.leave(w);}}}}"
            )?;
        }
    }
    for entry in schema.entries.values().filter(|e| e.flat && e.js_value) {
        let body = entry.body.as_ref().unwrap();
        let ty = ml::type_name(schema, options, maps, &named(&entry.name));
        let args = body.fields.iter().map(|f| b.typ(&f.ty)).collect::<Vec<_>>();
        let values = (0..args.len()).map(|i| format!("v{i}")).collect::<Vec<_>>();
        let name = label(&snake(&entry.name));
        b.output.helpers.push_str(&ml::external(
            options,
            &format!("flat_make_{name}"),
            &format!("flat_make_{name}"),
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
            &ty,
        ));
        let value = if entry.transparent {
            "v0".into()
        } else {
            format!(
                "{{{}}}",
                body.fields
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!(
                        "{}:v{i}",
                        Builder::property(&Builder::field_name(body, i))
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };
        writeln!(
            b.output.javascript,
            "methods.flat_make_{name}=({})=>({value});",
            values.join(",")
        )?;
        for (i, field) in body.fields.iter().enumerate() {
            b.output.helpers.push_str(&ml::external(
                options,
                &format!("flat_get_{name}_{i}"),
                &format!("flat_get_{name}_{i}"),
                &[&ty],
                &b.typ(&field.ty),
            ));
            let value = if entry.transparent {
                "v".into()
            } else {
                format!("js_field(v,{:?})", Builder::field_name(body, i))
            };
            writeln!(
                b.output.javascript,
                "methods.flat_get_{name}_{i}=v=>({value});"
            )?;
        }
    }
    for (name, t) in roots {
        b.validate(&t, &mut BTreeSet::new())?;
        b.callbacks = dependencies(schema, &t);
        let ty = b.typ(&t);
        if let Some(raw) = raw_target(schema, &t) {
            let input = raw_type(&raw);
            b.output.helpers.push_str(&ml::external(
                options,
                &format!("raw_read_{name}"),
                &format!("raw_read_{name}"),
                &[input],
                &ty,
            ));
            b.output.helpers.push_str(&ml::external(
                options,
                &format!("raw_write_{name}"),
                &format!("raw_write_{name}"),
                &[&ty],
                input,
            ));
            writeln!(
                b.output.javascript,
                "methods.raw_read_{name}=v=>({});methods.raw_write_{name}=v=>({});",
                raw_js(&raw, "v", true),
                raw_js(&raw, "v", false)
            )?;
        }
        if collections::key_check(schema, &t, true, &mut BTreeSet::new()).is_ok() {
            let callback_types = b
                .callbacks
                .iter()
                .map(|native| {
                    let ty = ml::type_name(schema, options, maps, native);
                    format!("({ty} -> {ty} -> int)")
                })
                .collect::<Vec<_>>();
            let args = std::iter::once(ty.clone())
                .chain(std::iter::once(ty.clone()))
                .chain(callback_types)
                .collect::<Vec<_>>();
            b.output.helpers.push_str(&ml::external(
                options,
                &format!("compare_js_{name}"),
                &format!("compare_js_{name}"),
                &args.iter().map(String::as_str).collect::<Vec<_>>(),
                "int",
            ));
            let callbacks = (0..b.callbacks.len())
                .map(|i| format!(",c{i}"))
                .collect::<String>();
            writeln!(
                b.output.javascript,
                "methods.compare_js_{name}=(a,b{callbacks})=>({});",
                b.compare(&t, "a", "b")
            )?;
        }
        let callback_args = (0..b.callbacks.len())
            .map(|i| format!(",c{i}"))
            .collect::<String>();
        let reader_types = std::iter::once("reader".into())
            .chain(b.callbacks.iter().map(|native| {
                format!(
                    "(reader -> {})",
                    ml::type_name(schema, options, maps, native)
                )
            }))
            .collect::<Vec<String>>();
        b.output.helpers.push_str(&ml::external(
            options,
            &format!("r_js_{name}"),
            &format!("r_js_{name}"),
            &reader_types.iter().map(String::as_str).collect::<Vec<_>>(),
            &ty,
        ));
        let writer_types = vec!["writer".into(), ty]
            .into_iter()
            .chain(b.callbacks.iter().map(|native| {
                format!("({} -> unit)", ml::type_name(schema, options, maps, native))
            }))
            .collect::<Vec<String>>();
        b.output.helpers.push_str(&ml::external(
            options,
            &format!("w_js_{name}"),
            &format!("w_js_{name}"),
            &writer_types.iter().map(String::as_str).collect::<Vec<_>>(),
            "unit",
        ));
        writeln!(
            b.output.javascript,
            "methods.r_js_{name}=(r{callback_args})=>({});methods.w_js_{name}=(w,v{callback_args})=>{{{}}};",
            b.decode(&t),
            b.encode(&t, "v")
        )?;
    }
    if !options.paranoid {
        b.output.javascript = b
            .output
            .javascript
            .replace("if(Object.hasOwn(d,key))fail('duplicate map key');", "");
    }
    Ok(b.output)
}
