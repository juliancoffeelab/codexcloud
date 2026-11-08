//! Bulk primitive codecs use an explicit JS-array FFI, never OCaml object layouts.
use super::*;
use std::fmt::Write;

pub(super) fn numeric(t: &Ty) -> bool {
    t.scalar() && !t.unit() && !matches!(t.name.as_str(), "bool" | "u64" | "i64")
}
pub(super) fn primitive(t: &Ty) -> bool {
    t.scalar() && !t.unit() || t.kind == "named" && t.name == "String"
}
pub(super) fn numeric_record(e: &Entry) -> bool {
    e.body
        .as_ref()
        .unwrap()
        .fields
        .iter()
        .all(|f| numeric(&f.ty))
}
fn method(t: &Ty) -> &str {
    match t.name.as_str() {
        "String" => "string",
        "usize" => "u32",
        "isize" => "i32",
        n => n,
    }
}
pub(super) fn record(e: &Entry) -> bool {
    e.kind == "struct"
        && !e.transparent
        && e.params.is_empty()
        && e.body.as_ref().is_some_and(|b| {
            !b.fields.is_empty()
                && b.fields.len() <= 64
                && b.fields.iter().all(|f| primitive(&f.ty))
        })
}
pub(super) fn generate(
    s: &Schema,
    implementation: &str,
    options: &Options,
) -> Result<String, Error> {
    let mut out = String::new();
    if !options.paranoid && !matches!(options.backend, Backend::JsOfOcaml) {
        for e in s
            .entries
            .values()
            .filter(|e| packet_record(s, e) && !record(e))
        {
            let method = format!("w_record_{}", snake(&e.name));
            if !implementation.contains(&format!("\"{method}\"")) {
                continue;
            }
            let fields = &e.body.as_ref().unwrap().fields;
            let args = (0..fields.len())
                .map(|j| format!("v{j}"))
                .collect::<Vec<_>>()
                .join(",");
            writeln!(out, "methods.{method}=function(w,{args}){{")?;
            for (j, f) in fields.iter().enumerate() {
                out.push_str(&packet_write(s, &f.ty, &format!("v{j}")));
            }
            out.push_str("};\n");
        }
    }

    for e in s.entries.values().filter(|e| {
        e.flat
            && e.body
                .as_ref()
                .unwrap()
                .fields
                .iter()
                .all(|f| numeric(&f.ty))
    }) {
        if !s.entries.values().any(|f| {
            f.kind == "function"
                && f.result
                    .as_ref()
                    .and_then(|t| abi::direct_result(s, t))
                    .is_some_and(|r| r.name == e.name)
        }) {
            continue;
        }
        let b = e.body.as_ref().unwrap();
        writeln!(
            out,
            "function readFlat_{}(i,p){{p>>>=0;const d=completionView(i);return [",
            e.name
        )?;
        let mut offset = 0;
        for f in &b.fields {
            let (size, get, _) = layout(&f.ty);
            writeln!(out, "d.{get}(p+{offset},true),")?;
            offset += size;
        }
        out.push_str("]; }\n");
    }
    for e in s.entries.values().filter(|e| record(e)) {
        let name = snake(&e.name);
        for direction in ["r", "w"] {
            let method = format!("{direction}_record_{name}");
            if !implementation.contains(&format!("\"{method}\"")) {
                continue;
            }
            let b = e.body.as_ref().unwrap();
            if !numeric_record(e) {
                if direction == "w" {
                    let args = (0..b.fields.len())
                        .map(|i| format!("v{i}"))
                        .collect::<Vec<_>>()
                        .join(",");
                    writeln!(out, "methods.{method}=function(w,{args}){{")?;
                    for (i, f) in b.fields.iter().enumerate() {
                        writeln!(out, "methods.w_{}(w,v{i});", self::method(&f.ty))?;
                    }
                    out.push_str("};\n");
                } else {
                    writeln!(out, "methods.{method}=r=>[")?;
                    for f in &b.fields {
                        writeln!(out, "methods.r_{}(r),", self::method(&f.ty))?;
                    }
                    out.push_str("];\n");
                }
                continue;
            }
            let params = (0..b.fields.len())
                .map(|i| format!("v{i}"))
                .collect::<Vec<_>>();
            if direction == "w" {
                writeln!(out, "methods.{method}=function(w,{}){{", params.join(","))?;
                for (f, v) in b.fields.iter().zip(&params) {
                    writeln!(
                        out,
                        "{};",
                        super::calls::scalar_checked(options.paranoid, &f.ty, v)
                    )?;
                }
                writeln!(
                    out,
                    "const p=w.reserve({});",
                    b.fields.iter().map(|f| layout(&f.ty).0).sum::<usize>()
                )?;
            } else {
                writeln!(
                    out,
                    "methods.{method}=function(r){{const p=r.take({}),d=r.view();return [",
                    b.fields.iter().map(|f| layout(&f.ty).0).sum::<usize>()
                )?;
            }
            let mut offset = 0;
            for (f, v) in b.fields.iter().zip(params) {
                let (size, get, set) = layout(&f.ty);
                if direction == "w" {
                    writeln!(out, "w.data.{set}(p+{offset},{v},true);")?;
                } else {
                    writeln!(out, "d.{get}(p+{offset},true),")?;
                }
                offset += size;
            }
            out.push_str(if direction == "w" { "};\n" } else { "];};\n" });
        }
    }
    for (name, size, get, set) in NUMBERS {
        for direction in ["r", "w"] {
            let method = format!("{direction}_array_{name}");
            if !implementation.contains(&format!("\"{method}\"")) {
                continue;
            }
            if direction == "w" && !options.paranoid {
                let ctor = match name {
                    "u8" => "Uint8Array",
                    "i8" => "Int8Array",
                    "u16" => "Uint16Array",
                    "i16" => "Int16Array",
                    "u32" => "Uint32Array",
                    "i32" => "Int32Array",
                    "f32" => "Float32Array",
                    "f64" => "Float64Array",
                    _ => unreachable!(),
                };
                writeln!(
                    out,
                    "methods.{method}=(w,a)=>{{methods.w_length(w,a.length);writeTyped(w,{ctor}.from(a),{set:?});}};"
                )?;
            } else if direction == "w" {
                writeln!(
                    out,
                    "methods.{method}=(w,a)=>{{methods.w_length(w,a.length);const p=w.reserve(a.length*{size});for(let j=0;j<a.length;j++){{{};w.data.{set}(p+j*{size},a[j],true);}}}};",
                    super::calls::scalar_checked(
                        options.paranoid,
                        &Ty {
                            kind: "named".into(),
                            name: name.into(),
                            qualified: false,
                            args: vec![],
                            inner: None,
                            len: None,
                            mutable: false,
                            map: String::new()
                        },
                        "a[j]"
                    )
                )?;
            } else {
                writeln!(
                    out,
                    "methods.{method}=r=>{{const n=methods.r_length(r),p=r.take(n*{size}),d=r.view(),a=new Array(n);for(let j=0;j<n;j++)a[j]=d.{get}(p+j*{size},true);return a;}};"
                )?;
            }
        }
    }
    Ok(out)
}
pub(super) const NUMBERS: [(&str, usize, &str, &str); 8] = [
    ("u8", 1, "getUint8", "setUint8"),
    ("i8", 1, "getInt8", "setInt8"),
    ("u16", 2, "getUint16", "setUint16"),
    ("i16", 2, "getInt16", "setInt16"),
    ("u32", 4, "getUint32", "setUint32"),
    ("i32", 4, "getInt32", "setInt32"),
    ("f32", 4, "getFloat32", "setFloat32"),
    ("f64", 8, "getFloat64", "setFloat64"),
];
fn layout(t: &Ty) -> (usize, &'static str, &'static str) {
    let n = match t.name.as_str() {
        "usize" => "u32",
        "isize" => "i32",
        n => n,
    };
    let (_, size, get, set) = NUMBERS.iter().find(|(name, ..)| *name == n).unwrap();
    (*size, *get, *set)
}

/// Keep each typed-array copy site monomorphic, including mixed numeric calls.
pub(super) fn numeric_input_writer(kind: usize, copy: &str, paranoid: bool) -> String {
    let index = match kind {
        8 => 4,
        9 => 5,
        _ => kind,
    };
    let (_, size, _, setter) = NUMBERS[index];
    let ctor = [
        "Uint8Array",
        "Int8Array",
        "Uint16Array",
        "Int16Array",
        "Uint32Array",
        "Int32Array",
        "Float32Array",
        "Float64Array",
    ][index];
    let check = if paranoid {
        format!("for(let j=0;j<length;j++)numericValue({kind},source[j]);")
    } else {
        String::new()
    };
    let write = if copy == "fill" {
        format!("source(view,pointer/{size});")
    } else {
        let little_copy = if copy == "native" {
            // Small ordinary arrays are faster to fill directly; preserve bulk copying for large inputs.
            format!(
                "if(length<=32){{const at=pointer/{size};for(let j=0;j<length;j++)view[at+j]=source[j];}}else view.set(source,pointer/{size});"
            )
        } else {
            // A one-element bulk-copy call costs more than the store itself.
            // Preserve typed-array NaN payloads via the bit-copy operation.
            let store = if index >= 6 {
                format!(
                    "const value=source[0];if(value===value)view[pointer/{size}]=value;else view.set(source,pointer/{size});"
                )
            } else {
                format!("view[pointer/{size}]=source[0];")
            };
            format!("if(length===1){{{store}}}else view.set(source,pointer/{size});")
        };
        format!(
            "if(source.length!==length)source=new {ctor}(view.buffer,offset,length);{check}if(littleEndian||{size}===1){{{little_copy}}}else for(let j=0;j<length;j++)view.{setter}(pointer+j*{size},source[j],true);"
        )
    };
    format!(
        "function writeNumericInput_{kind}_{copy}(i,pointer,length,source,offset){{if(length===0)return;let view=i.numericViews[{kind}];if(!view||(littleEndian||{size}===1?view.byteLength===0:view.buffer.byteLength===0)){{const memory=i.e.memory.buffer;view=littleEndian||{size}===1?new {ctor}(memory):new DataView(memory);i.numericViews[{kind}]=view;}}{write}if(i.diagnostics)i.stats.inputBytes+=length*{size};}}\n"
    )
}

// Compound native fields still cross the JS FFI as typed scalar/array values;
// keep their byte serialization in one generated call rather than one per field.
pub(super) fn packet_record(s: &Schema, e: &Entry) -> bool {
    fn field(s: &Schema, t: &Ty) -> bool {
        let r = abi::resolve(s, t);
        numeric(&r)
            || r.name == "bool"
            || abi::text_output(s, t)
            || r.name == "Vec" && r.args.len() == 1 && abi::text_output(s, &r.args[0])
    }
    e.kind == "struct"
        && !e.transparent
        && e.params.is_empty()
        && !e.js_value
        && e.body.as_ref().is_some_and(|b| {
            !b.fields.is_empty()
                && b.fields.len() <= 64
                && b.fields
                    .iter()
                    .all(|f| !projection::marked(f) && field(s, &f.ty))
        })
}
fn packet_write(s: &Schema, t: &Ty, value: &str) -> String {
    if let Some(text) = abi::text_input(s, t) {
        return format!(
            "methods.w_{}(w,{value});",
            if text.js { "js_string" } else { "string" }
        );
    }
    let r = abi::resolve(s, t);
    if r.name == "Vec" {
        return format!(
            "methods.w_length(w,{value}.length);for(let j=0;j<{value}.length;j++){{{}}}",
            packet_write(s, &r.args[0], &format!("{value}[j]"))
        );
    }
    format!("methods.w_{}(w,{value});", method(&r))
}
