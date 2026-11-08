//! Input layout shared by signature validation and both code generators.
use super::*;
pub(super) fn opaque<'a>(s: &'a Schema, t: &Ty) -> Option<&'a Entry> {
    let mut current = t.clone();
    for _ in 0..128 {
        let entry = s.entries.get(&current.name)?;
        if entry.kind == "alias" {
            let bindings = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(current.args.iter().cloned())
                .collect();
            current = collections::substitute(entry.ty.as_ref()?, &bindings);
        } else {
            return (entry.kind == "opaque").then_some(entry);
        }
    }
    None
}
pub(super) fn flat<'a>(s: &'a Schema, t: &Ty) -> Option<&'a Entry> {
    let mut current = t;
    for _ in 0..128 {
        let e = s.entries.get(&current.name)?;
        if e.kind == "alias" && e.params.is_empty() {
            current = e.ty.as_ref()?;
            continue;
        }
        return e.flat.then_some(e);
    }
    None
}
pub(super) fn arguments(s: &Schema, t: &Ty) -> Vec<Ty> {
    if let Some(lanes) = tuple(t) {
        return lanes;
    }
    flat(s, t)
        .map(|e| {
            e.body
                .as_ref()
                .unwrap()
                .fields
                .iter()
                .map(|f| f.ty.clone())
                .collect()
        })
        .unwrap_or_else(|| vec![t.clone()])
}

pub(super) fn tuple(t: &Ty) -> Option<Vec<Ty>> {
    fn collect(t: &Ty, lanes: &mut Vec<Ty>) -> bool {
        if t.scalar() && !t.unit() {
            lanes.push(t.clone());
            return lanes.len() <= 8;
        }
        t.kind == "tuple" && !t.args.is_empty() && t.args.iter().all(|a| collect(a, lanes))
    }
    let mut lanes = Vec::new();
    (t.kind == "tuple" && collect(t, &mut lanes)).then_some(lanes)
}

pub(super) fn direct_result<'a>(s: &'a Schema, t: &Ty) -> Option<&'a Entry> {
    flat(s, t).filter(|e| {
        e.body
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .all(|f| super::bulk::numeric(&f.ty))
    })
}

pub(super) fn resolve(s: &Schema, t: &Ty) -> Ty {
    let mut t = t.clone();
    for _ in 0..128 {
        let Some(e) = s.entries.get(&t.name).filter(|e| e.kind == "alias") else {
            break;
        };
        let bindings = e
            .params
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .zip(t.args.iter().cloned())
            .collect();
        t = collections::substitute(e.ty.as_ref().unwrap(), &bindings);
    }
    t
}

/// Actual Rust-owned handles reachable through a result, including aliases,
/// instantiated generics, nested records and both Result branches.
pub(super) fn owns_result(s: &Schema, t: &Ty) -> bool {
    fn visit(s: &Schema, t: &Ty, seen: &mut std::collections::BTreeSet<String>) -> bool {
        if seen.len() >= 128 {
            return true;
        }
        if let Some(entry) = s.entries.get(&t.name) {
            if entry.kind == "opaque" {
                return true;
            }
            let key = serde_json::to_string(t).unwrap();
            if !seen.insert(key.clone()) {
                return false;
            }
            let bindings = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(t.args.iter().cloned())
                .collect();
            let owned = collections::fields(entry)
                .any(|field| visit(s, &collections::substitute(field, &bindings), seen));
            seen.remove(&key);
            owned
        } else {
            t.args.iter().any(|arg| visit(s, arg, seen))
                || t.inner.as_ref().is_some_and(|inner| visit(s, inner, seen))
        }
    }
    visit(s, t, &mut Default::default())
}
pub(super) fn vector(s: &Schema, t: &Ty) -> bool {
    let t = resolve(s, t);
    t.name == "Vec" && t.args.len() == 1
}
pub(super) struct NumericInput {
    pub element: Ty,
    pub kind: usize,
    pub borrowed: bool,
    pub bytes: bool,
    pub js: bool,
}
pub(super) fn numeric_input(s: &Schema, t: &Ty) -> Option<NumericInput> {
    let aliases = |mut current: Ty, js: &mut bool| {
        for _ in 0..128 {
            let Some(e) = s.entries.get(&current.name).filter(|e| e.kind == "alias") else {
                break;
            };
            *js |= e.js_value;
            let bindings = e
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(current.args.iter().cloned())
                .collect();
            current = collections::substitute(e.ty.as_ref()?, &bindings);
        }
        Some(current)
    };
    let mut js = t.map.starts_with("js:");
    let mut current = aliases(t.clone(), &mut js)?;
    let borrowed = current.kind == "ref";
    if borrowed {
        current = aliases(*current.inner.clone()?, &mut js)?;
    }
    let element = if current.name == "Vec" && !borrowed {
        current.args.first()?
    } else if current.kind == "slice" && borrowed {
        current.inner.as_ref()?
    } else {
        return None;
    };
    let element = resolve(s, element);
    let kind = [
        "u8", "i8", "u16", "i16", "u32", "i32", "f32", "f64", "usize", "isize",
    ]
    .iter()
    .position(|n| *n == element.name)?;
    Some(NumericInput {
        bytes: borrowed && kind == 0,
        element,
        kind,
        borrowed,
        js,
    })
}
pub(super) fn wasm_arguments(s: &Schema, t: &Ty) -> Vec<Ty> {
    if text_input(s, t).is_some() {
        return vec![
            Ty {
                kind: "named".into(),
                name: "u32".into(),
                qualified: false,
                args: vec![],
                inner: None,
                len: None,
                mutable: false,
                map: String::new()
            };
            3
        ];
    }
    if numeric_input(s, t).is_some() || vector(s, t) {
        let lane = Ty {
            kind: "named".into(),
            name: "u32".into(),
            qualified: false,
            args: vec![],
            inner: None,
            len: None,
            mutable: false,
            map: String::new(),
        };
        vec![lane.clone(), lane]
    } else {
        arguments(s, t)
    }
}

pub(super) struct TextInput {
    pub js: bool,
    pub borrowed: bool,
}
pub(super) fn text_input(s: &Schema, ty: &Ty) -> Option<TextInput> {
    let mut t = ty.clone();
    let mut js = t.map.starts_with("js:");
    let mut borrowed = false;
    for _ in 0..128 {
        if t.kind == "ref" {
            borrowed = true;
            t = *t.inner?;
            continue;
        }
        if let Some(e) = s.entries.get(&t.name).filter(|e| e.kind == "alias") {
            js |= e.js_value;
            let bindings = e
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(t.args.iter().cloned())
                .collect();
            t = collections::substitute(e.ty.as_ref()?, &bindings);
            continue;
        }
        if s.entries.contains_key(&t.name) {
            return None;
        }
        return ((t.name == "String" && !borrowed) || (t.name == "str" && borrowed))
            .then_some(TextInput { js, borrowed });
    }
    None
}
pub(super) fn text_output(s: &Schema, t: &Ty) -> bool {
    let value = resolve(s, t);
    value.name == "String" && !s.entries.contains_key(&value.name)
}

pub(super) fn text_result(s: &Schema, ty: &Ty) -> Option<Ty> {
    let ty = resolve(s, ty);
    (ty.name == "Result" && ty.args.len() == 2 && text_output(s, &ty.args[0]))
        .then(|| ty.args[0].clone())
}
// Each alias owns its representation. The alias directly defining Result
// determines whether the outer value is a JS tagged object or a native result.
pub(super) fn text_result_js(s: &Schema, ty: &Ty) -> bool {
    let mut current = ty.clone();
    let mut js = false;
    for _ in 0..128 {
        let Some(entry) = s.entries.get(&current.name).filter(|e| e.kind == "alias") else {
            return current.name == "Result" && js;
        };
        js = entry.js_value;
        let bindings = entry
            .params
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .zip(current.args.iter().cloned())
            .collect();
        current = collections::substitute(entry.ty.as_ref().unwrap(), &bindings);
    }
    false
}

/// Result-specific task completion shares the synchronous type-directed ABI.
pub(super) fn async_direct(s: &Schema, e: &Entry) -> bool {
    let Some(t) = &e.result else {
        return false;
    };
    let t = resolve(s, t);
    e.asynchronous
        && (t.scalar()
            || references::host(s, &t)
            || opaque(s, &t).is_some()
            || direct_result(s, &t).is_some()
            || text_output(s, &t)
            || !e.buffered && text_result(s, &t).is_some())
}
