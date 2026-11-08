//! First-principles post-link reference ABI. Rust uses numeric internal slots;
//! public Wasm calls/imports use externref and all slot operations stay in Wasm.
use super::*;
use std::collections::BTreeMap;
use wasm_encoder::{
    self as w,
    reencode::{Error as ReError, Reencode},
};

pub(super) const MARKER: &str = "ocamljs.references";
pub(super) fn lowered(wasm: &[u8]) -> bool {
    wasmparser::Parser::new(0)
        .parse_all(wasm)
        .any(|p| matches!(p,Ok(wasmparser::Payload::CustomSection(c)) if c.name()==MARKER))
}
pub(super) fn resolved<'a>(s: &'a Schema, t: &'a Ty) -> &'a Ty {
    let mut t = t;
    for _ in 0..128 {
        if t.kind == "ref" {
            if let Some(inner) = &t.inner {
                t = inner;
                continue;
            }
        }
        if let Some(e) = s.entries.get(&t.name) {
            if e.kind == "alias" && e.params.is_empty() {
                if let Some(inner) = &e.ty {
                    t = inner;
                    continue;
                }
            }
        }
        break;
    }
    t
}
pub(super) fn host(s: &Schema, t: &Ty) -> bool {
    let t = resolved(s, t);
    t.kind == "named"
        && (matches!(
            t.name.as_str(),
            "HostError" | "Promise" | "JsString" | "Uint8ClampedArray"
        ) || s.entries.get(&t.name).is_some_and(|e| e.kind == "host"))
}
pub(super) fn lane(s: &Schema, t: &Ty) -> bool {
    host(s, t) || t.name == "Callback"
}
pub(super) fn used(s: &Schema) -> bool {
    s.entries.values().any(|e| {
        e.kind == "host"
            || e.kind == "import"
            || !e.reference_support.is_empty()
            || e.params.iter().any(|p| p["ty"]["name"] == "Callback")
    })
}
pub(super) fn promises(s: &Schema) -> bool {
    fn check(t: &Ty) -> bool {
        t.name == "Promise"
            || t.args.iter().any(check)
            || t.inner.as_ref().is_some_and(|t| check(t))
    }
    s.entries.values().any(|e| {
        e.result.as_ref().is_some_and(check)
            || collections::fields(e).any(check)
            || e.params
                .iter()
                .any(|p| serde_json::from_value::<Ty>(p["ty"].clone()).is_ok_and(|t| check(&t)))
    })
}
pub(super) fn contains_host(
    s: &Schema,
    t: &Ty,
    seen: &mut std::collections::BTreeSet<String>,
) -> bool {
    if host(s, t) {
        return true;
    }
    if t.args.iter().any(|t| contains_host(s, t, seen))
        || t.inner.as_ref().is_some_and(|t| contains_host(s, t, seen))
    {
        return true;
    }
    if let Some(e) = s.entries.get(&t.name) {
        if seen.insert(t.name.clone()) {
            return collections::fields(e).any(|t| contains_host(s, t, seen));
        }
    }
    false
}
pub(super) fn javascript_modules(s: &Schema) -> String {
    s.entries
        .values()
        .filter(|e| e.kind == "import" && !e.js_module.is_empty())
        .map(|e| e.js_module.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(i, m)| format!("import * as __host_module_{i} from {m:?};\n"))
        .collect()
}
pub(super) fn raw(t: &Ty) -> Option<w::ValType> {
    if t.unit() {
        None
    } else {
        Some(match t.name.as_str() {
            "i64" | "u64" => w::ValType::I64,
            "f32" => w::ValType::F32,
            "f64" => w::ValType::F64,
            _ => w::ValType::I32,
        })
    }
}
/// Shared intrinsic signatures for validation and post-link lowering.
pub(super) fn intrinsic(name: &str, external: bool) -> Option<(Vec<w::ValType>, Vec<w::ValType>)> {
    use w::ValType::I32;
    let reference = if external { w::ValType::EXTERNREF } else { I32 };
    Some(match name {
        "host_promise_subscribe" => (vec![reference, I32], vec![]),
        "host_promise_unsubscribe" => (vec![I32], vec![]),
        "callback_call" => (vec![reference, I32], vec![I32]),
        "host_string_length" => (vec![reference, I32], vec![I32]),
        "host_string_copy" => (vec![reference, I32, I32, I32], vec![I32]),
        "host_bytes_copy" => (vec![reference, I32, I32, I32], vec![]),
        "host_bytes_prepare" => (vec![reference, I32, I32, I32], vec![reference]),
        "host_bytes_new" => (vec![I32, I32, I32], vec![reference]),
        _ => return None,
    })
}
pub(super) fn import_signature(
    s: &Schema,
    e: &Entry,
    external: bool,
) -> Result<(Vec<w::ValType>, Vec<w::ValType>), Error> {
    let mut params = Vec::new();
    for p in &e.params {
        let t: Ty = serde_json::from_value(p["ty"].clone())?;
        if t.kind == "ref" && t.inner.as_ref().is_some_and(|t| t.name == "str") {
            params.extend([w::ValType::I32, w::ValType::I32]);
        } else if external && host(s, &t) {
            params.push(w::ValType::EXTERNREF);
        } else {
            params.push(raw(&t).unwrap());
        }
    }
    params.push(w::ValType::I32); // caught-exception destination, written only on failure
    let t = e.result.as_ref().unwrap();
    let result = if external && host(s, t) {
        Some(w::ValType::EXTERNREF)
    } else {
        raw(t)
    };
    Ok((params, result.into_iter().collect()))
}
pub(super) fn validate_import(s: &Schema, e: &Entry) -> Result<(), Error> {
    if !matches!(
        e.operation.as_str(),
        "function" | "method" | "getter" | "setter" | "call" | "constructor" | "indexing_getter"
    ) || e.js_name.is_empty()
    {
        return Err("invalid host operation".into());
    }
    if !e.js_namespace.is_empty()
        && e.js_namespace.split('.').any(|n| {
            n.is_empty()
                || !n
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        })
    {
        return Err("invalid host namespace".into());
    }
    let receiver = matches!(
        e.operation.as_str(),
        "method" | "getter" | "setter" | "call" | "indexing_getter"
    );
    if receiver
        && e.params.first().is_none_or(|p| {
            serde_json::from_value::<Ty>(p["ty"].clone())
                .map_or(true, |t| t.kind != "ref" || !host(s, &t))
        })
    {
        return Err("host operation requires a host receiver".into());
    }
    if e.operation == "getter" && e.params.len() != 1
        || matches!(e.operation.as_str(), "setter" | "indexing_getter") && e.params.len() != 2
    {
        return Err("invalid host property arity".into());
    }
    for p in &e.params {
        let t: Ty = serde_json::from_value(p["ty"].clone())?;
        s.check_ty(&t, &[], true)?;
        if t.kind == "ref" {
            if t.mutable
                || t.inner
                    .as_ref()
                    .is_none_or(|t| t.name != "str" && !host(s, t))
            {
                return Err("host import reference must be &str or &HostType".into());
            }
        } else if !t.scalar() || t.unit() {
            return Err("host import parameters must be scalars or host references".into());
        }
    }
    let t = e.result.as_ref().ok_or("host import missing result")?;
    s.check_ty(t, &[], false)?;
    validation::instantiated(s, t, false, &mut std::collections::BTreeSet::new())?;
    if !t.scalar() && !host(s, t) {
        return Err(
            "host imports return scalars or host values inside Result<T, HostError>".into(),
        );
    }
    Ok(())
}
pub(super) fn javascript(s: &Schema) -> Result<String, Error> {
    use std::fmt::Write;
    let mut out = String::new();
    let modules = s
        .entries
        .values()
        .filter(|e| e.kind == "import" && !e.js_module.is_empty())
        .map(|e| e.js_module.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(i, m)| (m, i))
        .collect::<BTreeMap<_, _>>();
    for e in s.entries.values().filter(|e| e.kind == "import") {
        let mut parameters = Vec::new();
        let mut values = Vec::new();
        for p in &e.params {
            let t: Ty = serde_json::from_value(p["ty"].clone())?;
            let index = parameters.len();
            let name = format!("a{index}");
            parameters.push(name.clone());
            if t.kind == "ref" && t.inner.as_ref().is_some_and(|t| t.name == "str") {
                let len = format!("a{}", index + 1);
                parameters.push(len.clone());
                values.push(format!(
                    "decoder.decode(new Uint8Array(e.memory.buffer,{name}>>>0,{len}))"
                ));
            } else if t.name == "bool" {
                values.push(format!("{name}!==0"));
            } else if matches!(t.name.as_str(), "u32" | "usize") {
                values.push(format!("{name}>>>0"));
            } else if t.name == "u64" {
                values.push(format!("BigInt.asUintN(64,{name})"));
            } else {
                values.push(name);
            }
        }
        let error = format!("a{}", parameters.len());
        parameters.push(error.clone());
        let base = if e.js_module.is_empty() {
            "globalThis".into()
        } else {
            format!("__host_module_{}", modules[e.js_module.as_str()])
        };
        let path = e
            .js_namespace
            .split('.')
            .filter(|s| !s.is_empty())
            .chain(std::iter::once(e.js_name.as_str()))
            .fold(base, |p, n| format!("{p}[{n:?}]"));
        let expr = match e.operation.as_str() {
            "method" => format!("{}[{:?}]({})", values[0], e.js_name, values[1..].join(",")),
            "getter" => format!("{}[{:?}]", values[0], e.js_name),
            "setter" => format!("({}[{:?}]={})", values[0], e.js_name, values[1]),
            "indexing_getter" => format!("{}[{}]", values[0], values[1]),
            "call" => format!("{}({})", values[0], values[1..].join(",")),
            "constructor" => format!("new {path}({})", values.join(",")),
            _ => format!("{path}({})", values.join(",")),
        };
        let ret = e.result.as_ref().unwrap();
        let success = if resolved(s, ret).name == "Promise" {
            format!("return observeHostPromise({expr});")
        } else if ret.unit() {
            format!("{expr};return;")
        } else if matches!(ret.name.as_str(), "i64" | "u64") {
            let (min, max) = if ret.name == "u64" {
                ("0n", "18446744073709551615n")
            } else {
                ("-9223372036854775808n", "9223372036854775807n")
            };
            format!(
                "const value={expr};if(typeof value!=='bigint'||value<{min}||value>{max})throw new RangeError('host integer out of range');return value;"
            )
        } else if ret.scalar() {
            format!("return {};", calls::scalar(ret, &expr))
        } else {
            format!("return {expr};")
        };
        let failure = if ret.unit() {
            ""
        } else if host(s, ret) {
            "null"
        } else if matches!(ret.name.as_str(), "i64" | "u64") {
            "0n"
        } else {
            "0"
        };
        writeln!(
            out,
            "{:?}:function({}){{try{{{success}}}catch(error){{e.__ocamljs_host_error({error},error);return {failure};}}}},",
            e.name,
            parameters.join(",")
        )?;
    }
    Ok(out)
}

#[derive(Clone)]
struct Added {
    ty: u32,
    body: w::Function,
}
struct Rewrite {
    imports: Vec<(String, String, u32)>,
    removed: Vec<u32>,
    adapters: BTreeMap<u32, u32>,
    export_wrappers: BTreeMap<String, u32>,
    native_types: BTreeMap<u32, u32>,
    extra_types: Vec<(Vec<w::ValType>, Vec<w::ValType>)>,
    added: Vec<Added>,
    base: u32,
    type_count: u32,
    table: u32,
    table_emitted: bool,
    global_emitted: bool,
    clear: u32,
    capture: u32,
    live: u32,
}
impl Rewrite {
    fn original(&self, idx: u32) -> u32 {
        idx - self.removed.iter().filter(|x| **x < idx).count() as u32
    }
    fn typ(&mut self, p: Vec<w::ValType>, r: Vec<w::ValType>) -> u32 {
        let id = self.type_count + self.extra_types.len() as u32;
        self.extra_types.push((p, r));
        id
    }
    fn add(&mut self, p: Vec<w::ValType>, r: Vec<w::ValType>, body: w::Function) -> u32 {
        let ty = self.typ(p, r);
        let id = self.base + self.added.len() as u32;
        self.added.push(Added { ty, body });
        id
    }
    fn table_section(&self) -> w::TableSection {
        let mut t = w::TableSection::new();
        t.table(w::TableType {
            element_type: w::RefType::EXTERNREF,
            minimum: ocamljs_bindgen::host::BORROW_SLOTS as u64,
            maximum: Some(1_000_192),
            table64: false,
            shared: false,
        });
        t
    }
    fn global_section(&self) -> w::GlobalSection {
        let mut g = w::GlobalSection::new();
        g.global(
            w::GlobalType {
                val_type: w::ValType::I32,
                mutable: true,
                shared: false,
            },
            &w::ConstExpr::i32_const(0),
        );
        g
    }
}
impl Reencode for Rewrite {
    type Error = String;
    fn function_index(&mut self, idx: u32) -> Result<u32, ReError<String>> {
        Ok(self
            .adapters
            .get(&idx)
            .copied()
            .unwrap_or_else(|| self.original(idx)))
    }
    fn parse_type_section(
        &mut self,
        t: &mut w::TypeSection,
        r: wasmparser::TypeSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        w::reencode::utils::parse_type_section(self, t, r)?;
        for (p, r) in &self.extra_types {
            t.ty().function(p.iter().copied(), r.iter().copied());
        }
        Ok(())
    }
    fn parse_import_section(
        &mut self,
        t: &mut w::ImportSection,
        r: wasmparser::ImportSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        let mut idx = 0;
        for i in r {
            let i = i?;
            if let wasmparser::TypeRef::Func(ty) = i.ty {
                if !self.removed.contains(&idx) {
                    t.import(
                        i.module,
                        i.name,
                        w::EntityType::Function(self.native_types.get(&idx).copied().unwrap_or(ty)),
                    );
                }
                idx += 1;
            } else {
                w::reencode::utils::parse_import(self, t, i)?;
            }
        }
        Ok(())
    }
    fn parse_function_section(
        &mut self,
        t: &mut w::FunctionSection,
        r: wasmparser::FunctionSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        w::reencode::utils::parse_function_section(self, t, r)?;
        for f in &self.added {
            t.function(f.ty);
        }
        Ok(())
    }
    fn parse_code_section(
        &mut self,
        t: &mut w::CodeSection,
        r: wasmparser::CodeSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        w::reencode::utils::parse_code_section(self, t, r)?;
        for f in &self.added {
            t.function(&f.body);
        }
        Ok(())
    }
    fn parse_export_section(
        &mut self,
        t: &mut w::ExportSection,
        r: wasmparser::ExportSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        for e in r {
            let e = e?;
            if e.name.starts_with("__ocamljs_ref_") {
                continue;
            }
            if let Some(idx) = self.export_wrappers.get(e.name) {
                t.export(e.name, w::ExportKind::Func, *idx);
            } else {
                w::reencode::utils::parse_export(self, t, e)?;
            }
        }
        for (name, idx) in &self.export_wrappers {
            if name.starts_with("__ocamljs_host_") {
                t.export(name, w::ExportKind::Func, *idx);
            }
        }
        t.export("__ocamljs_host_live", w::ExportKind::Func, self.live);
        t.export(
            "__ocamljs_host_references",
            w::ExportKind::Table,
            self.table,
        );
        t.export("__ocamljs_host_clear", w::ExportKind::Func, self.clear);
        t.export("__ocamljs_host_error", w::ExportKind::Func, self.capture);
        Ok(())
    }
    fn parse_table_section(
        &mut self,
        t: &mut w::TableSection,
        r: wasmparser::TableSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        w::reencode::utils::parse_table_section(self, t, r)?;
        t.table(w::TableType {
            element_type: w::RefType::EXTERNREF,
            minimum: 128,
            maximum: Some(1_000_192),
            table64: false,
            shared: false,
        });
        self.table_emitted = true;
        Ok(())
    }
    fn parse_global_section(
        &mut self,
        t: &mut w::GlobalSection,
        r: wasmparser::GlobalSectionReader<'_>,
    ) -> Result<(), ReError<String>> {
        w::reencode::utils::parse_global_section(self, t, r)?;
        t.global(
            w::GlobalType {
                val_type: w::ValType::I32,
                mutable: true,
                shared: false,
            },
            &w::ConstExpr::i32_const(0),
        );
        self.global_emitted = true;
        Ok(())
    }
    fn intersperse_section_hook(
        &mut self,
        m: &mut w::Module,
        _after: Option<w::SectionId>,
        before: Option<w::SectionId>,
    ) -> Result<(), ReError<String>> {
        let before = before.map(|s| s as u8).unwrap_or(255);
        if !self.table_emitted && before > w::SectionId::Table as u8 {
            m.section(&self.table_section());
            self.table_emitted = true;
        }
        if !self.global_emitted && before > w::SectionId::Global as u8 {
            m.section(&self.global_section());
            self.global_emitted = true;
        }
        Ok(())
    }
}

pub(super) fn lower(s: &Schema, wasm: &[u8]) -> Result<Vec<u8>, Error> {
    if !used(s) || lowered(wasm) {
        return Ok(wasm.to_vec());
    }
    use w::{Instruction as I, ValType as V};
    let mut imports = Vec::new();
    let mut exports = BTreeMap::new();
    let mut types = Vec::new();
    let (mut functions, mut table, mut global) = (0, 0, 0);
    for p in wasmparser::Parser::new(0).parse_all(wasm) {
        match p? {
            wasmparser::Payload::TypeSection(r) => {
                for t in r.into_iter_err_on_gc_types() {
                    types.push(t?);
                }
            }
            wasmparser::Payload::ImportSection(r) => {
                for i in r {
                    let i = i?;
                    if let wasmparser::TypeRef::Func(t) = i.ty {
                        imports.push((i.module.to_owned(), i.name.to_owned(), t));
                    }
                }
            }
            wasmparser::Payload::FunctionSection(r) => functions = r.count(),
            wasmparser::Payload::TableSection(r) => table = r.count(),
            wasmparser::Payload::GlobalSection(r) => global = r.count(),
            wasmparser::Payload::ExportSection(r) => {
                for e in r {
                    let e = e?;
                    if e.kind == wasmparser::ExternalKind::Func {
                        exports.insert(e.name.to_owned(), e.index);
                    }
                }
            }
            _ => {}
        }
    }
    let removed = imports
        .iter()
        .enumerate()
        .filter(|(_, i)| i.0 == "ocamljs_ref")
        .map(|(i, _)| i as u32)
        .collect::<Vec<_>>();
    let support = s
        .entries
        .values()
        .find(|e| !e.reference_support.is_empty())
        .ok_or("missing reference support descriptor")?
        .reference_support
        .clone();
    let mut g = Rewrite {
        base: imports.len() as u32 - removed.len() as u32 + functions,
        type_count: types.len() as u32,
        imports,
        removed,
        adapters: BTreeMap::new(),
        export_wrappers: BTreeMap::new(),
        native_types: BTreeMap::new(),
        extra_types: Vec::new(),
        added: Vec::new(),
        table,
        table_emitted: false,
        global_emitted: false,
        clear: 0,
        capture: 0,
        live: 0,
    };
    let alloc = g.original(
        *exports
            .get(&format!("{support}_alloc"))
            .ok_or("missing host allocator")?,
    );
    let release = g.original(
        *exports
            .get(&format!("{support}_release"))
            .ok_or("missing host release")?,
    );
    g.live = g.original(
        *exports
            .get(&format!("{support}_live"))
            .ok_or("missing host live counter")?,
    );

    // Store one host reference in an owning Rust-managed root slot.
    let mut f = w::Function::new([(1, V::I32)]);
    for i in [
        I::Call(alloc),
        I::LocalSet(1),
        I::LocalGet(1),
        I::TableSize(table),
        I::I32GeU,
        I::If(w::BlockType::Empty),
        I::RefNull(w::HeapType::EXTERN),
        I::LocalGet(1),
        I::TableSize(table),
        I::I32Sub,
        I::I32Const(128),
        I::I32Add,
        I::I32Const(-128),
        I::I32And,
        I::TableGrow(table),
        I::I32Const(-1),
        I::I32Eq,
        I::If(w::BlockType::Empty),
        I::Unreachable,
        I::End,
        I::End,
        I::LocalGet(1),
        I::LocalGet(0),
        I::TableSet(table),
        I::LocalGet(1),
        I::End,
    ] {
        f.instruction(&i);
    }
    let store = g.add(vec![V::EXTERNREF], vec![V::I32], f);
    let mut f = w::Function::new([]);
    for i in [
        I::LocalGet(0),
        I::RefNull(w::HeapType::EXTERN),
        I::TableSet(table),
        I::LocalGet(0),
        I::Call(release),
        I::End,
    ] {
        f.instruction(&i);
    }
    let drop = g.add(vec![V::I32], vec![], f);
    let mut f = w::Function::new([]);
    for i in [I::LocalGet(0), I::TableGet(table), I::Call(store), I::End] {
        f.instruction(&i);
    }
    let clone = g.add(vec![V::I32], vec![V::I32], f);
    let mut f = w::Function::new([(1, V::EXTERNREF)]);
    for i in [
        I::LocalGet(0),
        I::TableGet(table),
        I::LocalSet(1),
        I::LocalGet(0),
        I::Call(drop),
        I::LocalGet(1),
        I::End,
    ] {
        f.instruction(&i);
    }
    let take = g.add(vec![V::I32], vec![V::EXTERNREF], f);
    if s.entries
        .values()
        .any(|entry| entry.asynchronous && entry.result.as_ref().is_some_and(|ty| host(s, ty)))
    {
        g.export_wrappers.insert("__ocamljs_host_take".into(), take);
        g.export_wrappers.insert("__ocamljs_host_drop".into(), drop);
    }
    let mut f = w::Function::new([]);
    for i in [
        I::Block(w::BlockType::Empty),
        I::Loop(w::BlockType::Empty),
        I::GlobalGet(global),
        I::I32Eqz,
        I::BrIf(1),
        I::GlobalGet(global),
        I::I32Const(1),
        I::I32Sub,
        I::GlobalSet(global),
        I::GlobalGet(global),
        I::RefNull(w::HeapType::EXTERN),
        I::TableSet(table),
        I::Br(0),
        I::End,
        I::End,
        I::End,
    ] {
        f.instruction(&i);
    }
    g.clear = g.add(vec![], vec![], f);
    let mut f = w::Function::new([]);
    for i in [
        I::LocalGet(0),
        I::LocalGet(1),
        I::Call(store),
        I::I32Store(w::MemArg {
            offset: 0,
            align: 2,
            memory_index: 0,
        }),
        I::End,
    ] {
        f.instruction(&i);
    }
    g.capture = g.add(vec![V::I32, V::EXTERNREF], vec![], f);
    for (idx, (_, name, _)) in g
        .imports
        .clone()
        .into_iter()
        .enumerate()
        .filter(|(_, i)| i.0 == "ocamljs_ref")
    {
        let target = match name.as_str() {
            "clone" => clone,
            "drop" => drop,
            _ => return Err(format!("unknown reference intrinsic {name}").into()),
        };
        g.adapters.insert(idx as u32, target);
    }
    // Native JS imports receive references directly. Internal call sites adapt
    // Rust slots with table.get and retain any returned host reference in Wasm.
    for (idx, (module, name, ty)) in g.imports.clone().into_iter().enumerate() {
        if module == "ocamljs_ref" {
            continue;
        }
        let (params, results, host_params, host_result) = if module == "ocamljs_host" {
            let e = &s.entries[&name];
            let (p, r) = import_signature(s, e, true)?;
            let h = p.iter().map(|t| *t == V::EXTERNREF).collect::<Vec<_>>();
            let hr = r == [V::EXTERNREF];
            (p, r, h, hr)
        } else if module == "ocamljs" {
            let Some((p, r)) = intrinsic(&name, true) else {
                continue;
            };
            let h = p.iter().map(|t| *t == V::EXTERNREF).collect::<Vec<_>>();
            let hr = r == [V::EXTERNREF];
            (p, r, h, hr)
        } else {
            continue;
        };
        let typeid = g.typ(params.clone(), results.clone());
        g.native_types.insert(idx as u32, typeid);
        let mut f = w::Function::new(if host_result {
            vec![(1, V::EXTERNREF)]
        } else {
            vec![]
        });
        for (n, host) in host_params.iter().enumerate() {
            f.instruction(&I::LocalGet(n as u32));
            if *host {
                f.instruction(&I::TableGet(table));
            }
        }
        f.instruction(&I::Call(g.original(idx as u32)));
        if host_result {
            let n = params.len() as u32;
            f.instruction(&I::LocalSet(n));
            f.instruction(&I::LocalGet(n - 1));
            f.instruction(&I::I32Load(w::MemArg {
                offset: 0,
                align: 2,
                memory_index: 0,
            }));
            f.instruction(&I::I32Eqz);
            f.instruction(&I::If(w::BlockType::Result(V::I32)));
            f.instruction(&I::LocalGet(n));
            f.instruction(&I::Call(store));
            f.instruction(&I::Else);
            f.instruction(&I::I32Const(0));
            f.instruction(&I::End);
        }
        f.instruction(&I::End);
        let p = params
            .iter()
            .map(|t| if *t == V::EXTERNREF { V::I32 } else { *t })
            .collect();
        let r = results
            .iter()
            .map(|t| if *t == V::EXTERNREF { V::I32 } else { *t })
            .collect();
        let adapter = g.add(p, r, f);
        g.adapters.insert(idx as u32, adapter);
        let _ = ty;
    }
    for e in s.entries.values().filter(|e| e.kind == "function") {
        let mut params = Vec::new();
        let mut host_params = Vec::new();
        for p in &e.params {
            let t: Ty = serde_json::from_value(p["ty"].clone())?;
            for t in abi::wasm_arguments(s, &t) {
                let host = lane(s, &t);
                if let Some(raw) = raw(&t) {
                    params.push(if host { V::EXTERNREF } else { raw });
                    host_params.push(host);
                }
            }
        }
        let ret = e.result.as_ref().unwrap();
        let host_result = !e.asynchronous && host(s, ret);
        if !host_params.contains(&true) && !host_result {
            continue;
        }
        let results = if host_result {
            vec![V::EXTERNREF]
        } else if e.asynchronous || !ret.scalar() {
            vec![V::I32]
        } else {
            raw(ret).into_iter().collect()
        };
        let name = format!("__ocamljs_fn_{}", e.name);
        let original = g.original(*exports.get(&name).ok_or("missing reference export")?);
        let body = export_body(
            params.len(),
            &host_params,
            &results,
            original,
            table,
            global,
            g.clear,
            if host_result { Some(take) } else { None },
        )?;
        let wrapper = g.add(params, results, body);
        g.export_wrappers.insert(name, wrapper);
    }
    if let Some(idx) = exports.get(&format!("{support}_output")) {
        let mut f = w::Function::new([]);
        for i in [
            I::LocalGet(0),
            I::LocalGet(1),
            I::Call(g.original(*idx)),
            I::TableGet(table),
            I::End,
        ] {
            f.instruction(&i);
        }
        let wrapper = g.add(vec![V::I32, V::I32], vec![V::EXTERNREF], f);
        g.export_wrappers
            .insert("__ocamljs_host_output".into(), wrapper);
    }
    if let Some(idx) = exports.get(&format!("{support}_settle")) {
        let params = vec![V::I32, V::EXTERNREF, V::I32];
        let body = export_body(
            3,
            &[false, true, false],
            &[],
            g.original(*idx),
            table,
            global,
            g.clear,
            None,
        )?;
        let wrapper = g.add(params, vec![], body);
        g.export_wrappers
            .insert("__ocamljs_host_settle".into(), wrapper);
    }
    let mut module = w::Module::new();
    g.parse_core_module(&mut module, wasmparser::Parser::new(0), wasm)
        .map_err(|e| format!("reference ABI lowering: {e}"))?;
    module.section(&w::CustomSection {
        name: MARKER.into(),
        data: std::borrow::Cow::Borrowed(&[1]),
    });
    let result = module.finish();
    wasmparser::Validator::new().validate_all(&result)?;
    Ok(result)
}

fn export_body(
    n: usize,
    hosts: &[bool],
    results: &[w::ValType],
    original: u32,
    table: u32,
    global: u32,
    clear: u32,
    take: Option<u32>,
) -> Result<w::Function, Error> {
    use w::Instruction as I;
    let count = hosts.iter().filter(|x| **x).count();
    if count > ocamljs_bindgen::host::BORROW_SLOTS as usize {
        return Err("too many direct host arguments".into());
    }
    let mut f = w::Function::new(results.iter().map(|t| (1, *t)));
    f.instruction(&I::I32Const(count as i32));
    f.instruction(&I::GlobalSet(global));
    let mut slot = 0;
    for (i, host) in hosts.iter().enumerate() {
        if *host {
            f.instruction(&I::I32Const(slot));
            f.instruction(&I::LocalGet(i as u32));
            f.instruction(&I::TableSet(table));
            slot += 1;
        }
    }
    let mut slot = 0;
    for (i, host) in hosts.iter().enumerate() {
        if *host {
            f.instruction(&I::I32Const(slot));
            slot += 1;
        } else {
            f.instruction(&I::LocalGet(i as u32));
        }
    }
    f.instruction(&I::Call(original));
    if let Some(take) = take {
        f.instruction(&I::Call(take));
    }
    if !results.is_empty() {
        f.instruction(&I::LocalSet(n as u32));
    }
    f.instruction(&I::Call(clear));
    if !results.is_empty() {
        f.instruction(&I::LocalGet(n as u32));
    }
    f.instruction(&I::End);
    Ok(f)
}
