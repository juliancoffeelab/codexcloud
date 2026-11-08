use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn substitute(t: &Ty, bindings: &BTreeMap<String, Ty>) -> Ty {
    if !t.qualified
        && let Some(value) = bindings.get(&t.name)
    {
        return value.clone();
    }
    let mut value = t.clone();
    value.args = value.args.iter().map(|t| substitute(t, bindings)).collect();
    value.inner = value
        .inner
        .as_ref()
        .map(|t| Box::new(substitute(t, bindings)));
    value
}
pub(super) fn fields(entry: &Entry) -> impl Iterator<Item = &Ty> {
    entry.ty.iter().chain(
        entry
            .body
            .iter()
            .chain(entry.variants.iter().map(|v| &v.body))
            .flat_map(|b| b.fields.iter().map(|f| &f.ty)),
    )
}
pub(super) fn key_check(
    s: &Schema,
    t: &Ty,
    ordered: bool,
    seen: &mut BTreeSet<String>,
) -> Result<(), Error> {
    if t.kind == "ref"
        || matches!(
            t.name.as_str(),
            "f32" | "f64" | "Callback" | "BTreeMap" | "HashMap"
        )
    {
        return Err(format!("unsupported map key {}; keys require stable owned structural equality/order without floats or maps",t.name).into());
    }
    if let Some(inner) = &t.inner {
        key_check(s, inner, ordered, seen)?;
    }
    for arg in &t.args {
        key_check(s, arg, ordered, seen)?;
    }
    if t.kind != "named"
        || t.scalar()
        || t.name == "String"
        || matches!(
            t.name.as_str(),
            "Vec" | "LinkedList" | "Option" | "Result" | "Box" | "Rc" | "Arc"
        )
    {
        return Ok(());
    }
    let entry = s.entries.get(&t.name).ok_or_else(|| {
        format!(
            "unsupported generic map key {}; use a concrete described key",
            t.name
        )
    })?;
    if matches!(entry.kind.as_str(), "opaque" | "host") {
        return Err(
            "opaque map keys cannot preserve Rust equality/order in detached native values".into(),
        );
    }
    if entry.kind != "alias" && !(entry.key_eq && (!ordered || entry.key_ord)) {
        return Err(format!("map key {} needs derived {} semantics or an explicit #[ocamljsify(key = \"structural\")] contract; custom Eq/Ord is unsupported",entry.name,if ordered {"Eq + Ord"}else{"Eq"}).into());
    }
    if !seen.insert(entry.name.clone()) {
        return Err("recursive map key descriptions are unsupported".into());
    }
    let bindings = entry
        .params
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .zip(t.args.iter().cloned())
        .collect();
    for field in fields(entry) {
        key_check(s, &substitute(field, &bindings), ordered, seen)?;
    }
    seen.remove(&entry.name);
    Ok(())
}
fn walk(t: &Ty, f: &mut impl FnMut(&Ty) -> Result<(), Error>) -> Result<(), Error> {
    f(t)?;
    for a in &t.args {
        walk(a, f)?;
    }
    if let Some(i) = &t.inner {
        walk(i, f)?;
    }
    Ok(())
}
fn native_each(s: &Schema, mut f: impl FnMut(&Ty) -> Result<(), Error>) -> Result<(), Error> {
    for entry in s.entries.values() {
        if entry.is_type() && !entry.js_value {
            if let Some(target) = &entry.ty {
                walk(target, &mut f)?;
            }
            for body in entry
                .body
                .iter()
                .chain(entry.variants.iter().map(|v| &v.body))
            {
                for field in &body.fields {
                    if !projection::marked(field) {
                        walk(&field.ty, &mut f)?;
                    }
                }
            }
        }
        if entry.kind == "function" {
            if let Some(ret) = &entry.result {
                if !entry.js_value {
                    walk(ret, &mut f)?;
                } else if ret.name == "Result" {
                    walk(&ret.args[1], &mut f)?;
                }
            }
            for param in &entry.params {
                walk(&serde_json::from_value::<Ty>(param["ty"].clone())?, &mut f)?;
            }
        }
    }
    Ok(())
}
pub(super) fn js_type_dependencies(
    s: &Schema,
    t: &Ty,
    params: &[String],
    out: &mut BTreeSet<String>,
) {
    if params.contains(&t.name) && !t.qualified {
        return;
    }
    if s.entries.get(&t.name).is_some_and(Entry::is_type) {
        out.insert(t.name.clone());
    }
    for arg in &t.args {
        js_type_dependencies(s, arg, params, out);
    }
    if let Some(inner) = &t.inner {
        js_type_dependencies(s, inner, params, out);
    }
}
pub(super) fn validate(s: &Schema) -> Result<(), Error> {
    fn abstract_key(t: &Ty, params: &[String]) -> bool {
        (!t.qualified && params.contains(&t.name))
            || t.args.iter().any(|t| abstract_key(t, params))
            || t.inner.as_ref().is_some_and(|t| abstract_key(t, params))
    }
    for entry in s.entries.values() {
        let params = if entry.kind == "function" {
            vec![]
        } else {
            entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        };
        let mut check = |t: &Ty| {
            if !t.map.is_empty()
                && (t.map != "dict"
                    || !matches!(t.name.as_str(), "HashMap" | "BTreeMap")
                    || t.args
                        .first()
                        .is_none_or(|k| k.kind != "named" || k.name != "String"))
            {
                return Err(
                    "dict map representation requires a String-keyed HashMap or BTreeMap".into(),
                );
            }
            if matches!(t.name.as_str(), "HashMap" | "BTreeMap") {
                if abstract_key(&t.args[0], &params) {
                    return Err(
                        "abstract generic map keys are unsupported; describe a concrete key type"
                            .into(),
                    );
                }
                key_check(s, &t.args[0], t.name == "BTreeMap", &mut BTreeSet::new())?;
            }
            Ok(())
        };
        for t in fields(entry).chain(entry.result.iter()) {
            walk(t, &mut check)?;
        }
        if entry.kind == "function" {
            for param in &entry.params {
                walk(
                    &serde_json::from_value::<Ty>(param["ty"].clone())?,
                    &mut check,
                )?;
            }
        }
    }
    Ok(())
}
pub(super) fn identity(t: &Ty) -> String {
    let mut t = t.clone();
    fn normalize(t: &mut Ty) {
        t.qualified = false;
        for a in &mut t.args {
            normalize(a);
        }
        if let Some(i) = &mut t.inner {
            normalize(i);
        }
    }
    normalize(&mut t);
    serde_json::to_string(&t).unwrap()
}
fn key_name(t: &Ty) -> String {
    if t.unit() {
        return "unit".into();
    }
    if t.kind == "tuple" {
        return format!(
            "tuple_{}",
            t.args.iter().map(key_name).collect::<Vec<_>>().join("_")
        );
    }
    if t.kind == "array" {
        return format!(
            "array_{}_{}",
            t.len.unwrap(),
            key_name(t.inner.as_ref().unwrap())
        );
    }
    format!(
        "{}{}",
        snake(&t.name),
        t.args
            .iter()
            .map(|a| format!("_{}", key_name(a)))
            .collect::<String>()
    )
}
#[derive(Default)]
pub(super) struct Maps {
    pub ordered: BTreeMap<String, (String, Ty)>,
    pub dict: bool,
}
impl Maps {
    pub fn new(s: &Schema, o: &Options) -> Result<Self, Error> {
        let mut maps = Self { ..Self::default() };
        let mut names = s
            .entries
            .values()
            .filter(|e| matches!(e.kind.as_str(), "opaque" | "host" | "module"))
            .map(|e| {
                if e.kind == "module" {
                    e.member.clone()
                } else {
                    e.name.clone()
                }
            })
            .collect::<BTreeSet<_>>();
        native_each(s, |t| {
            if t.map == "dict" {
                maps.dict = true;
                if matches!(o.backend, Backend::JsOfOcaml) {
                    return Err(
                        "map = \"dict\" requires the Melange or ReScript Js.Dict backend".into(),
                    );
                }
            }
            if t.name == "BTreeMap" && t.map != "dict" {
                let key = &t.args[0];
                let id = identity(key);
                if !maps.ordered.contains_key(&id) {
                    let mut name = key_name(key);
                    name.get_mut(..1).unwrap().make_ascii_uppercase();
                    name.push_str("_map");
                    if !names.insert(name.clone()) {
                        name.push_str(&format!("_{:x}", Sha256::digest(id.as_bytes()))[..9]);
                        let suffixed = name.clone();
                        let mut collision = 2;
                        while !names.insert(name.clone()) {
                            name = format!("{suffixed}_{collision}");
                            collision += 1;
                        }
                    }
                    maps.ordered.insert(id, (name, key.clone()));
                }
            }
            Ok(())
        })?;
        Ok(maps)
    }
    pub fn name(&self, key: &Ty) -> &str {
        &self.ordered[&identity(key)].0
    }
}
/// Dependency-first strongly connected groups, shared by type and codec declarations.
pub(super) fn groups(graph: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    fn visit(
        n: &str,
        g: &BTreeMap<String, BTreeSet<String>>,
        seen: &mut BTreeSet<String>,
        order: &mut Vec<String>,
    ) {
        if !seen.insert(n.into()) {
            return;
        }
        if let Some(deps) = g.get(n) {
            for d in deps {
                if g.contains_key(d) {
                    visit(d, g, seen, order);
                }
            }
        }
        order.push(n.into());
    }
    let mut order = vec![];
    let mut seen = BTreeSet::new();
    for n in graph.keys() {
        visit(n, graph, &mut seen, &mut order);
    }
    let mut reverse: BTreeMap<String, BTreeSet<String>> =
        graph.keys().map(|n| (n.clone(), BTreeSet::new())).collect();
    for (n, deps) in graph {
        for d in deps {
            if let Some(e) = reverse.get_mut(d) {
                e.insert(n.clone());
            }
        }
    }
    seen.clear();
    let mut result = vec![];
    for n in order.iter().rev() {
        if !seen.contains(n) {
            let mut component = vec![];
            visit(n, &reverse, &mut seen, &mut component);
            component.sort();
            result.push(component);
        }
    }
    result.reverse();
    result
}
pub(super) fn type_dependencies(
    s: &Schema,
    maps: &Maps,
    t: &Ty,
    params: &[String],
    out: &mut BTreeSet<String>,
) {
    if params.contains(&t.name) && !t.qualified {
        return;
    }
    if t.name == "BTreeMap" && t.map != "dict" {
        out.insert(format!("@{}", maps.name(&t.args[0])));
        type_dependencies(s, maps, &t.args[1], params, out);
        return;
    }
    if s.entries.get(&t.name).is_some_and(Entry::is_type) {
        out.insert(t.name.clone());
    }
    for a in &t.args {
        type_dependencies(s, maps, a, params, out);
    }
    if let Some(i) = &t.inner {
        type_dependencies(s, maps, i, params, out);
    }
}
