//! Signature-specialized JS shims. No runtime schema interpreter.
use super::*;
use std::fmt::Write;

pub(super) fn scalar(t: &Ty, n: &str) -> String {
    match t.name.as_str() {
        "u8" => format!("integer({n},0,255)"),
        "i8" => format!("integer({n},-128,127)"),
        "u16" => format!("integer({n},0,65535)"),
        "i16" => format!("integer({n},-32768,32767)"),
        "u32" | "usize" => format!("integer({n},0,4294967295)"),
        "i32" | "isize" => format!("integer({n},-2147483648,2147483647)"),
        "bool" => format!("boolean({n})"),
        "f32" | "f64" => format!("number({n})"),
        "i64" => format!("wideInteger({n},false)"),
        "u64" => format!("wideInteger({n},true)"),
        _ => n.into(),
    }
}

pub(super) fn scalar_checked(paranoid: bool, t: &Ty, n: &str) -> String {
    if paranoid {
        scalar(t, n)
    } else {
        match t.name.as_str() {
            "bool" => format!("+{n}"),
            "i64" | "u64" => format!("BigInt({n})"),
            _ => n.into(),
        }
    }
}
fn plain_result(s: &Schema, t: &Ty, seen: &mut std::collections::BTreeSet<String>) -> bool {
    if references::host(s, t) || t.name == "Callback" {
        return false;
    }
    if !t.args.iter().all(|a| plain_result(s, a, seen))
        || t.inner.as_ref().is_some_and(|a| !plain_result(s, a, seen))
    {
        return false;
    }
    let Some(e) = s.entries.get(&t.name) else {
        return true;
    };
    if e.kind == "opaque" || e.kind == "host" {
        return false;
    }
    if !seen.insert(t.name.clone()) {
        return true;
    }
    let result = collections::fields(e).all(|f| plain_result(s, f, seen));
    seen.remove(&t.name);
    result
}
fn completion_scalar(t: &Ty, pointer: &str) -> String {
    if t.unit() {
        return "undefined".into();
    }
    let get = match t.name.as_str() {
        "bool" | "u8" => "getUint8",
        "i8" => "getInt8",
        "u16" => "getUint16",
        "i16" => "getInt16",
        "u32" | "usize" => "getUint32",
        "i32" | "isize" => "getInt32",
        "f32" => "getFloat32",
        "f64" => "getFloat64",
        "i64" | "u64" => "getBigInt64",
        _ => unreachable!(),
    };
    let value = format!("completionView(i).{get}({pointer},true)");
    if t.name == "bool" {
        format!("({value}!==0)")
    } else if matches!(t.name.as_str(), "i64" | "u64") {
        format!("{value}.toString()")
    } else {
        value
    }
}
pub(super) fn generate(
    s: &Schema,
    options: &Options,
    implementation: &str,
) -> Result<String, Error> {
    let async_owners = s.entries.values().any(|entry| {
        entry.asynchronous
            && entry
                .result
                .as_ref()
                .is_some_and(|ty| abi::owns_result(s, ty))
    });
    let flush = if s.entries.values().any(|e| e.kind == "opaque") {
        "if(i.finalizerPending.size)flushFinalizers(i);"
    } else {
        ""
    };
    let callbacks = s
        .entries
        .values()
        .any(|e| e.params.iter().any(|p| p["ty"]["name"] == "Callback"));
    let mut code = String::new();
    for e in s.entries.values().filter(|e| e.kind == "opaque") {
        if implementation.contains(&format!("\"r_handle_{}\"", snake(&e.name))) {
            writeln!(
                code,
                "methods.r_handle_{}=r=>methods.r_handle(r,{:?});",
                snake(&e.name),
                e.name
            )?;
        }
        writeln!(
            code,
            "methods.dispose_{}=h=>dispose(defaultInstance,h,{:?});",
            e.name, e.name
        )?;
    }
    for e in s.entries.values().filter(|e| e.kind == "function") {
        let guarded = options.paranoid || callbacks || references::used(s) || e.asynchronous;
        let instance = if options.paranoid {
            "inst(defaultInstance)"
        } else {
            "defaultInstance"
        };
        let busy_check = if guarded {
            "if(i.busy)fail('reentrant call');"
        } else {
            ""
        };
        let busy_enter = if guarded { "i.busy=true;" } else { "" };
        let busy_leave = if guarded {
            format!("i.busy=false;{flush}")
        } else {
            String::new()
        };
        let numeric_inputs = e.params.iter().any(|p| {
            serde_json::from_value::<Ty>(p["ty"].clone()).is_ok_and(|t| {
                abi::numeric_input(s, &t).is_some() || abi::text_input(s, &t).is_some()
            })
        });
        let mut snapshots = String::new();
        let mut parameters = Vec::new();
        let mut declarations = String::new();
        let mut prepare = String::new();
        let mut cleanup = String::new();
        let mut numeric_cleanup = String::new();
        let mut transferred = String::new();
        let mut moving = 0;
        let mut arguments = Vec::new();
        for p in &e.params {
            let ty: Ty = serde_json::from_value(p["ty"].clone())?;
            let lanes = if p["mode"] == "move" {
                vec![ty.clone()]
            } else {
                abi::arguments(s, &ty)
            };
            for t in lanes {
                let k = parameters.len();
                let a = format!("a{k}");
                let v = format!("v{k}");
                let binding = if numeric_inputs {
                    writeln!(declarations, "let {v};")?;
                    ""
                } else {
                    "const "
                };
                parameters.push(a.clone());
                arguments.push(v.clone());
                if let Some(input) = abi::text_input(s, &t) {
                    let n = format!("n{k}");
                    let cap = format!("c{k}");
                    let units = format!("u{k}");
                    writeln!(declarations, "let {n},{cap},{units};")?;
                    let fill = matches!(options.backend, Backend::JsOfOcaml) && !input.js;
                    let source = if fill {
                        let f = format!("a{}", k + 1);
                        parameters.push(f.clone());
                        f
                    } else {
                        a.clone()
                    };
                    if options.paranoid && !fill {
                        writeln!(
                            snapshots,
                            "if(typeof {a}!==\"string\")fail(\"expected string\");"
                        )?;
                    }
                    writeln!(
                        snapshots,
                        "{units}=Number({});",
                        if fill {
                            a.clone()
                        } else {
                            format!("{a}.length")
                        }
                    )?;
                    if options.paranoid {
                        writeln!(snapshots, "integer({units},0,MAX_BYTES);")?;
                    }
                    let prefix = format!("x{k}");
                    let unicode = input.js || matches!(options.backend, Backend::Rescript);
                    if !options.paranoid && !fill && unicode {
                        writeln!(
                            prepare,
                            "{v}=prepareUnicodeString(i,{source},{units});{n}={cap}=i.stringInputLength;"
                        )?;
                    } else if !options.paranoid && !fill {
                        writeln!(
                            prepare,
                            "{cap}={units};{n}={units};if({units}>256){{{v}=prepareNativeByteString(i,{source},{units});}}else{{{v}=i.e.__ocamljs_string_alloc({cap})>>>0;const bytes=stringView(i);for(let j=0;j<{units};j++)bytes[{v}+j]={source}.charCodeAt(j);}}"
                        )?;
                    } else {
                        if !fill {
                            writeln!(declarations, "let {prefix};")?;
                            writeln!(snapshots, "{prefix}={units}>32?stringPrefix({source}):-1;")?;
                        }
                        let capacity = if !fill && unicode {
                            let bound = format!("{prefix}+({units}-{prefix})*3");
                            let bound = if options.paranoid {
                                format!("Math.min({bound},MAX_BYTES)")
                            } else {
                                bound
                            };
                            format!("{prefix}<0?{units}:{bound}")
                        } else {
                            units.clone()
                        };
                        writeln!(
                            prepare,
                            "{cap}={capacity};{v}=i.e.__ocamljs_string_alloc({cap})>>>0;"
                        )?;
                        if fill {
                            writeln!(prepare, "{source}(stringView(i),{v});{n}={units};")?;
                        } else if input.js || matches!(options.backend, Backend::Rescript) {
                            if options.paranoid && !input.js {
                                writeln!(prepare, "checkedUnicodeScalars({source});")?;
                            }
                            let bound = if options.paranoid {
                                "Math.min(next,MAX_BYTES)"
                            } else {
                                "next"
                            };
                            let complete = if options.paranoid {
                                format!(
                                    "if(encoded.read!=={units}-j)fail('string input size limit');"
                                )
                            } else {
                                String::new()
                            };
                            writeln!(
                                prepare,
                                "if({prefix}>=0){{let j={prefix};const bytes=stringView(i);if(j<=32){{for(let at=0;at<j;at++)bytes[{v}+at]={source}.charCodeAt(at);}}else encoder.encodeInto(j==={units}?{source}:{source}.slice(0,j),bytes.subarray({v},{v}+j));{n}=j;if(j!=={units}){{const encoded=encoder.encodeInto({source}.slice(j),bytes.subarray({v}+j,{v}+{cap}));{complete}{n}=j+encoded.written;if({cap}!=={n}){{{v}=i.e.__ocamljs_string_realloc({v},{cap},{n})>>>0;{cap}={n};}}}}}}else"
                            )?;
                            writeln!(
                                prepare,
                                "{{let j=0;const bytes=stringView(i);for(;j<{units};j++){{const code={source}.charCodeAt(j);if(code>=128)break;bytes[{v}+j]=code;}}{n}=j;if(j!=={units}){{const next=j+({units}-j)*3,bound={bound};{v}=i.e.__ocamljs_string_realloc({v},{cap},bound)>>>0;{cap}=bound;const encoded=encoder.encodeInto({source}.slice(j),stringView(i).subarray({v}+j,{v}+{cap}));{complete}{n}=j+encoded.written;if({cap}!=={n}){{{v}=i.e.__ocamljs_string_realloc({v},{cap},{n})>>>0;{cap}={n};}}}}}}"
                            )?;
                        } else {
                            let check = if options.paranoid {
                                "if(code>255)fail('expected native byte string');"
                            } else {
                                ""
                            };
                            writeln!(
                                prepare,
                                "{{const bytes=stringView(i);let j=0;if({prefix}>32){{j={prefix};encoder.encodeInto(j==={units}?{source}:{source}.slice(0,j),bytes.subarray({v},{v}+j));}}for(;j<{units};j++){{const code={source}.charCodeAt(j);{check}bytes[{v}+j]=code;}}}}{n}={units};"
                            )?;
                        }
                    }
                    writeln!(prepare, "if(i.diagnostics)i.stats.inputBytes+={n};")?;
                    arguments.push(n);
                    arguments.push(cap.clone());
                    writeln!(
                        numeric_cleanup,
                        "if({v}!==undefined)freeStringInput(i,{v},{cap});"
                    )?;
                } else if let Some(input) = abi::numeric_input(s, &t) {
                    let length = format!("n{k}");
                    let offset = format!("o{k}");
                    writeln!(declarations, "let {length};")?;
                    let fill = matches!(options.backend, Backend::JsOfOcaml) && !input.js;
                    let source = if fill {
                        let source = format!("a{}", k + 1);
                        parameters.push(source.clone());
                        writeln!(snapshots, "{length}=Number({a});const {offset}=-1;")?;
                        source
                    } else {
                        writeln!(
                            snapshots,
                            "{length}=Number({a}.length);const {offset}={a}.byteOffset;"
                        )?;
                        a.clone()
                    };
                    if options.paranoid {
                        writeln!(snapshots, "integer({length},0,MAX_ITEMS);")?;
                    }
                    let kind = input.kind;
                    let copy = if fill {
                        "fill"
                    } else if input.js {
                        "js"
                    } else {
                        "native"
                    };
                    writeln!(
                        prepare,
                        "{v}=i.e.__ocamljs_numeric_alloc({length},{kind})>>>0;writeNumericInput_{kind}_{copy}(i,{v},{length},{source},{offset});"
                    )?;
                    arguments.push(length.clone());
                    writeln!(
                        numeric_cleanup,
                        "if({v}!==undefined)freeNumericInput(i,{v},{length},{kind});"
                    )?;
                } else if references::host(s, &t) {
                    if references::resolved(s, &t).name == "Promise" {
                        writeln!(prepare, "{binding}{v}=observeHostPromise({a});")?;
                    } else {
                        writeln!(prepare, "{binding}{v}={a};")?;
                    }
                } else if p["mode"] == "move"
                    || t.kind == "ref"
                        && t.inner.as_ref().is_some_and(|t| {
                            s.entries.get(&t.name).is_some_and(|e| e.kind == "opaque")
                        })
                {
                    let name = if t.kind == "ref" {
                        &t.inner.as_ref().unwrap().name
                    } else {
                        &t.name
                    };
                    let value = format!("handle(i,{a},{name:?})");
                    writeln!(
                        prepare,
                        "{binding}{v}={};",
                        if numeric_inputs {
                            format!("Number({value})")
                        } else {
                            value
                        }
                    )?;
                    if p["mode"] == "move" {
                        moving += 1;
                        writeln!(
                            transferred,
                            "if(moved>={moving}){{{a}.ptr=0;finalizers?.unregister({a});}}"
                        )?;
                    }
                } else if t.scalar() {
                    writeln!(
                        prepare,
                        "{binding}{v}={};",
                        if numeric_inputs && !matches!(t.name.as_str(), "i64" | "u64") {
                            format!("Number({})", scalar_checked(options.paranoid, &t, &a))
                        } else {
                            scalar_checked(options.paranoid, &t, &a)
                        }
                    )?;
                } else if t.name == "Callback" {
                    writeln!(
                        prepare,
                        "if(typeof {a}!==\"function\")fail(\"expected callback function\");{binding}{v}={a};"
                    )?;
                } else {
                    if !numeric_inputs {
                        writeln!(declarations, "let {v};")?;
                    }
                    writeln!(prepare, "{v}=encodeInput(i,{a});")?;
                    writeln!(cleanup, "if({v}!==undefined)freeInput(i,{v});")?;
                    if abi::vector(s, &t) {
                        arguments.push("0".into());
                    }
                }
            }
        }
        let result = e.result.as_ref().unwrap();
        let flat_return =
            abi::direct_result(s, result).map(|record| format!("readFlat_{}(i,r)", record.name));
        let opaque_return =
            abi::opaque(s, result).map(|opaque| format!("own(i,r>>>0,{:?})", opaque.name));
        let output_reader = if plain_result(s, result, &mut Default::default()) {
            "readOutput(i,r,true)"
        } else {
            "readOutput(i,r)"
        };
        let text_success = abi::text_result(s, result).filter(|_| !e.buffered);
        let string_result = if abi::text_output(s, result) {
            let js = abi::text_input(s, result).is_some_and(|t| t.js)
                || e.js_value
                || matches!(options.backend, Backend::Rescript);
            let reader = if js {
                "readUnicodeStringOutput"
            } else {
                "readNativeStringOutput"
            };
            Some(format!("{reader}(i,r>>>0)"))
        } else if let Some(ref success) = text_success {
            let js = abi::text_input(s, &success).is_some_and(|t| t.js)
                || abi::text_result_js(s, result) && success.name == "String"
                || e.js_value
                || matches!(options.backend, Backend::Rescript);
            let reader = if js {
                "readUnicodeStringOutput"
            } else {
                "readNativeStringOutput"
            };
            let plain = plain_result(s, result, &mut Default::default());
            Some(format!("readStringResultOutput(i,r>>>0,{reader},{plain})"))
        } else {
            None
        };
        let resolved = abi::resolve(s, result);
        let completion = if !abi::async_direct(s, e) {
            None
        } else if references::host(s, result) {
            Some("(i,r,decode)=>decode(i.e.__ocamljs_host_take(r))".into())
        } else if let Some(opaque) = abi::opaque(s, result) {
            Some(format!(
                "(i,r,decode,j)=>completeOpaque(i,r,decode,j,{:?})",
                opaque.name
            ))
        } else if let Some(text) = &string_result {
            if abi::text_result(s, result).is_some() {
                let error = abi::resolve(s, &resolved.args[1]);
                let scalar_error = if error.scalar() {
                    let name = format!("scalarError_{}", e.name);
                    writeln!(
                        code,
                        "const {name}=(i,p)=>{};",
                        completion_scalar(&error, "p")
                    )?;
                    name
                } else {
                    "null".into()
                };
                Some(format!(
                    "(i,r,decode,j)=>completeStringResult(i,r,decode,j,{}, {}, {scalar_error})",
                    if text.contains("readUnicodeStringOutput") {
                        "readUnicodeStringOutput"
                    } else {
                        "readNativeStringOutput"
                    },
                    plain_result(s, result, &mut Default::default())
                ))
            } else {
                Some(format!("(i,r,decode)=>decode({text})"))
            }
        } else if let Some(flat) = &flat_return {
            Some(format!("(i,r,decode)=>decode({flat})"))
        } else if resolved.scalar() {
            let raw = completion_scalar(&resolved, "r");
            Some(format!("(i,r,decode)=>decode({raw})"))
        } else {
            None
        };
        let async_convert = if let Some(completion) = completion {
            let discard = if abi::opaque(s, result).is_some() {
                "freeOwner"
            } else if references::host(s, result) {
                "discardHost"
            } else if text_success.is_some() {
                "discardStringResultOutput"
            } else if abi::text_output(s, result) {
                "discardStringOutput"
            } else {
                "null"
            };
            writeln!(code, "const complete_{}={completion};", e.name)?;
            format!(
                "createTask(i,r>>>0,{}, complete_{}, {discard})",
                !abi::owns_result(s, result),
                e.name
            )
        } else if async_owners {
            format!("createTask(i,r>>>0,{})", !abi::owns_result(s, result))
        } else {
            "createTask(i,r>>>0)".into()
        };
        let convert = if e.asynchronous {
            async_convert.as_str()
        } else if let Some(ref text) = string_result {
            text.as_str()
        } else if let Some(ref direct) = opaque_return {
            direct.as_str()
        } else if let Some(ref flat) = flat_return {
            flat.as_str()
        } else if references::host(s, result) {
            "r"
        } else if !result.scalar() {
            output_reader
        } else {
            match result.name.as_str() {
                "u32" | "usize" => "r>>>0",
                "bool" => "r!==0",
                "i64" | "u64" => "BigInt.asIntN(64,r).toString()",
                _ => "r",
            }
        };
        let name = &e.name;
        let parameter_list = parameters.join(",");
        let argument_list = arguments.join(",");
        let callback_origin = if callbacks { "&&!i.callbackError" } else { "" };
        let callback_state = if callbacks {
            "let callbackFailure;"
        } else {
            ""
        };
        let prepare_scope = if numeric_inputs { "try{" } else { "" };
        writeln!(
            code,
            "methods.call_{name}=function({parameter_list}){{\nconst i={instance};if(i.poisoned)fail('invalid or poisoned instance');{busy_check}\n{declarations}{callback_state}{busy_enter}\ntry{{\n{prepare_scope}{snapshots}{prepare}"
        )?;
        if e.asynchronous {
            writeln!(
                code,
                "if(i.jobs.size>=128)fail('task result limit exceeded');"
            )?;
        }
        writeln!(code, "if(i.diagnostics)i.stats.calls++;")?;
        if numeric_inputs {
            // Prepared numeric allocations belong to JS only until Rust entry.
            // Rollback is entirely on the exceptional preparation path.
            writeln!(
                code,
                "}}catch(error){{if(error instanceof WebAssembly.RuntimeError{callback_origin})poison(i,error);{numeric_cleanup}throw error;}}"
            )?;
        }
        if transferred.is_empty() {
            writeln!(code, "const r=i.e.__ocamljs_fn_{name}({argument_list});")?;
        } else {
            // Mark traps before reconciling consumed owners, since reconciliation enters Wasm.
            writeln!(
                code,
                "let r;try{{r=i.e.__ocamljs_fn_{name}({argument_list});}}catch(error){{if(error instanceof WebAssembly.RuntimeError{callback_origin})poison(i,error);throw error;}}finally{{if(!i.poisoned){{const moved=i.e.__ocamljs_moved();{transferred}}}}}"
            )?;
        }
        if callbacks {
            writeln!(
                code,
                "if(i.callbackError){{callbackFailure=i.callbackError;i.callbackError=null;"
            )?;
            if !e.asynchronous && opaque_return.is_some() {
                writeln!(code, "freeOwner(i,r);")?;
            } else if !e.buffered && !e.asynchronous && abi::text_result(s, result).is_some() {
                writeln!(code, "discardStringResultOutput(i,r>>>0);")?;
            } else if string_result.is_some() {
                writeln!(code, "discardStringOutput(i,r>>>0);")?;
            } else if !e.asynchronous
                && !result.scalar()
                && !references::host(s, result)
                && flat_return.is_none()
            {
                writeln!(code, "freeOutput(i,r);")?;
            }
            writeln!(code, "throw callbackFailure.value;}}")?;
        }
        let callback_trap = if callbacks {
            "&&!callbackFailure&&!i.callbackError"
        } else {
            ""
        };
        writeln!(
            code,
            "return {convert};\n}}catch(error){{if(error instanceof WebAssembly.RuntimeError{callback_trap})poison(i,error);"
        )?;
        if references::used(s) {
            writeln!(code, "if(!i.poisoned)i.e.__ocamljs_host_clear();")?;
        }
        if callbacks {
            writeln!(
                code,
                "const failure=callbackFailure??i.callbackError;i.callbackError=null;if(failure)throw failure.value;"
            )?;
        }
        writeln!(code, "throw error;}}finally{{")?;
        if cleanup.is_empty() {
            writeln!(code, "{busy_leave}")?;
        } else {
            writeln!(code, "try{{{cleanup}}}finally{{{busy_leave}}}")?;
        }
        writeln!(code, "}}}};")?;
    }
    Ok(code)
}
