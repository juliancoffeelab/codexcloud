//! Explicit Rust type/module associations. Ownership never determines grouping by heuristic.
use super::*;
use std::{collections::BTreeSet, fmt::Write};

/// Mixed native data can reference carrier aliases; ordinary opaques expose only Module.t.
pub(super) fn public_carriers(s: &Schema) -> BTreeSet<String> {
    fn walk(s: &Schema, t: &Ty, out: &mut BTreeSet<String>) {
        if s.entries
            .get(&t.name)
            .is_some_and(|e| matches!(e.kind.as_str(), "opaque" | "host"))
        {
            out.insert(t.name.clone());
        }
        for a in &t.args {
            walk(s, a, out);
        }
        if let Some(inner) = &t.inner {
            walk(s, inner, out);
        }
    }
    let mut out = BTreeSet::new();
    for e in s.entries.values().filter(|e| e.is_type()) {
        for t in collections::fields(e) {
            walk(s, t, &mut out);
        }
        if e.module.is_empty() {
            for t in collections::fields(e) {
                native_carriers(s, t, &mut out);
            }
        }
    }
    fn native_carriers(s: &Schema, t: &Ty, out: &mut BTreeSet<String>) {
        if let Some(e) = s
            .entries
            .get(&t.name)
            .filter(|e| e.is_type() && owner(e).is_some())
        {
            if out.insert(e.name.clone()) {
                for t in collections::fields(e) {
                    native_carriers(s, t, out);
                }
            }
        }
        for t in &t.args {
            native_carriers(s, t, out);
        }
        if let Some(t) = &t.inner {
            native_carriers(s, t, out);
        }
    }
    // Map.S key equalities are emitted outside the data modules.
    fn map_keys(s: &Schema, t: &Ty, out: &mut BTreeSet<String>) {
        if matches!(t.name.as_str(), "BTreeMap" | "HashMap") && t.map != "dict" {
            native_carriers(s, &t.args[0], out);
        }
        for t in &t.args {
            map_keys(s, t, out);
        }
        if let Some(t) = &t.inner {
            map_keys(s, t, out);
        }
    }
    for e in s.entries.values().filter(|e| e.kind != "module") {
        for t in collections::fields(e).chain(e.result.iter()) {
            map_keys(s, t, &mut out);
        }
        if e.kind == "function" {
            for p in &e.params {
                if let Ok(t) = serde_json::from_value::<Ty>(p["ty"].clone()) {
                    map_keys(s, &t, &mut out);
                }
            }
        }
    }
    out
}
pub(super) fn descriptor<'a>(s: &'a Schema, name: &str) -> Option<&'a Entry> {
    s.entries
        .values()
        .find(|e| e.kind == "module" && e.member == name)
}
pub(super) fn owner<'a>(e: &'a Entry) -> Option<&'a str> {
    if !e.module.is_empty() {
        Some(&e.module)
    } else if matches!(e.kind.as_str(), "opaque" | "host") {
        Some(&e.name)
    } else {
        None
    }
}
fn local_type(s: &Schema, e: &Entry) -> String {
    if e.module.is_empty()
        || descriptor(s, &e.module)
            .and_then(|d| d.ty.as_ref())
            .is_some_and(|t| t.name == e.name)
    {
        "t".into()
    } else {
        label(&snake(&e.member))
    }
}
fn selected_type<'a>(s: &'a Schema, module: &str) -> Option<&'a Entry> {
    if let Some(d) = descriptor(s, module) {
        d.ty.as_ref().and_then(|t| s.entries.get(&t.name))
    } else {
        s.entries
            .get(module)
            .filter(|e| matches!(e.kind.as_str(), "opaque" | "host"))
    }
}
pub(super) fn member(e: &Entry) -> String {
    let name = if e.member.is_empty() {
        &e.name
    } else {
        &e.member
    };
    label(name)
}

pub(super) fn validate(s: &Schema) -> Result<(), Error> {
    let mut names = BTreeSet::new();
    for module in s.entries.values().filter(|e| e.kind == "module") {
        let name = &module.member;
        if name.is_empty()
            || !name.starts_with(|c: char| c.is_ascii_uppercase())
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || matches!(name.as_str(), "Wasm" | "Internal")
            || !names.insert(name.clone())
        {
            return Err(format!("invalid, reserved or duplicate module name {name}").into());
        }
        if let Some(t) = &module.ty {
            if t.kind != "named"
                || !t.args.is_empty()
                || s.entries
                    .get(&t.name)
                    .is_none_or(|e| !e.is_type() || e.module != *name)
            {
                return Err(format!("{name}: t must select a type defined in this module").into());
            }
        }
    }
    for ty in s.entries.values().filter(|e| e.is_type()) {
        if !ty.module.is_empty() && descriptor(s, &ty.module).is_none() {
            return Err(format!("{}: unknown module {}", ty.name, ty.module).into());
        }
        if ty.kind == "opaque"
            && !ty.module.is_empty()
            && selected_type(s, &ty.module).is_none_or(|t| t.name != ty.name)
        {
            return Err(format!("{}: select an opaque module type with t = Type", ty.name).into());
        }
        if matches!(ty.kind.as_str(), "opaque" | "host") && ty.module.is_empty() {
            if matches!(ty.name.as_str(), "Wasm" | "Internal") || !names.insert(ty.name.clone()) {
                return Err(format!("reserved or duplicate module name {}", ty.name).into());
            }
        }
    }
    let mut types = BTreeSet::new();
    for e in s
        .entries
        .values()
        .filter(|e| e.is_type() && owner(e).is_some())
    {
        let name = local_type(s, e);
        if name.is_empty()
            || !name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || matches!(
                name.as_str(),
                "unit"
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
            )
            || !types.insert((owner(e).unwrap(), name.clone()))
        {
            return Err(format!("{}: colliding module type {name}", e.name).into());
        }
    }
    let mut members = BTreeSet::new();
    for e in s.entries.values().filter(|e| e.kind == "function") {
        if e.module.is_empty() {
            let mut types = e
                .params
                .iter()
                .map(|p| serde_json::from_value::<Ty>(p["ty"].clone()))
                .collect::<Result<Vec<_>, _>>()?;
            types.extend(e.result.clone());
            if types
                .iter()
                .any(|t| validation::contains_opaque(s, t, &mut BTreeSet::new()))
            {
                return Err(format!(
                    "{}: opaque operations require an explicit module association or exported impl",
                    e.name
                )
                .into());
            }
        } else {
            if descriptor(s, &e.module).is_none() && selected_type(s, &e.module).is_none() {
                return Err(format!("{}: unknown module {}", e.name, e.module).into());
            }
            let name = member(e);
            let dispose = selected_type(s, &e.module).is_some_and(|t| t.kind == "opaque");
            if name.is_empty()
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || name.starts_with(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit())
                || (name == "dispose" && dispose)
                || !members.insert((e.module.clone(), name))
            {
                return Err(format!(
                    "{}: invalid, colliding or reserved module operation",
                    e.name
                )
                .into());
            }
        }
    }
    Ok(())
}

/// Rewrite type identifiers only; generated function signatures are concrete.
pub(super) fn scoped_type(s: &Schema, context: &str, text: &str) -> String {
    let names = s
        .entries
        .values()
        .filter(|e| e.is_type())
        .filter_map(|e| {
            owner(e).map(|module| {
                (
                    label(&snake(&e.name)),
                    if module == context {
                        local_type(s, e)
                    } else {
                        format!("{module}.{}", local_type(s, e))
                    },
                )
            })
        })
        .collect::<BTreeMap<_, String>>();
    let mut out = String::new();
    let mut token = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for (at, c) in text
        .char_indices()
        .chain(std::iter::once((text.len(), ' ')))
    {
        if quoted {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
            continue;
        }
        if c == '"' {
            quoted = true;
        }
        if c.is_ascii_alphanumeric() || c == '_' {
            token.push(c);
        } else {
            if !token.is_empty() {
                out.push_str(
                    if text[at..].trim_start().starts_with(':')
                        || out.ends_with('.')
                        || out.ends_with('\'')
                    {
                        &token
                    } else {
                        names.get(&token).map(String::as_str).unwrap_or(&token)
                    },
                );
                token.clear();
            }
            out.push(c);
        }
    }
    out.pop();
    out
}
#[derive(Default)]
pub(super) struct Modules {
    signatures: BTreeMap<String, String>,
    bodies: BTreeMap<String, String>,
    types: BTreeMap<String, BTreeMap<String, (String, String)>>,
}
impl Modules {
    pub fn function(
        &mut self,
        s: &Schema,
        e: &Entry,
        signature: &str,
        backend: Backend,
        lwt: bool,
    ) -> Result<(), Error> {
        let name = member(e);
        writeln!(
            self.signatures.entry(e.module.clone()).or_default(),
            "val {name} : {}{}",
            scoped_type(s, &e.module, signature),
            documentation::attribute(&documentation::function(e, backend, lwt), true)
        )?;
        writeln!(
            self.bodies.entry(e.module.clone()).or_default(),
            "let {name} = {}",
            label(&e.name)
        )?;
        Ok(())
    }
    pub fn exception(
        &mut self,
        s: &Schema,
        e: &Entry,
        private_name: &str,
        ty: &str,
    ) -> Result<(), Error> {
        let mut name = format!("{}_error", member(e));
        name.get_mut(0..1).unwrap().make_ascii_uppercase();
        writeln!(
            self.signatures.entry(e.module.clone()).or_default(),
            "exception {name} of {}{}",
            scoped_type(s, &e.module, ty),
            documentation::attribute(
                "Carries the Rust Err payload for this throwing function.",
                true
            )
        )?;
        writeln!(
            self.bodies.entry(e.module.clone()).or_default(),
            "exception {name} = {private_name}"
        )?;
        Ok(())
    }
    pub fn typ(
        &mut self,
        s: &Schema,
        e: &Entry,
        prefix: &str,
        def: &str,
        carriers: &BTreeSet<String>,
        carrier: &str,
    ) -> Result<(), Error> {
        let Some(module) = owner(e) else {
            return Ok(());
        };
        if e.module.is_empty() {
            return Ok(());
        } // Struct/impl module: added below.
        let name = local_type(s, e);
        let alias = if e.kind == "enum" && !e.js_value {
            format!("{prefix}{carrier}.{}", label(&snake(&e.name)))
        } else {
            format!("{prefix}{}", label(&snake(&e.name)))
        };
        let representation = matches!(e.kind.as_str(), "struct" | "enum")
            && !e.js_value
            && !e.transparent
            && !def.is_empty();
        let repeated = if representation {
            format!(" {def}")
        } else {
            String::new()
        };
        let body = format!("{prefix}{name} = {alias}{repeated}");
        let mut signature = if carriers.contains(&e.name) {
            format!("{prefix}{name} = {alias}{repeated}")
        } else {
            format!("{prefix}{name} {}", scoped_type(s, module, def))
        };
        signature.push_str(&documentation::attribute(
            &documentation::type_docs(e),
            true,
        ));
        self.types
            .entry(module.into())
            .or_default()
            .insert(e.name.clone(), (signature, body));
        Ok(())
    }
    pub fn finish(
        mut self,
        s: &Schema,
        interface: &mut enum_modules::Interface,
    ) -> Result<String, Error> {
        let carriers = public_carriers(s);
        for d in s.entries.values().filter(|e| e.kind == "module") {
            self.signatures.entry(d.member.clone()).or_default();
            self.bodies.entry(d.member.clone()).or_default();
        }
        for ty in s
            .entries
            .values()
            .filter(|e| e.is_type() && owner(e).is_some())
        {
            let module = owner(ty).unwrap();
            let sig = self.signatures.entry(module.into()).or_default();
            let body = self.bodies.entry(module.into()).or_default();
            if ty.kind == "opaque" {
                writeln!(
                    sig,
                    "val dispose : t -> unit{}",
                    documentation::attribute(documentation::DISPOSE, true)
                )?;
                writeln!(body, "let dispose = Internal.dispose_{}", snake(&ty.name))?;
            }
            if ty.module.is_empty() {
                let alias = format!("type nonrec t = {}\n", label(&snake(&ty.name)));
                *sig = format!(
                    "{}{sig}",
                    if matches!(ty.kind.as_str(), "opaque" | "host") && !carriers.contains(&ty.name)
                    {
                        format!(
                            "type t{}\n",
                            documentation::attribute(&documentation::type_docs(ty), true)
                        )
                    } else {
                        format!(
                            "{}{}\n",
                            alias.trim_end(),
                            documentation::attribute(&documentation::type_docs(ty), true)
                        )
                    }
                );
                *body = format!("{alias}{body}");
            }
        }
        for (module, types) in self.types {
            let mut declarations = String::new();
            let mut aliases = String::new();
            // One recursive type group permits ordinary mutually recursive data.
            for (index, (_, (signature, body))) in types.into_iter().enumerate() {
                let kw = if index == 0 { "type" } else { "and" };
                writeln!(declarations, "{kw} {signature}")?;
                let kw = if index == 0 { "type nonrec" } else { "and" };
                writeln!(aliases, "{kw} {body}")?;
            }
            let sig = self.signatures.entry(module.clone()).or_default();
            *sig = format!("{declarations}{sig}");
            let body = self.bodies.entry(module).or_default();
            *body = format!("{aliases}{body}");
        }
        let graph = self
            .signatures
            .iter()
            .map(|(name, signature)| {
                (
                    name.clone(),
                    self.signatures
                        .keys()
                        .filter(|other| {
                            *other != name
                                && documentation::without_strings(signature)
                                    .split(|c: char| {
                                        !c.is_ascii_alphanumeric() && c != '_' && c != '.'
                                    })
                                    .any(|word| word.starts_with(&format!("{other}.")))
                        })
                        .cloned()
                        .collect(),
                )
            })
            .collect::<BTreeMap<_, BTreeSet<_>>>();
        let mut ml = String::new();

        for component in collections::groups(&graph) {
            for name in &component {
                let signature = &self.signatures[name];
                let body = &self.bodies[name];
                let docs = descriptor(s, name).map(|d| d.docs.as_str()).unwrap_or("");
                interface.module(
                    name,
                    format!(
                        "module {name} : sig\n{signature}end{}",
                        documentation::attribute(docs, true)
                    ),
                );
                // Only signatures refer to other modules. Implementations alias private
                // functions/carriers, so no recursive-module initialization or trampolines.
                writeln!(ml, "module {name} = struct\n{body}end")?;
            }
        }
        Ok(ml)
    }
}
