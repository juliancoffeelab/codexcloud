use super::*;
use std::collections::{BTreeMap, BTreeSet};
fn identifier(s: &str) -> bool {
    let mut c = s.chars();
    c.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && c.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn body(b: &Body) -> Result<(), Error> {
    if !matches!(b.style.as_str(), "named" | "tuple" | "unit") {
        return Err("invalid body style".into());
    }
    let mut names = BTreeSet::new();
    for f in &b.fields {
        if b.style == "named"
            && (!identifier(&f.name)
                || f.name.starts_with(|c: char| c.is_ascii_uppercase())
                || !names.insert(label(&f.name)))
        {
            return Err(format!("invalid or colliding field {}", f.name).into());
        }
    }
    if b.style == "unit" && !b.fields.is_empty() {
        return Err("unit body cannot have fields".into());
    }
    Ok(())
}
fn alias_cycles(s: &Schema) -> Result<(), Error> {
    fn dependencies(s: &Schema, t: &Ty, params: &[Value], out: &mut BTreeSet<String>) {
        if t.kind == "named"
            && (t.qualified || !params.iter().any(|p| p.as_str() == Some(t.name.as_str())))
            && s.entries.get(&t.name).is_some_and(|e| e.kind == "alias")
        {
            out.insert(t.name.clone());
        }
        for arg in &t.args {
            dependencies(s, arg, params, out);
        }
        if let Some(inner) = &t.inner {
            dependencies(s, inner, params, out);
        }
    }
    // Alias expansion cannot recurse, even through type arguments or Box.
    // Nominal bodies stop expansion: alias -> struct -> alias is legal.
    let graph = s
        .entries
        .values()
        .filter(|e| e.kind == "alias")
        .map(|e| {
            let mut deps = BTreeSet::new();
            if let Some(target) = &e.ty {
                dependencies(s, target, &e.params, &mut deps);
            }
            (e.name.clone(), deps)
        })
        .collect::<BTreeMap<_, _>>();
    for group in collections::groups(&graph) {
        if group.len() > 1 || graph[&group[0]].contains(&group[0]) {
            return Err(format!("recursive type alias: {}", group.join(", ")).into());
        }
    }
    Ok(())
}
pub(super) fn structure(s: &Schema) -> Result<(), Error> {
    let mut types = BTreeMap::new();
    let mut functions = BTreeMap::new();
    let mut constructors = BTreeSet::new();
    for e in s.entries.values() {
        if !identifier(&e.name) {
            return Err(format!("invalid ABI identifier {}", e.name).into());
        }
        if e.kind == "module" {
            continue; // Module namespaces are checked together with their members.
        }
        if e.kind == "import" {
            continue;
        }
        if e.kind == "function" {
            if e.name.starts_with(|c: char| c.is_ascii_uppercase()) {
                return Err(
                    "exported function names must start with a lowercase letter or underscore"
                        .into(),
                );
            }
            let key = label(&e.name);
            if functions.insert(key, e.name.clone()).is_some() {
                return Err("colliding function names".into());
            }
            if e.result.is_none() {
                return Err(format!("{}: function missing result type", e.name).into());
            }
            for p in &e.params {
                let p: Field = serde_json::from_value(p.clone())?;
                if !identifier(&p.name) || p.ty.unit() {
                    return Err("invalid function argument; omit explicit unit parameters".into());
                }
            }
        } else {
            if !e
                .name
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_uppercase())
            {
                return Err(
                    "exported Rust type names must start with an uppercase ASCII letter".into(),
                );
            }
            if !matches!(
                e.kind.as_str(),
                "struct" | "enum" | "alias" | "opaque" | "host"
            ) {
                return Err(format!("unknown schema entry kind {}", e.kind).into());
            }
            let key = label(&snake(&e.name));
            if matches!(
                key.as_str(),
                "writer"
                    | "reader"
                    | "raw_task"
                    | "ocamljs_task"
                    | "js_promise"
                    | "unit"
                    | "bool"
                    | "char"
                    | "string"
                    | "bytes"
                    | "int"
                    | "int32"
                    | "int64"
                    | "nativeint"
                    | "float"
                    | "array"
                    | "list"
                    | "option"
                    | "result"
                    | "exn"
                    | "lazy_t"
            ) || matches!(
                e.name.as_str(),
                "Callback"
                    | "bool"
                    | "u8"
                    | "i8"
                    | "u16"
                    | "i16"
                    | "u32"
                    | "i32"
                    | "u64"
                    | "i64"
                    | "usize"
                    | "isize"
                    | "f32"
                    | "f64"
                    | "String"
                    | "Vec"
                    | "LinkedList"
                    | "Option"
                    | "Result"
                    | "Box"
                    | "Rc"
                    | "Arc"
                    | "HashMap"
                    | "BTreeMap"
            ) {
                return Err(format!("{} collides with an ABI built-in type", e.name).into());
            }
            if let Some(other) = types.insert(key, e.name.clone()) {
                return Err(
                    format!("normalized type names collide: {other} and {}", e.name).into(),
                );
            }
            let mut ps = BTreeSet::new();
            for p in &e.params {
                let n = p.as_str().ok_or("generic parameters must be names")?;
                if !identifier(n)
                    || matches!(
                        n,
                        "bool"
                            | "u8"
                            | "i8"
                            | "u16"
                            | "i16"
                            | "u32"
                            | "i32"
                            | "u64"
                            | "i64"
                            | "usize"
                            | "isize"
                            | "f32"
                            | "f64"
                            | "String"
                            | "Vec"
                            | "LinkedList"
                            | "Option"
                            | "Result"
                            | "Box"
                            | "Rc"
                            | "Arc"
                            | "HashMap"
                            | "BTreeMap"
                    )
                    || !ps.insert(label(&snake(n)))
                {
                    return Err("invalid or colliding generic parameters".into());
                }
            }
            if e.flat && e.kind != "struct" {
                return Err("only structs may declare a flat ABI".into());
            }
            match e.kind.as_str() {
                "struct" => {
                    let b = e.body.as_ref().ok_or("struct missing body")?;
                    body(b)?;
                    if e.flat
                        && (!e.params.is_empty()
                            || b.fields.is_empty()
                            || b.fields.len() > 8
                            || b.fields.iter().any(|f| !f.ty.scalar() || f.ty.unit()))
                    {
                        return Err("invalid flat record ABI".into());
                    }
                    if e.transparent && b.fields.len() != 1 {
                        return Err("transparent requires exactly one field".into());
                    }
                }
                "alias" => {
                    if e.ty.is_none() {
                        return Err("alias missing target type".into());
                    }
                }
                "enum" => {
                    for v in &e.variants {
                        if !identifier(&v.name)
                            || !constructors.insert(format!(
                                "{}_{}",
                                snake(&e.name),
                                snake(&v.name)
                            ))
                        {
                            return Err("invalid or colliding variant names".into());
                        }
                        body(&v.body)?;
                    }
                }
                "opaque" | "host" => {
                    if !e.params.is_empty() {
                        return Err("opaque types must be concrete".into());
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    alias_cycles(s)
}
fn substitute(t: &Ty, bindings: &BTreeMap<String, Ty>) -> Ty {
    if !t.qualified {
        if let Some(t) = bindings.get(&t.name) {
            return t.clone();
        }
    }
    let mut t = t.clone();
    t.args = t.args.iter().map(|t| substitute(t, bindings)).collect();
    t.inner = t.inner.as_ref().map(|i| Box::new(substitute(i, bindings)));
    t
}
pub(super) fn contains_opaque(s: &Schema, t: &Ty, seen: &mut BTreeSet<String>) -> bool {
    if seen.len() > 128 {
        return true;
    }
    if let Some(i) = &t.inner {
        if contains_opaque(s, i, seen) {
            return true;
        }
    }
    if t.args.iter().any(|a| contains_opaque(s, a, seen)) {
        return true;
    }
    let Some(e) = s.entries.get(&t.name) else {
        return false;
    };
    if e.kind == "opaque" {
        return true;
    }
    let key = serde_json::to_string(t).unwrap();
    if !seen.insert(key) {
        return false;
    }
    let bindings = e
        .params
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .zip(t.args.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    if let Some(t) = &e.ty {
        if contains_opaque(s, &substitute(t, &bindings), seen) {
            return true;
        }
    }
    if let Some(b) = &e.body {
        for f in &b.fields {
            if contains_opaque(s, &substitute(&f.ty, &bindings), seen) {
                return true;
            }
        }
    }
    for v in &e.variants {
        for f in &v.body.fields {
            if contains_opaque(s, &substitute(&f.ty, &bindings), seen) {
                return true;
            }
        }
    }
    false
}
fn raw(t: &Ty) -> Option<wasmparser::ValType> {
    use wasmparser::ValType::*;
    if t.unit() {
        None
    } else {
        Some(match t.name.as_str() {
            "i64" | "u64" => I64,
            "f32" => F32,
            "f64" => F64,
            _ => I32,
        })
    }
}
pub(super) fn wasm_exports(s: &Schema, wasm: &[u8]) -> Result<(), Error> {
    use wasmparser::{ExternalKind, Payload, TypeRef, ValType};
    let mut types = vec![];
    let mut functions = vec![];
    let mut exports = BTreeMap::new();
    let mut memory = false;
    let reference_abi = references::lowered(wasm);
    let mut marker_seen = false;
    let mut pruned = false;
    for p in wasmparser::Parser::new(0).parse_all(wasm) {
        match p? {
            Payload::CustomSection(c) if c.name() == super::packaging::PRUNED_MARKER => {
                if c.data() != [1] || pruned {
                    return Err("invalid pruned runtime marker".into());
                }
                pruned = true;
            }
            Payload::CustomSection(c) if c.name() == references::MARKER => {
                if c.data() != [1] || marker_seen {
                    return Err("invalid reference ABI marker".into());
                }
                marker_seen = true;
            }
            Payload::TypeSection(r) => {
                for t in r.into_iter_err_on_gc_types() {
                    types.push(t?);
                }
            }
            Payload::ImportSection(r) => {
                for i in r {
                    let i = i?;
                    match i.ty {
                        TypeRef::Func(t) if i.module == "ocamljs_host" => {
                            let e = s
                                .entries
                                .get(i.name)
                                .filter(|e| e.kind == "import")
                                .ok_or("undescribed host import")?;
                            let (p, r) = references::import_signature(s, e, reference_abi)?;
                            let convert = |t: wasm_encoder::ValType| match t {
                                wasm_encoder::ValType::I32 => ValType::I32,
                                wasm_encoder::ValType::I64 => ValType::I64,
                                wasm_encoder::ValType::F32 => ValType::F32,
                                wasm_encoder::ValType::F64 => ValType::F64,
                                _ => ValType::EXTERNREF,
                            };
                            let f = types.get(t as usize).ok_or("invalid host import type")?;
                            if f.params() != p.into_iter().map(convert).collect::<Vec<_>>()
                                || f.results() != r.into_iter().map(convert).collect::<Vec<_>>()
                            {
                                return Err("host import signature mismatch".into());
                            }
                            functions.push(t);
                        }
                        TypeRef::Func(t) if i.module == "ocamljs_ref" && !reference_abi => {
                            let (p, r) = match i.name {
                                "clone" => (vec![ValType::I32], vec![ValType::I32]),
                                "drop" => (vec![ValType::I32], vec![]),
                                _ => return Err("unknown reference intrinsic".into()),
                            };
                            let f = types.get(t as usize).ok_or("invalid reference intrinsic")?;
                            if f.params() != p || f.results() != r {
                                return Err("reference intrinsic signature mismatch".into());
                            }
                            functions.push(t);
                        }
                        TypeRef::Func(t) if i.module == "ocamljs" => {
                            use ValType::I32;
                            let (params, results): (Vec<_>, Vec<_>) = match i.name {
                                "boundary_error" => (vec![I32, I32], vec![]),
                                "task_wake" | "callback_release" | "fetch_close"
                                | "fetch_credit" => (vec![I32], vec![]),
                                "fetch_open" => (vec![I32, I32, I32], vec![]),
                                "callback_retain" => (vec![I32], vec![I32]),
                                name if references::intrinsic(name, reference_abi).is_some() => {
                                    let (p, r) =
                                        references::intrinsic(name, reference_abi).unwrap();
                                    let convert = |t| match t {
                                        wasm_encoder::ValType::EXTERNREF => ValType::EXTERNREF,
                                        _ => I32,
                                    };
                                    (
                                        p.into_iter().map(convert).collect(),
                                        r.into_iter().map(convert).collect(),
                                    )
                                }
                                _ => {
                                    return Err(format!(
                                        "unsupported Wasm import {}.{}",
                                        i.module, i.name
                                    )
                                    .into());
                                }
                            };
                            let f = types.get(t as usize).ok_or("invalid import type")?;
                            if f.params() != params || f.results() != results {
                                return Err(format!("{} import signature mismatch", i.name).into());
                            }
                            functions.push(t);
                        }
                        _ => {
                            return Err(
                                format!("unsupported Wasm import {}.{}", i.module, i.name).into()
                            );
                        }
                    }
                }
            }
            Payload::FunctionSection(r) => {
                for f in r {
                    functions.push(f?);
                }
            }
            Payload::ExportSection(r) => {
                for e in r {
                    let e = e?;
                    if e.kind == ExternalKind::Func {
                        exports.insert(e.name.to_string(), e.index);
                    }
                    if e.kind == ExternalKind::Memory && e.name == "memory" {
                        memory = true;
                    }
                }
            }
            Payload::MemorySection(r) => {
                for m in r {
                    let m = m?;
                    if m.memory64 || m.shared {
                        return Err("only single-threaded wasm32 memory is supported".into());
                    }
                }
            }
            _ => {}
        }
    }
    if !memory {
        return Err("Wasm must export memory".into());
    }
    let check = |name: &str, params: Vec<ValType>, results: Vec<ValType>| -> Result<(), Error> {
        let idx = *exports
            .get(name)
            .ok_or_else(|| format!("Wasm missing export {name}"))?;
        let ty = *functions
            .get(idx as usize)
            .ok_or("invalid function index")?;
        let f = types.get(ty as usize).ok_or("invalid function type")?;
        if f.params() != params || f.results() != results {
            return Err(format!("Wasm export {name} has an incompatible ABI signature").into());
        }
        Ok(())
    };
    use ValType::I32;
    for (name, params, results) in [
        ("__ocamljs_abi", vec![], vec![I32]),
        ("__ocamljs_alloc", vec![I32], vec![I32]),
        ("__ocamljs_string_alloc", vec![I32], vec![I32]),
        ("__ocamljs_string_ascii", vec![I32, I32], vec![I32]),
        ("__ocamljs_string_realloc", vec![I32, I32, I32], vec![I32]),
        ("__ocamljs_string_free", vec![I32, I32], vec![]),
        ("__ocamljs_numeric_alloc", vec![I32, I32], vec![I32]),
        ("__ocamljs_numeric_free", vec![I32, I32, I32], vec![]),
        ("__ocamljs_input_ptr", vec![I32], vec![I32]),
        ("__ocamljs_input_free", vec![I32], vec![]),
        ("__ocamljs_output_info", vec![I32], vec![I32]),
        ("__ocamljs_output_ptr", vec![I32], vec![I32]),
        ("__ocamljs_output_len", vec![I32], vec![I32]),
        ("__ocamljs_output_pool_ptr", vec![I32], vec![I32]),
        ("__ocamljs_output_pool_len", vec![I32], vec![I32]),
        ("__ocamljs_output_free", vec![I32], vec![]),
        ("__ocamljs_claim", vec![I32, I32], vec![I32]),
        ("__ocamljs_dispose", vec![I32], vec![I32]),
        ("__ocamljs_moved", vec![], vec![I32]),
    ] {
        if !pruned || name == "__ocamljs_abi" || exports.contains_key(name) {
            check(name, params, results)?;
        }
    }
    // Raw Rust artifacts root hooks only from explicit async declarations.
    if exports.contains_key("__ocamljs_host_take") {
        check("__ocamljs_host_take", vec![I32], vec![ValType::EXTERNREF])?;
    }
    if exports.contains_key("__ocamljs_host_drop") {
        check("__ocamljs_host_drop", vec![I32], vec![])?;
    }
    // Packaged artifacts canonicalize one pair and may prune unused hooks.
    for name in exports.keys().filter(|name| {
        matches!(
            name.as_str(),
            "__ocamljs_task_poll" | "__ocamljs_task_cancel" | "__ocamljs_task_ready"
        ) || name.starts_with("__ocamljs_task_poll_for_")
            || name.starts_with("__ocamljs_task_cancel_for_")
    }) {
        check(name, vec![I32], vec![I32])?;
    }
    if !pruned && s.entries.values().any(|e| e.asynchronous) {
        if exports.contains_key("__ocamljs_task_poll")
            || exports.contains_key("__ocamljs_task_cancel")
        {
            check("__ocamljs_task_poll", vec![I32], vec![I32])?;
            check("__ocamljs_task_cancel", vec![I32], vec![I32])?;
        } else {
            for entry in s.entries.values().filter(|e| e.asynchronous) {
                for hook in ["poll", "cancel"] {
                    check(
                        &format!("__ocamljs_task_{hook}_for_{}", entry.name),
                        vec![I32],
                        vec![I32],
                    )?;
                }
            }
        }
    }
    for e in s.entries.values().filter(|e| e.kind == "function") {
        let p = e
            .params
            .iter()
            .map(|p| serde_json::from_value::<Ty>(p["ty"].clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let params = p
            .iter()
            .flat_map(|t| abi::wasm_arguments(s, t))
            .filter_map(|t| {
                if reference_abi && references::lane(s, &t) {
                    Some(ValType::EXTERNREF)
                } else {
                    raw(&t)
                }
            })
            .collect();
        let result = e.result.as_ref().unwrap();
        let results = if reference_abi && references::host(s, result) && !e.asynchronous {
            vec![ValType::EXTERNREF]
        } else if result.scalar() && !e.asynchronous {
            raw(result).into_iter().collect()
        } else {
            vec![I32]
        };
        check(&format!("__ocamljs_fn_{}", e.name), params, results)?;
    }
    Ok(())
}
/// Validate concrete type instantiations, including ownership hidden by aliases.
pub(super) fn instantiated(
    s: &Schema,
    t: &Ty,
    input: bool,
    seen: &mut BTreeSet<String>,
) -> Result<(), Error> {
    if seen.len() > 128 {
        return Err("ABI type expansion limit exceeded".into());
    }
    if t.kind == "ref" {
        return Ok(());
    }
    if matches!(t.name.as_str(), "Rc" | "Arc")
        && t.args
            .first()
            .is_some_and(|a| contains_opaque(s, a, &mut BTreeSet::new()))
    {
        return Err(
            "Rc<opaque> cannot preserve identity, including through generic aliases or records"
                .into(),
        );
    }
    for a in &t.args {
        instantiated(s, a, input, seen)?;
    }
    if let Some(i) = &t.inner {
        instantiated(s, i, input, seen)?;
    }
    let Some(e) = s.entries.get(&t.name) else {
        return Ok(());
    };
    if input && e.kind == "opaque" {
        return Err("opaque inputs must be call-scoped references".into());
    }
    let key = serde_json::to_string(t)?;
    if !seen.insert(key) {
        return Ok(());
    }
    let bindings = e
        .params
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .zip(t.args.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    if let Some(t) = &e.ty {
        instantiated(s, &substitute(t, &bindings), input, seen)?;
    }
    if e.kind != "opaque" {
        if let Some(b) = &e.body {
            for f in &b.fields {
                instantiated(s, &substitute(&f.ty, &bindings), input, seen)?;
            }
        }
        for v in &e.variants {
            for f in &v.body.fields {
                instantiated(s, &substitute(&f.ty, &bindings), input, seen)?;
            }
        }
    }
    Ok(())
}
