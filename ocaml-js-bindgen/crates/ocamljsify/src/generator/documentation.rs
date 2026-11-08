//! Standard OCaml documentation attributes preserve Rust text without letting it
//! terminate comments or alter generated declarations. Odoc consumes these just
//! as it consumes documentation comments.
use super::*;

fn literal(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write;
                for b in c.encode_utf8(&mut [0; 4]).bytes() {
                    write!(out, "\\{b:03}").unwrap();
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub(super) fn attribute(text: &str, item: bool) -> String {
    if text.trim().is_empty() {
        String::new()
    } else {
        format!(
            " [@{}ocaml.doc {}]",
            if item { "@" } else { "" },
            literal(text)
        )
    }
}

pub(super) fn type_docs(e: &Entry) -> String {
    let mut paragraphs = vec![e.docs.trim().to_owned()];
    if e.kind == "opaque" {
        paragraphs.push("Opaque Rust-owned value. Call [dispose] to release it deterministically. Rust borrows are checked at runtime; the value cannot be used after disposal or transfer to Rust.".into());
    }
    if e.kind == "host" {
        paragraphs.push("A JavaScript host value passed by reference without serialization. The host GC owns its lifetime; there is no Rust disposal operation. Rust can retain it across calls and async suspension.".into());
    }
    if let Some(b) = &e.body {
        if b.style == "tuple" || e.transparent {
            for f in &b.fields {
                if !f.docs.is_empty() {
                    paragraphs.push(format!("Field {}: {}", f.name, f.docs.trim()));
                }
            }
        }
    }
    join(paragraphs)
}

fn join(paragraphs: Vec<String>) -> String {
    paragraphs
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(super) fn exception_name(e: &Entry) -> String {
    let mut name = format!(
        "{}_error",
        if e.module.is_empty() {
            e.name.trim_start_matches('_').to_owned()
        } else {
            modules::member(e)
        }
    );
    name.get_mut(0..1).unwrap().make_ascii_uppercase();
    name
}

pub(super) fn function(e: &Entry, backend: Backend, lwt: bool) -> String {
    let mut paragraphs = vec![e.docs.trim().to_owned()];
    if e.throws {
        paragraphs.push(format!(
            "Rust [Err] {} [{}] with the error payload; [Ok] returns its value.",
            if e.asynchronous {
                "rejects the returned computation with"
            } else {
                "raises"
            },
            exception_name(e)
        ));
    } else if e.result.as_ref().is_some_and(|t| t.name == "Result") {
        paragraphs.push("Rust [Ok] and [Err] map to native OCaml [Ok] and [Error]. Binding failures are separate from these application errors.".into());
    }
    for (index, p) in e.params.iter().enumerate() {
        if p["mode"] == "move" {
            paragraphs.push(format!(
                "Transfers ownership of argument {} to Rust; do not use that value after the call.",
                index + 1
            ));
        }
    }
    if e.asynchronous {
        paragraphs.push(format!("Binding failures reject the returned computation. Rust borrows remain held while its future is pending; conflicting use or disposal fails. {}", if matches!(backend, Backend::JsOfOcaml) && lwt {
            "Cancelling the Lwt computation drops its Rust future and releases its resources."
        } else {
            "Promises have no built-in cancellation; use an exported Rust cancellation operation where available."
        }));
    } else {
        paragraphs.push(
            "Binding failures raise an exception. See [Wasm] for initialization and trap recovery."
                .into(),
        );
    }
    join(paragraphs)
}

pub(super) const DISPOSE: &str = "Runs Rust Drop and invalidates this value. Call exactly once, after all borrows have ended. Disposal fails for a borrowed, already disposed, consumed, or foreign value. Finalization is a nondeterministic fallback; explicit disposal is preferred. A Rust trap makes the Wasm module unusable.";

/// Exclude documentation strings from type/module dependency discovery.
pub(super) fn without_strings(text: &str) -> String {
    let mut quoted = false;
    let mut escaped = false;
    text.chars()
        .map(|c| {
            if quoted {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    quoted = false;
                }
                ' '
            } else if c == '"' {
                quoted = true;
                ' '
            } else {
                c
            }
        })
        .collect()
}
