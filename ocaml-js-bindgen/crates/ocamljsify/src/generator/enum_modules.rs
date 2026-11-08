//! Public enum namespaces. Private carrier types keep codec representation stable.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Layout {
    pub schema: Schema,
    pub automatic: BTreeSet<String>,
    pub companions: BTreeSet<String>,
}

pub(super) fn prepare(schema: &Schema) -> Result<Layout, Error> {
    let mut layout = Layout {
        schema: schema.clone(),
        automatic: BTreeSet::new(),
        companions: BTreeSet::new(),
    };
    let mut occupied = schema
        .entries
        .values()
        .filter(|e| {
            e.kind == "module"
                || matches!(e.kind.as_str(), "opaque" | "host") && e.module.is_empty()
        })
        .map(|e| {
            if e.kind == "module" {
                e.member.clone()
            } else {
                e.name.clone()
            }
        })
        .chain(["Internal".into(), "Wasm".into()])
        .collect::<BTreeSet<_>>();
    for entry in schema
        .entries
        .values()
        .filter(|e| e.kind == "enum" && e.module.is_empty())
    {
        let mut module = snake(&entry.name);
        module.get_mut(0..1).unwrap().make_ascii_uppercase();
        if !occupied.insert(module.clone()) {
            return Err(format!(
                "{}: enum module {module} collides with another binding module",
                entry.name
            )
            .into());
        }
        let descriptor: Entry = serde_json::from_value(serde_json::json!({
            "kind": "module", "name": format!("__ocamljs_enum_module_{}", entry.name),
            "member": module, "ty": { "kind": "named", "name": entry.name }
        }))?;
        layout
            .schema
            .entries
            .insert(descriptor.name.clone(), descriptor);
        let target = layout.schema.entries.get_mut(&entry.name).unwrap();
        target.module = module;
        target.member = entry.name.clone();
        layout.automatic.insert(entry.name.clone());
    }
    // An enum/record cycle needs a name for the record inside the recursive
    // namespace. Re-export that same record at its original public root later;
    // never expose a duplicate root alias for the enum itself.
    fn dependencies(schema: &Schema, ty: &Ty, out: &mut BTreeSet<String>) {
        if schema.entries.get(&ty.name).is_some_and(Entry::is_type) {
            out.insert(ty.name.clone());
        }
        for arg in &ty.args {
            dependencies(schema, arg, out);
        }
        if let Some(inner) = &ty.inner {
            dependencies(schema, inner, out);
        }
    }
    let graph = schema
        .entries
        .values()
        .filter(|e| e.is_type())
        .map(|entry| {
            let mut deps = BTreeSet::new();
            for ty in collections::fields(entry) {
                dependencies(schema, ty, &mut deps);
            }
            (entry.name.clone(), deps)
        })
        .collect();
    for component in collections::groups(&graph) {
        let Some(anchor) = component
            .iter()
            .find(|name| layout.automatic.contains(*name))
        else {
            continue;
        };
        let module = layout.schema.entries[anchor].module.clone();
        for name in &component {
            let entry = layout.schema.entries.get_mut(name).unwrap();
            if entry.module.is_empty() {
                entry.module = module.clone();
                entry.member = entry.name.clone();
                layout.companions.insert(name.clone());
            }
        }
    }
    Ok(layout)
}

#[derive(Default)]
pub(super) struct Interface {
    items: BTreeMap<String, String>,
    types: BTreeMap<String, String>,
    modules: BTreeMap<String, String>,
    locals: BTreeMap<String, BTreeSet<String>>,
}
impl Interface {
    pub fn types(&mut self, names: Vec<String>, declaration: String) {
        if names.is_empty() {
            return;
        }
        let key = format!("type:{}", names[0]);
        for name in names {
            self.types.insert(name, key.clone());
        }
        self.items.insert(key, declaration);
    }
    pub fn module(&mut self, name: &str, declaration: String) {
        let key = format!("module:{name}");
        self.modules.insert(name.into(), key.clone());
        let locals = declaration
            .lines()
            .filter_map(|line| {
                let line = line.trim_start();
                let declaration = line
                    .strip_prefix("type ")
                    .or_else(|| line.strip_prefix("and "))?;
                declaration
                    .split('=')
                    .next()?
                    .split('[')
                    .next()?
                    .split_whitespace()
                    .last()
                    .map(str::to_owned)
            })
            .collect();
        self.locals.insert(key.clone(), locals);
        self.items.insert(key, declaration);
    }
    pub fn finish(self) -> Result<String, Error> {
        let mut graph = BTreeMap::new();
        for (key, declaration) in &self.items {
            let text = documentation::without_strings(declaration);
            let mut dependencies = BTreeSet::new();
            let bytes = text.as_bytes();
            let mut at = 0;
            while at < bytes.len() {
                if !bytes[at].is_ascii_alphanumeric() && bytes[at] != b'_' {
                    at += 1;
                    continue;
                }
                let start = at;
                while at < bytes.len() && (bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_') {
                    at += 1;
                }
                let token = &text[start..at];
                if at < bytes.len() && bytes[at] == b'.' && (start == 0 || bytes[start - 1] != b'.')
                {
                    if let Some(dependency) = self.modules.get(token) {
                        dependencies.insert(dependency.clone());
                    }
                } else if (start == 0 || !matches!(bytes[start - 1], b'.' | b'\''))
                    && !text[at..].trim_start().starts_with(':')
                    && text[..start].split_whitespace().next_back() != Some("type")
                    && !self
                        .locals
                        .get(key)
                        .is_some_and(|locals| locals.contains(token))
                    && let Some(dependency) = self.types.get(token)
                {
                    dependencies.insert(dependency.clone());
                }
            }
            dependencies.remove(key);
            graph.insert(key.clone(), dependencies);
        }
        let mut output = String::new();
        for component in collections::groups(&graph) {
            let recursive = component.len() > 1;
            if recursive && component.iter().any(|key| !key.starts_with("module:")) {
                return Err(format!(
                    "public type/module dependency cycle: {}",
                    component.join(", ")
                )
                .into());
            }
            for (index, key) in component.iter().enumerate() {
                let declaration = &self.items[key];
                if recursive {
                    output.push_str(if index == 0 { "module rec " } else { "and " });
                    output.push_str(declaration.strip_prefix("module ").unwrap());
                } else {
                    output.push_str(declaration);
                }
                output.push('\n');
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::Interface;

    #[test]
    fn map_signature_key_binder_does_not_depend_on_a_record_named_key() {
        let mut interface = Interface::default();
        interface.types(
            vec!["key".into()],
            "type key = { values : int String_map.t }".into(),
        );
        interface.module(
            "String_map",
            "module String_map : Map.S with type key = string".into(),
        );
        let signature = interface.finish().unwrap();
        assert!(
            signature.find("module String_map").unwrap() < signature.find("type key = {").unwrap()
        );
    }
}
