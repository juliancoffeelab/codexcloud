//! Bind the runtime to the validated wire contract before removing build metadata.
use super::{Error, Schema, read};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(super) const PRUNED_MARKER: &str = "ocamljs.pruned";

/// Remove only SDK-private export roots which the emitted runtime cannot use,
/// then collect their unreachable bodies and dependencies. User exports, start
/// functions, active segment effects and indirect-call targets remain GC roots.
pub(super) fn prune_runtime(wasm: &[u8], runtime: &str, debug: bool) -> Result<Vec<u8>, Error> {
    let mut config = walrus::ModuleConfig::new();
    config
        .generate_dwarf(debug)
        .generate_name_section(debug)
        .generate_producers_section(debug);
    let mut module = config.parse(wasm)?;
    canonicalize_task_hooks(&mut module)?;
    let probes = debug
        || module
            .exports
            .iter()
            .any(|e| e.name == "__ocamljs_resource_diagnostics");
    let referenced: std::collections::BTreeSet<&str> = runtime
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .filter(|word| word.starts_with("__ocamljs_"))
        .collect();
    let present: std::collections::BTreeSet<&str> =
        module.exports.iter().map(|e| e.name.as_str()).collect();
    // A pruned release can be replayed with its sidecar, but cannot restore code
    // needed by a different runtime/backend. Fail during generation in that case.
    for name in &referenced {
        if private_runtime_export(name) && !present.contains(name) {
            return Err(format!("runtime requires missing Wasm export {name}; regenerate from the original Rust Wasm input").into());
        }
    }
    let remove: Vec<_> = module
        .exports
        .iter()
        .filter(|e| {
            !probes && private_runtime_export(&e.name) && !referenced.contains(e.name.as_str())
        })
        .map(|e| e.id())
        .collect();
    for id in remove {
        module.exports.delete(id);
    }
    // Preserve potentially trapping/observable table initialization even when
    // no live instruction refers to the table. A local, unexported table with
    // provably in-bounds constant segments and pure reference initializers can
    // disappear with its unreachable targets. Reachable tables remain natural
    // roots through call_indirect/table operations and keep all their elements.
    let mut tables: Vec<_> = module
        .elements
        .iter()
        .filter_map(|element| {
            if let walrus::ElementKind::Active { table, ref offset } = element.kind {
                let target = module.tables.get(table);
                let exported = module
                    .exports
                    .iter()
                    .any(|e| matches!(e.item, walrus::ExportItem::Table(t) if t == table));
                let offset = match offset {
                    walrus::ConstExpr::Value(walrus::ir::Value::I32(n)) if !target.table64 => {
                        u64::try_from(*n).ok()
                    }
                    walrus::ConstExpr::Value(walrus::ir::Value::I64(n)) if target.table64 => {
                        u64::try_from(*n).ok()
                    }
                    _ => None,
                };
                let (length, pure_items) = match &element.items {
                    walrus::ElementItems::Functions(items) => (items.len(), true),
                    walrus::ElementItems::Expressions(_, items) => {
                        (items.len(), items.iter().all(pure_reference))
                    }
                };
                let in_bounds = offset
                    .and_then(|n| n.checked_add(length as u64))
                    .is_some_and(|end| end <= target.initial);
                let pure_table = target.init.as_ref().is_none_or(pure_reference);
                if target.import.is_some() || exported || !in_bounds || !pure_items || !pure_table {
                    Some(table)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    tables.extend(
        module
            .tables
            .iter()
            .filter(|table| {
                table
                    .init
                    .as_ref()
                    .is_some_and(|init| !pure_reference(init))
            })
            .map(|table| table.id()),
    );
    let mut roots = Vec::new();
    for table in tables {
        let mut name = format!("__ocamljs_gc_table_root_{}", roots.len());
        while module.exports.iter().any(|e| e.name == name) {
            name.push('_');
        }
        roots.push(module.exports.add(&name, table));
    }
    // Walrus follows instructions, initializer expressions, tables and start;
    // deleting an export alone would leave all its implementation code behind.
    walrus::passes::gc::run(&mut module);
    for id in roots {
        module.exports.delete(id);
    }
    let mut output = module.emit_wasm();
    // Replace, rather than duplicate, this marker on sidecar replay.
    let mut stripped = output.get(..8).ok_or("truncated Wasm header")?.to_vec();
    for payload in wasmparser::Parser::new(0).parse_all(&output) {
        let payload = payload?;
        if matches!(&payload, wasmparser::Payload::CustomSection(c) if c.name() == PRUNED_MARKER) {
            continue;
        }
        if let Some((id, range)) = payload.as_section() {
            stripped.push(id);
            unsigned(&mut stripped, range.len());
            stripped.extend_from_slice(&output[range]);
        }
    }
    output = stripped;
    custom_section(&mut output, PRUNED_MARKER, &[1]);
    Ok(output)
}

fn pure_reference(expression: &walrus::ConstExpr) -> bool {
    matches!(
        expression,
        walrus::ConstExpr::RefNull(_) | walrus::ConstExpr::RefFunc(_)
    )
}

fn canonicalize_task_hooks(module: &mut walrus::Module) -> Result<(), Error> {
    for hook in ["poll", "cancel"] {
        let canonical = format!("__ocamljs_task_{hook}");
        let prefix = format!("{canonical}_for_");
        let mut aliases: Vec<_> = module
            .exports
            .iter()
            .filter(|export| export.name.starts_with(&prefix))
            .map(|export| (export.name.clone(), export.id(), export.item))
            .collect();
        aliases.sort_by(|a, b| a.0.cmp(&b.0));
        if !module.exports.iter().any(|export| export.name == canonical) {
            if let Some((_, _, item)) = aliases.first() {
                module.exports.add(&canonical, *item);
            }
        }
        for (_, id, _) in aliases {
            module.exports.delete(id);
        }
    }
    Ok(())
}

fn private_runtime_export(name: &str) -> bool {
    matches!(
        name,
        "__ocamljs_abi"
            | "__ocamljs_paranoid"
            | "__ocamljs_alloc"
            | "__ocamljs_contains"
            | "__ocamljs_claim"
            | "__ocamljs_dispose"
            | "__ocamljs_fetch_end"
            | "__ocamljs_fetch_push"
            | "__ocamljs_input_free"
            | "__ocamljs_input_ptr"
            | "__ocamljs_input_truncate"
            | "__ocamljs_live"
            | "__ocamljs_moved"
            | "__ocamljs_numeric_alloc"
            | "__ocamljs_numeric_free"
            | "__ocamljs_output_free"
            | "__ocamljs_output_info"
            | "__ocamljs_output_len"
            | "__ocamljs_output_pool_len"
            | "__ocamljs_output_pool_ptr"
            | "__ocamljs_output_ptr"
            | "__ocamljs_string_alloc"
            | "__ocamljs_string_ascii"
            | "__ocamljs_string_kind"
            | "__ocamljs_string_compact_latin1"
            | "__ocamljs_string_finish_latin1"
            | "__ocamljs_string_realloc"
            | "__ocamljs_string_free"
            | "__ocamljs_task_cancel"
            | "__ocamljs_task_poll"
            | "__ocamljs_task_ready"
            | "__ocamljs_tasks"
            | "__ocamljs_host_output"
            | "__ocamljs_host_settle"
            | "__ocamljs_host_live"
            | "__ocamljs_host_clear"
            | "__ocamljs_host_error"
            | "__ocamljs_host_take"
            | "__ocamljs_host_drop"
    )
}

/// Restore a release artifact's schema only when its stamp matches the sidecar.
/// The restored section is for offline generation; release output strips it again.
pub fn with_schema(wasm: &[u8], sidecar: &[u8]) -> Result<Vec<u8>, Error> {
    let entries: BTreeMap<String, serde_json::Value> = serde_json::from_slice(sidecar)?;
    let fingerprint = format!("{:x}", Sha256::digest(serde_json::to_vec(&entries)?));
    let mut data = Vec::new();
    for (name, entry) in entries {
        if entry["name"] != name {
            return Err("schema sidecar entry name mismatch".into());
        }
        serde_json::to_writer(&mut data, &entry)?;
        data.push(b'\n');
    }
    let mut restored = wasm.to_vec();
    let mut embedded = false;
    let mut stamps = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(wasm) {
        if let wasmparser::Payload::CustomSection(section) = payload? {
            embedded |= section.name() == "ocamljs.schema";
            if section.name() == "ocamljs.abi" {
                stamps.push(section.data());
            }
        }
    }
    if !embedded {
        custom_section(&mut restored, "ocamljs.schema", &data);
    }
    let schema = read(&restored)?;
    let expected = stamp(&schema);
    if schema.fingerprint != fingerprint {
        return Err("schema sidecar does not match the embedded Wasm schema".into());
    }
    if stamps.len() != 1 || stamps[0] != expected.as_bytes() {
        return Err("schema sidecar does not match the Wasm ABI stamp".into());
    }
    Ok(restored)
}

pub(super) fn sidecar(wasm: &[u8]) -> Result<Vec<u8>, Error> {
    let mut entries = BTreeMap::new();
    for payload in wasmparser::Parser::new(0).parse_all(wasm) {
        if let wasmparser::Payload::CustomSection(section) = payload? {
            if section.name() == "ocamljs.schema" {
                for line in std::str::from_utf8(section.data())?
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                {
                    let entry: serde_json::Value = serde_json::from_str(line)?;
                    let name = entry["name"]
                        .as_str()
                        .ok_or("schema entry is missing its name")?;
                    entries.insert(name.to_owned(), entry);
                }
            }
        }
    }
    Ok(serde_json::to_vec_pretty(&entries)?)
}

pub(super) fn stamp(schema: &Schema) -> String {
    format!("{}:{}", ocamljs_bindgen::ABI_VERSION, schema.fingerprint)
}

pub(super) fn package(schema: &Schema, wasm: &[u8], debug: bool) -> Result<Vec<u8>, Error> {
    // Public callers may pass a Schema independently of the artifact. Never stamp
    // supplied metadata until it agrees with the actual validated Wasm exports.
    let actual = read(wasm)?;
    if actual.fingerprint != schema.fingerprint
        || serde_json::to_vec(&actual.entries)? != serde_json::to_vec(&schema.entries)?
    {
        return Err("schema does not match the input Wasm artifact".into());
    }
    let lowered = super::references::lower(schema, wasm)?;
    let mut output = strip_metadata(&lowered, debug)?;
    custom_section(&mut output, "ocamljs.abi", stamp(schema).as_bytes());
    Ok(output)
}

fn custom_section(output: &mut Vec<u8>, name: &str, contents: &[u8]) {
    let mut section = Vec::new();
    unsigned(&mut section, name.len());
    section.extend_from_slice(name.as_bytes());
    section.extend_from_slice(contents);
    output.push(0);
    unsigned(output, section.len());
    output.extend(section);
}

fn unsigned(output: &mut Vec<u8>, mut value: usize) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        output.push(byte | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            break;
        }
    }
}

fn strip_metadata(wasm: &[u8], debug: bool) -> Result<Vec<u8>, Error> {
    let mut output = wasm.get(..8).ok_or("truncated Wasm header")?.to_vec();
    for payload in wasmparser::Parser::new(0).parse_all(wasm) {
        let payload = payload?;
        let Some((id, range)) = payload.as_section() else {
            continue;
        };
        if let wasmparser::Payload::CustomSection(section) = &payload {
            let name = section.name();
            if name == "ocamljs.abi"
                || !debug
                    && (matches!(
                        name,
                        "ocamljs.schema"
                            | "name"
                            | "producers"
                            | "sourceMappingURL"
                            | "external_debug_info"
                    ) || name.starts_with(".debug_"))
            {
                continue;
            }
        }
        output.push(id);
        unsigned(&mut output, range.len());
        output.extend_from_slice(&wasm[range]);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_removes_build_metadata_while_debug_preserves_it() {
        let mut input = b"\0asm\x01\0\0\0".to_vec();
        for name in [
            "name",
            "producers",
            ".debug_info",
            "ocamljs.schema",
            "target_features",
            "ocamljs.abi",
        ] {
            let mut data = Vec::new();
            unsigned(&mut data, name.len());
            data.extend_from_slice(name.as_bytes());
            input.push(0);
            unsigned(&mut input, data.len());
            input.extend(data);
        }
        let names = |wasm: &[u8]| {
            wasmparser::Parser::new(0)
                .parse_all(wasm)
                .filter_map(|payload| {
                    if let wasmparser::Payload::CustomSection(section) = payload.unwrap() {
                        Some(section.name().to_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(&strip_metadata(&input, false).unwrap()),
            ["target_features"]
        );
        assert_eq!(
            names(&strip_metadata(&input, true).unwrap()),
            [
                "name",
                "producers",
                ".debug_info",
                "ocamljs.schema",
                "target_features"
            ]
        );
    }
}
