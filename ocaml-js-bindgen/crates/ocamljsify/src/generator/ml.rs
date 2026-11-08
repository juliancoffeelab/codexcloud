use super::*;
use std::{collections::BTreeSet, fmt::Write};
// Shared binary list readers use the same typed codecs and SCC ordering as
// ordinary readers. Free generic parameters retain the compact List.init path.
#[derive(Default)]
struct SequenceReaders {
    entries: BTreeMap<String, (String, Ty)>,
}
impl SequenceReaders {
    fn element(t: &Ty) -> Option<&Ty> {
        if t.kind == "named" && t.name == "LinkedList" {
            Some(&t.args[0])
        } else {
            None
        }
    }
    fn generic(t: &Ty, params: &[String]) -> bool {
        !t.qualified && params.contains(&t.name)
            || t.args.iter().any(|t| Self::generic(t, params))
            || t.inner.as_ref().is_some_and(|t| Self::generic(t, params))
    }
    fn name(&self, t: &Ty, params: &[String]) -> Option<&str> {
        if Self::generic(t, params) {
            return None;
        }
        self.entries
            .get(&collections::identity(t))
            .map(|(name, _)| name.as_str())
    }
    fn collect(&mut self, t: &Ty, params: &[String], numeric_only: bool) {
        if t.kind == "ref" {
            return;
        }
        if let Some(element) = Self::element(t)
            && !Self::generic(element, params)
            && (!numeric_only || bulk::numeric(element))
        {
            self.entries
                .entry(collections::identity(element))
                .or_insert_with(|| (String::new(), element.clone()));
        }
        for arg in &t.args {
            self.collect(arg, params, numeric_only);
        }
        if let Some(inner) = &t.inner {
            self.collect(inner, params, numeric_only);
        }
    }
    fn new(s: &Schema, o: &Options, reachable: &Reachability) -> Self {
        let mut readers = Self::default();
        // Numeric reverse-span loops are ordinary tail recursion on every
        // backend. Other shared list loops require tail-modulo-cons support.
        let numeric_only = matches!(o.backend, Backend::Rescript);
        for entry in s.entries.values() {
            if entry.is_type() && !entry.js_value && reachable.read.contains(&entry.name) {
                let params = entry
                    .params
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                if let Some(target) = &entry.ty {
                    readers.collect(target, &params, numeric_only);
                }
                for body in entry
                    .body
                    .iter()
                    .chain(entry.variants.iter().map(|v| &v.body))
                {
                    for field in &body.fields {
                        if !projection::marked(field) {
                            readers.collect(&field.ty, &params, numeric_only);
                        }
                    }
                }
            } else if entry.kind == "function" {
                if let Some(result) = native_result(s, entry) {
                    readers.collect(result, &[], numeric_only);
                }
            }
        }
        for (index, (name, _)) in readers.entries.values_mut().enumerate() {
            *name = format!("__ocamljs_read_list_{index}");
        }
        readers
    }
}
pub(super) struct Gen<'a> {
    o: &'a Options,
    s: &'a Schema,
    maps: &'a collections::Maps,
    sequences: &'a SequenceReaders,
    params: Vec<String>,
    scope: &'a str,
    carriers: &'a BTreeMap<String, String>,
}
impl Gen<'_> {
    pub(super) fn typ_js(&self, t: &Ty) -> String {
        projection::js_type(self.s, self.o, self.maps, t)
    }
    fn field_type(&self, field: &Field) -> String {
        if projection::marked(field) {
            self.typ_js(&field.ty)
        } else {
            self.typ(&field.ty)
        }
    }
    fn enc_js(&self, t: &Ty, value: &str) -> String {
        let callbacks = projection::dependencies(self.s, t)
            .iter()
            .map(|native| format!(" (fun __js_value -> {})", self.enc(native, "__js_value")))
            .collect::<String>();
        format!(
            "{}w_js_{} w ({value}){callbacks}",
            self.scope,
            projection::key(t)
        )
    }
    fn dec_js(&self, t: &Ty) -> String {
        let callbacks = projection::dependencies(self.s, t)
            .iter()
            .map(|native| format!(" (fun r -> {})", self.dec(native)))
            .collect::<String>();
        format!("{}r_js_{} r{callbacks}", self.scope, projection::key(t))
    }
    fn enc_field(&self, field: &Field, value: &str) -> String {
        if projection::marked(field) {
            self.enc_js(&field.ty, value)
        } else {
            self.enc(&field.ty, value)
        }
    }
    fn dec_field(&self, field: &Field) -> String {
        if projection::marked(field) {
            self.dec_js(&field.ty)
        } else {
            self.dec(&field.ty)
        }
    }

    fn to_raw_js(&self, t: &Ty, value: &str) -> String {
        if projection::raw_target(self.s, t).is_some() {
            format!("{}raw_write_{} ({value})", self.scope, projection::key(t))
        } else if let Some(entry) = self.s.entries.get(&t.name).filter(|e| e.kind == "alias") {
            self.to_raw(entry.ty.as_ref().unwrap(), value)
        } else {
            self.to_raw(t, value)
        }
    }
    fn from_raw_js(&self, t: &Ty, value: &str) -> String {
        if projection::raw_target(self.s, t).is_some() {
            format!("{}raw_read_{} ({value})", self.scope, projection::key(t))
        } else if let Some(entry) = self.s.entries.get(&t.name).filter(|e| e.kind == "alias") {
            self.from_raw(entry.ty.as_ref().unwrap(), value)
        } else {
            self.from_raw(t, value)
        }
    }
    fn scalar_from_raw(&self, t: &Ty, value: &str, selected: bool) -> String {
        if selected && projection::raw_target(self.s, t).is_some() {
            return self.from_raw_js(t, value);
        }
        let mut current = t.clone();
        for _ in 0..128 {
            let Some(entry) = self
                .s
                .entries
                .get(&current.name)
                .filter(|e| e.kind == "alias")
            else {
                break;
            };
            if entry.js_value && projection::raw_target(self.s, &current).is_some() {
                return self.from_raw_js(&current, value);
            }
            let bindings = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .zip(current.args.iter().cloned())
                .collect();
            current = collections::substitute(entry.ty.as_ref().unwrap(), &bindings);
        }
        self.from_raw(&abi::resolve(self.s, t), value)
    }
    fn to_raw_field(&self, entry: &Entry, field: &Field, value: &str) -> String {
        if projection::marked(field)
            || entry.js_value
                && !self
                    .s
                    .entries
                    .get(&field.ty.name)
                    .is_some_and(|e| !e.js_value)
        {
            self.to_raw_js(&field.ty, value)
        } else {
            self.to_raw(&field.ty, value)
        }
    }
    fn from_raw_field(&self, entry: &Entry, field: &Field, value: &str) -> String {
        if projection::marked(field)
            || entry.js_value
                && !self
                    .s
                    .entries
                    .get(&field.ty.name)
                    .is_some_and(|e| !e.js_value)
        {
            self.from_raw_js(&field.ty, value)
        } else {
            self.from_raw(&field.ty, value)
        }
    }
    fn flat_array_type(&self) -> &str {
        if matches!(self.o.backend, Backend::JsOfOcaml) {
            "float Js_of_ocaml.Js.js_array Js_of_ocaml.Js.t"
        } else {
            "float array"
        }
    }
    fn flat_at(&self, index: usize) -> String {
        if matches!(self.o.backend, Backend::JsOfOcaml) {
            format!("(Js_of_ocaml.Js.Unsafe.get values {index} : float)")
        } else {
            format!("values.({index})")
        }
    }
    pub(super) fn typ(&self, t: &Ty) -> String {
        if t.kind == "array" {
            return format!("({}) {}", self.typ(t.inner.as_ref().unwrap()), "array");
        }
        if let Some(input) = abi::text_input(self.s, t).filter(|i| i.borrowed) {
            return if input.js {
                self.typ(abi::resolve(self.s, t).inner.as_ref().unwrap())
            } else {
                "string".into()
            };
        }
        if let Some(input) = abi::numeric_input(self.s, t).filter(|i| i.borrowed) {
            if input.js {
                let resolved = abi::resolve(self.s, t);
                return self.typ(resolved.inner.as_ref().unwrap());
            }
            return if input.bytes {
                "bytes".into()
            } else {
                format!("({}) array", self.typ(&input.element))
            };
        }
        if t.kind == "slice" {
            let element = abi::resolve(self.s, t.inner.as_ref().unwrap());
            return if element.name == "u8" {
                "bytes".into()
            } else {
                format!("({}) array", self.typ(&element))
            };
        }
        if t.kind == "ref" {
            let i = t.inner.as_ref().unwrap();
            return if i.name == "str" {
                "string".into()
            } else if i.kind == "slice" {
                "bytes".into()
            } else {
                self.typ(i)
            };
        }
        if t.kind == "tuple" {
            return match t.args.len() {
                0 => "unit".into(),
                1 => self.typ(&t.args[0]),
                _ => format!(
                    "({})",
                    t.args
                        .iter()
                        .map(|a| self.typ(a))
                        .collect::<Vec<_>>()
                        .join(" * ")
                ),
            };
        }
        let args = t.args.iter().map(|a| self.typ(a)).collect::<Vec<_>>();
        match t.name.as_str() {
            "Promise" => format!(
                "({}) {}",
                args[0],
                if matches!(self.o.backend, Backend::JsOfOcaml) {
                    if self.o.uses_lwt() {
                        "Lwt.t"
                    } else {
                        "Js_of_ocaml.Promise.t"
                    }
                } else {
                    "Js.Promise.t"
                }
            ),
            "Callback" => format!("({} -> unit)", args[0]),
            "String" | "str" => "string".into(),
            "bool" => "bool".into(),
            "f32" | "f64" => "float".into(),
            "u8" | "i8" | "u16" | "i16" => "int".into(),
            "u32" | "usize" => "float".into(),
            "i32" | "isize" => "int32".into(),
            "i64" | "u64" => "int64".into(),
            "Vec" => format!("({}) {}", args[0], "array"),
            "LinkedList" => format!("({}) list", args[0]),
            "Option" => format!("({}) option", args[0]),
            "Result" => format!("({}, {}) result", args[0], args[1]),
            "BTreeMap" | "HashMap" if t.map == "dict" => format!("({}) Js.Dict.t", args[1]),
            "HashMap" => {
                format!("({}, {}) Hashtbl.t", args[0], args[1])
            }
            "BTreeMap" => format!("({}) {}.t", args[1], self.maps.name(&t.args[0])),
            "Rc" | "Box" | "Arc" => args[0].clone(),
            n => {
                let n = label(&snake(n));
                if args.is_empty() {
                    if self.params.contains(&t.name) && !t.qualified {
                        format!("'{}", n)
                    } else {
                        n
                    }
                } else if args.len() == 1 {
                    format!("({}) {n}", args[0])
                } else {
                    format!("({}) {n}", args.join(", "))
                }
            }
        }
    }
    fn primitive(&self, t: &Ty) -> Option<&str> {
        match t.name.as_str() {
            "String" | "str" => Some("string"),
            "bool" => Some("bool"),
            "usize" => Some("u32"),
            "isize" => Some("i32"),
            "u8" => Some("u8"),
            "i8" => Some("i8"),
            "u16" => Some("u16"),
            "i16" => Some("i16"),
            "u32" => Some("u32"),
            "i32" => Some("i32"),
            "u64" => Some("u64"),
            "i64" => Some("i64"),
            "f32" => Some("f32"),
            "f64" => Some("f64"),
            _ => None,
        }
    }
    fn raw_type(&self, t: &Ty) -> String {
        if abi::numeric_input(self.s, t).is_some() || abi::text_input(self.s, t).is_some() {
            return self.typ(t);
        }
        let resolved = references::resolved(self.s, t);
        if references::host(self.s, t) && !std::ptr::eq(resolved, t) {
            return self.raw_type(resolved);
        }
        if t.kind == "ref"
            && t.inner
                .as_ref()
                .is_some_and(|t| references::host(self.s, t))
        {
            return self.raw_type(t.inner.as_ref().unwrap());
        }
        if t.name == "Promise" && matches!(self.o.backend, Backend::JsOfOcaml) {
            format!("({}) js_promise", self.typ(&t.args[0]))
        } else if references::host(self.s, t) {
            self.typ(t)
        } else if t.unit() {
            "unit".into()
        } else if matches!(t.name.as_str(), "i64" | "u64") {
            "string".into()
        } else if t.scalar() {
            if t.name == "bool" {
                "bool".into()
            } else {
                "float".into()
            }
        } else if t.name == "String" {
            "string".into()
        } else if t.name == "Callback" {
            "(reader -> unit)".into()
        } else if t.kind == "ref" {
            self.typ(t)
        } else {
            "writer".into()
        }
    }
    fn to_raw(&self, t: &Ty, v: &str) -> String {
        let resolved = references::resolved(self.s, t);
        if references::host(self.s, t) && !std::ptr::eq(resolved, t) {
            return self.to_raw(resolved, v);
        }
        if t.kind == "ref"
            && t.inner
                .as_ref()
                .is_some_and(|t| references::host(self.s, t))
        {
            return self.to_raw(t.inner.as_ref().unwrap(), v);
        }
        match t.name.as_str() {
            "Promise" if matches!(self.o.backend, Backend::JsOfOcaml) => {
                format!("{}promise_to_raw ({v})", self.scope)
            }
            "Callback" => {
                let finish = if self.o.paranoid || abi::owns_result(self.s, &t.args[0]) {
                    "Internal.finish r; "
                } else {
                    ""
                };
                format!(
                    "fun r -> let value = {} in {finish}{v} value",
                    self.dec(&t.args[0])
                )
            }
            "i64" | "u64" => format!("Int64.to_string ({v})"),
            "i32" | "isize" => {
                format!("float_of_int (Int32.to_int ({v}))")
            }
            "u8" | "i8" | "u16" | "i16" => format!("float_of_int ({v})"),
            _ => v.into(),
        }
    }
    pub(super) fn from_raw(&self, t: &Ty, v: &str) -> String {
        let resolved = references::resolved(self.s, t);
        if references::host(self.s, t) && !std::ptr::eq(resolved, t) {
            return self.from_raw(resolved, v);
        }
        match t.name.as_str() {
            "Promise" if matches!(self.o.backend, Backend::JsOfOcaml) => {
                format!("{}promise_from_raw ({v})", self.scope)
            }
            "i64" | "u64" => format!("Int64.of_string ({v})"),
            "i32" | "isize" => format!("Int32.of_float ({v})"),
            "u8" | "i8" | "u16" | "i16" => format!("int_of_float ({v})"),
            _ => v.into(),
        }
    }
    fn enc(&self, t: &Ty, v: &str) -> String {
        let scope = self.scope;
        if t.kind == "array" {
            let module = "Array";
            if !self.o.paranoid {
                return format!(
                    "({module}.iter (fun value -> {}) {v})",
                    self.enc(t.inner.as_ref().unwrap(), "value")
                );
            }
            return format!(
                "(if {module}.length {v} <> {} then invalid_arg \"fixed array length\"; {scope}w_budget w (float_of_int ({module}.length {v})); {module}.iter (fun value -> {}) {v})",
                t.len.unwrap(),
                self.enc(t.inner.as_ref().unwrap(), "value")
            );
        }
        if t.unit() {
            return "()".into();
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.enc(&t.args[0], v);
            }
            let vs = (0..t.args.len())
                .map(|i| format!("t{i}"))
                .collect::<Vec<_>>();
            return format!(
                "(let ({}) = {v} in {})",
                vs.join(", "),
                t.args
                    .iter()
                    .zip(vs)
                    .map(|(t, v)| self.enc(t, &v))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        if let Some(p) = self.primitive(t) {
            return format!("{scope}w_{p} w ({})", self.to_raw(t, v));
        }
        let args = &t.args;
        match t.name.as_str() {
            "Vec" if bulk::numeric(&args[0]) => {
                let p = self.primitive(&args[0]).unwrap();
                let convert = self.to_raw(&args[0], "value");
                let source: String = v.into();
                let mapped = if convert == "value" {
                    source
                } else {
                    format!("Array.map (fun value -> {convert}) ({source})")
                };
                format!("{scope}w_array_{p} w ({mapped})")
            }
            "LinkedList" | "Vec" => {
                let module = if t.name == "LinkedList" {
                    "List"
                } else {
                    "Array"
                };
                format!(
                    "({scope}w_length w (float_of_int ({module}.length {v})); {module}.iter (fun value -> {}) {v})",
                    self.enc(&args[0], "value")
                )
            }
            "Option" => format!(
                "(match {v} with None -> {scope}w_u8 w 0. | Some value -> {scope}w_u8 w 1.; {})",
                self.enc(&args[0], "value")
            ),
            "Result" => format!(
                "(match {v} with Ok value -> {scope}w_u8 w 0.; {} | Error error -> {scope}w_u8 w 1.; {})",
                self.enc(&args[0], "value"),
                self.enc(&args[1], "error")
            ),
            "Box" | "Rc" | "Arc" => self.enc(&args[0], v),
            "BTreeMap" | "HashMap" if t.map == "dict" => {
                let sort = if t.name == "BTreeMap" && self.o.paranoid {
                    format!(
                        "Array.sort (fun a b -> {}) keys; ",
                        self.compare(&args[0], "a", "b")
                    )
                } else {
                    String::new()
                };
                format!(
                    "(let keys = Js.Dict.keys {v} in {sort}{scope}w_length w (float_of_int (Array.length keys)); Array.iter (fun key -> let value = Js.Dict.unsafeGet {v} key in {}; {}) keys)",
                    self.enc(&args[0], "key"),
                    self.enc(&args[1], "value")
                )
            }
            "HashMap" if !self.o.paranoid => format!(
                "({scope}w_length w (float_of_int (Hashtbl.length {v})); Hashtbl.iter (fun key value -> {}; {}) {v})",
                self.enc(&args[0], "key"),
                self.enc(&args[1], "value")
            ),
            "HashMap" => format!(
                "({scope}w_length w (float_of_int (Hashtbl.length {v})); let seen = Hashtbl.create (Hashtbl.length {v}) in Hashtbl.iter (fun key value -> if Hashtbl.mem seen key then invalid_arg \"duplicate map key\"; Hashtbl.add seen key (); {}; {}) {v})",
                self.enc(&args[0], "key"),
                self.enc(&args[1], "value")
            ),
            "BTreeMap" => {
                let map = self.maps.name(&args[0]);
                format!(
                    "({scope}w_length w (float_of_int ({map}.cardinal {v})); {map}.iter (fun key value -> {}; {}) {v})",
                    self.enc(&args[0], "key"),
                    self.enc(&args[1], "value")
                )
            }
            n => {
                let suffix = args
                    .iter()
                    .map(|a| format!(" (fun w value -> {})", self.enc(a, "value")))
                    .collect::<String>();
                if self.params.contains(&t.name) && !t.qualified {
                    format!("writer_of_{} w ({v})", label(&snake(n)))
                } else {
                    format!("{scope}write_{}{suffix} w ({v})", snake(n))
                }
            }
        }
    }
    fn sequence_read(&self, t: &Ty, length: &str) -> String {
        format!("Array.init {length} (fun _ -> {})", self.dec(t))
    }
    fn list_read(&self, t: &Ty, length: &str) -> String {
        if let Some(name) = self.sequences.name(t, &self.params) {
            return format!("{}{} r {length}", self.scope, name);
        }
        format!("List.init {length} (fun _ -> {})", self.dec(t))
    }
    fn dec(&self, t: &Ty) -> String {
        let scope = self.scope;
        if references::host(self.s, t) {
            return self.from_raw(t, &format!("{scope}r_host r"));
        }
        if t.kind == "array" {
            let value = self.sequence_read(t.inner.as_ref().unwrap(), &t.len.unwrap().to_string());
            return if self.o.paranoid {
                format!("({scope}r_budget r {}.; {value})", t.len.unwrap())
            } else {
                value
            };
        }
        if t.unit() {
            return "()".into();
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.dec(&t.args[0]);
            }
            let vs = (0..t.args.len())
                .map(|i| format!("t{i}"))
                .collect::<Vec<_>>();
            return format!(
                "({}({}))",
                self.read_bindings(&t.args.iter().collect::<Vec<_>>(), &vs),
                vs.join(", ")
            );
        }
        if let Some(p) = self.primitive(t) {
            return self.from_raw(t, &format!("{scope}r_{p} r"));
        }
        let args = &t.args;
        match t.name.as_str() {
            "Vec" if bulk::numeric(&args[0]) => {
                let p = self.primitive(&args[0]).unwrap();
                let convert = self.from_raw(&args[0], "value");
                let source = format!("{scope}r_array_{p} r");
                let value = if convert == "value" {
                    source
                } else {
                    format!("Array.map (fun value -> {convert}) ({source})")
                };
                value
            }
            "LinkedList" => format!(
                "(let length = int_of_float ({scope}r_length r) in {})",
                self.list_read(&args[0], "length")
            ),
            "Vec" => format!(
                "(let length = int_of_float ({scope}r_length r) in {})",
                self.sequence_read(&args[0], "length")
            ),
            "Option" => format!(
                "(match int_of_float ({scope}r_u8 r) with 0 -> None | 1 -> Some ({}) | _ -> invalid_arg \"invalid option tag\")",
                self.dec(&args[0])
            ),
            "Result" => format!(
                "(match int_of_float ({scope}r_u8 r) with 0 -> Ok ({}) | 1 -> Error ({}) | _ -> invalid_arg \"invalid result tag\")",
                self.dec(&args[0]),
                self.dec(&args[1])
            ),
            "Box" | "Rc" | "Arc" => self.dec(&args[0]),
            "BTreeMap" | "HashMap" if t.map == "dict" => {
                let order = if t.name == "BTreeMap" && self.o.paranoid {
                    format!(
                        "(match !previous with Some previous -> if ({}) >= 0 then invalid_arg \"map keys are not strictly increasing\" | None -> ()); previous := Some key; ",
                        self.compare(&args[0], "previous", "key")
                    )
                } else {
                    String::new()
                };
                let duplicate = if self.o.paranoid {
                    format!("if {scope}dict_has table key then invalid_arg \"duplicate map key\"; ")
                } else {
                    String::new()
                };
                format!(
                    "(let length = int_of_float ({scope}r_length r) in let table = {scope}dict_create Js.Nullable.null in let previous = ref None in for _ = 1 to length do let key = {} in {order}{duplicate} let value = {} in Js.Dict.set table key value done; ignore previous; table)",
                    self.dec(&args[0]),
                    self.dec(&args[1])
                )
            }
            "HashMap" if !self.o.paranoid => format!(
                "(let length = int_of_float ({scope}r_length r) in let table = Hashtbl.create length in for _ = 1 to length do let key = {} in let value = {} in Hashtbl.add table key value done; table)",
                self.dec(&args[0]),
                self.dec(&args[1])
            ),
            "HashMap" => format!(
                "(let length = int_of_float ({scope}r_length r) in let table = Hashtbl.create length in for _ = 1 to length do let key = {} in if Hashtbl.mem table key then invalid_arg \"duplicate map key\"; let value = {} in Hashtbl.add table key value done; table)",
                self.dec(&args[0]),
                self.dec(&args[1])
            ),
            "BTreeMap" => {
                let map = self.maps.name(&args[0]);
                let check = if !self.o.paranoid {
                    String::new()
                } else if t.name == "BTreeMap" {
                    format!(
                        "(match !previous with Some previous -> if ({}) >= 0 then invalid_arg \"map keys are not strictly increasing\" | None -> ()); previous := Some key; ",
                        self.compare(&args[0], "previous", "key")
                    )
                } else {
                    format!("if {map}.mem key !table then invalid_arg \"duplicate map key\"; ")
                };
                format!(
                    "(let length = int_of_float ({scope}r_length r) in let table = ref {map}.empty in let previous = ref None in for _ = 1 to length do let key = {} in {check}let value = {} in table := {map}.add key value !table done; ignore previous; !table)",
                    self.dec(&args[0]),
                    self.dec(&args[1])
                )
            }
            n => {
                let suffix = args
                    .iter()
                    .map(|a| format!(" (fun r -> {})", self.dec(a)))
                    .collect::<String>();
                if self.params.contains(&t.name) && !t.qualified {
                    format!("reader_of_{} r", label(&snake(n)))
                } else {
                    format!("{scope}read_{}{suffix} r", snake(n))
                }
            }
        }
    }
    fn compare(&self, t: &Ty, a: &str, b: &str) -> String {
        if self.s.entries.get(&t.name).is_some_and(|e| e.js_value) {
            return self.compare_js(t, a, b);
        }
        format!(
            "(let (key_left : {}) = {a} in let (key_right : {}) = {b} in {})",
            self.typ(t),
            self.typ(t),
            self.compare_value(t, "key_left", "key_right")
        )
    }
    fn compare_js(&self, t: &Ty, a: &str, b: &str) -> String {
        let ty = self.typ_js(t);
        let natives = projection::dependencies(self.s, t);
        let callback_types = natives
            .iter()
            .map(|native| {
                let ty = self.typ(native);
                format!("({ty} -> {ty} -> int)")
            })
            .collect::<Vec<_>>();
        let args = std::iter::once(ty.clone())
            .chain(std::iter::once(ty))
            .chain(callback_types)
            .collect::<Vec<_>>();
        let runtime = if matches!(self.o.backend, Backend::JsOfOcaml) {
            format!(
                "let runtime = Js_of_ocaml.Js.Unsafe.get Js_of_ocaml.Js.Unsafe.global {:?}\n",
                format!("Ocamljs_{}", self.o.name)
            )
        } else {
            String::new()
        };
        let function = external(
            self.o,
            "apply",
            &format!("compare_js_{}", projection::key(t)),
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
            "int",
        );
        let callbacks = natives
            .iter()
            .map(|native| {
                format!(
                    " (fun left right -> {})",
                    self.compare(native, "left", "right")
                )
            })
            .collect::<String>();
        format!(
            "(let module Compare = struct {runtime}{function} end in Compare.apply ({a}) ({b}){callbacks})"
        )
    }
    fn compare_value(&self, t: &Ty, a: &str, b: &str) -> String {
        let primitive_compare = if matches!(self.o.backend, Backend::Rescript) {
            "Pervasives.compare"
        } else {
            "Stdlib.compare"
        };
        fn chain(parts: Vec<String>) -> String {
            parts
                .into_iter()
                .rev()
                .reduce(|rest, part| {
                    format!("(let order = {part} in if order = 0 then {rest} else order)")
                })
                .unwrap_or_else(|| "0".into())
        }
        if t.unit() {
            return "0".into();
        }
        if t.kind == "array" || matches!(t.name.as_str(), "Vec" | "LinkedList") {
            let inner = if t.kind == "array" {
                t.inner.as_ref().unwrap()
            } else {
                &t.args[0]
            };
            if t.name == "LinkedList" {
                return format!(
                    "(let rec loop left right = match left, right with [], [] -> 0 | [], _ -> -1 | _, [] -> 1 | left :: rest_left, right :: rest_right -> let order = {} in if order = 0 then loop rest_left rest_right else order in loop ({a}) ({b}))",
                    self.compare(inner, "left", "right")
                );
            }
            return format!(
                "(let left = {a} in let right = {b} in let rec loop index = if index = Array.length left then (if index = Array.length right then 0 else -1) else if index = Array.length right then 1 else let order = {} in if order = 0 then loop (index + 1) else order in loop 0)",
                self.compare(inner, "left.(index)", "right.(index)")
            );
        }
        if t.kind == "tuple" {
            if t.args.len() == 1 {
                return self.compare(&t.args[0], a, b);
            }
            let left = (0..t.args.len())
                .map(|i| format!("ka{i}"))
                .collect::<Vec<_>>();
            let right = (0..t.args.len())
                .map(|i| format!("kb{i}"))
                .collect::<Vec<_>>();
            return format!(
                "(let ({}) = {a} in let ({}) = {b} in {})",
                left.join(", "),
                right.join(", "),
                chain(
                    t.args
                        .iter()
                        .enumerate()
                        .map(|(i, t)| self.compare(t, &left[i], &right[i]))
                        .collect()
                )
            );
        }
        match t.name.as_str() {
            // ReScript String.get uses codePointAt, while length/indexing use
            // UTF-16 units. A non-BMP scalar has already consumed both units.
            "String" if matches!(self.o.backend, Backend::Rescript) => format!(
                "(let next text index = let first = Char.code (String.get text index) in if first > 0xffff then (first, 2) else if first >= 0xd800 && first <= 0xdfff then invalid_arg \"unpaired surrogate\" else (first, 1) in let rec loop left right = if left = String.length {a} then (if right = String.length {b} then 0 else -1) else if right = String.length {b} then 1 else let x, dx = next {a} left in let y, dy = next {b} right in if x < y then -1 else if x > y then 1 else loop (left + dx) (right + dy) in loop 0 0)"
            ),
            "String" => format!("String.compare ({a}) ({b})"),
            "u64" => format!(
                "Int64.compare (Int64.logxor ({a}) Int64.min_int) (Int64.logxor ({b}) Int64.min_int)"
            ),
            "i64" => format!("Int64.compare ({a}) ({b})"),
            "Box" | "Rc" | "Arc" => self.compare(&t.args[0], a, b),
            "Option" => format!(
                "(match {a}, {b} with None, None -> 0 | None, Some _ -> -1 | Some _, None -> 1 | Some left, Some right -> {})",
                self.compare(&t.args[0], "left", "right")
            ),
            "Result" => format!(
                "(match {a}, {b} with Ok left, Ok right -> {} | Error left, Error right -> {} | Ok _, Error _ -> -1 | Error _, Ok _ -> 1)",
                self.compare(&t.args[0], "left", "right"),
                self.compare(&t.args[1], "left", "right")
            ),
            _ if t.scalar() => format!("{primitive_compare} ({a}) ({b})"),
            _ => {
                let entry = &self.s.entries[&t.name];
                if entry.kind == "enum" && entry.variants.is_empty() {
                    return format!(
                        "(ignore ({a}); ignore ({b}); invalid_arg \"uninhabited map key\")"
                    );
                }
                let bindings = entry
                    .params
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .zip(t.args.iter().cloned())
                    .collect();
                if let Some(target) = &entry.ty {
                    return self.compare(&collections::substitute(target, &bindings), a, b);
                }
                if let Some(body) = &entry.body {
                    if entry.transparent {
                        return self.compare(
                            &collections::substitute(&body.fields[0].ty, &bindings),
                            a,
                            b,
                        );
                    }
                    if body.fields.is_empty() {
                        return "0".into();
                    }
                    if body.style == "named" {
                        return chain(
                            body.fields
                                .iter()
                                .map(|f| {
                                    let ty = collections::substitute(&f.ty, &bindings);
                                    let left = format!("({a}).{}", label(&f.name));
                                    let right = format!("({b}).{}", label(&f.name));
                                    if projection::marked(f) {
                                        self.compare_js(&ty, &left, &right)
                                    } else {
                                        self.compare(&ty, &left, &right)
                                    }
                                })
                                .collect(),
                        );
                    }
                    let pattern = |side: &str| {
                        format!(
                            "{} ({})",
                            self.constructor(entry, "value"),
                            (0..body.fields.len())
                                .map(|i| format!("{side}{i}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };
                    return format!(
                        "(let {} = {a} in let {} = {b} in {})",
                        pattern("ka"),
                        pattern("kb"),
                        chain(
                            body.fields
                                .iter()
                                .enumerate()
                                .map(|(i, f)| self.compare(
                                    &collections::substitute(&f.ty, &bindings),
                                    &format!("ka{i}"),
                                    &format!("kb{i}")
                                ))
                                .collect()
                        )
                    );
                }
                let pattern = |v: &Variant, side: &str| {
                    let c = self.constructor(entry, &v.name);
                    if v.body.fields.is_empty() {
                        return c;
                    }
                    if v.body.style == "named" {
                        format!(
                            "{c} {{ {} }}",
                            v.body
                                .fields
                                .iter()
                                .enumerate()
                                .map(|(i, f)| format!("{} = {side}{i}", label(&f.name)))
                                .collect::<Vec<_>>()
                                .join("; ")
                        )
                    } else {
                        format!(
                            "{c} ({})",
                            (0..v.body.fields.len())
                                .map(|i| format!("{side}{i}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                };
                let arms = entry
                    .variants
                    .iter()
                    .map(|v| {
                        format!(
                            "| {}, {} -> {}",
                            pattern(v, "ka"),
                            pattern(v, "kb"),
                            chain(
                                v.body
                                    .fields
                                    .iter()
                                    .enumerate()
                                    .map(|(i, f)| self.compare(
                                        &collections::substitute(&f.ty, &bindings),
                                        &format!("ka{i}"),
                                        &format!("kb{i}")
                                    ))
                                    .collect()
                            )
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let tags = entry
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        format!(
                            "| {}{} -> {i}",
                            self.constructor(entry, &v.name),
                            if v.body.fields.is_empty() { "" } else { " _" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let ty = self.typ(t);
                if entry.params.is_empty()
                    && entry
                        .variants
                        .iter()
                        .all(|variant| variant.body.fields.is_empty())
                {
                    // Declaration-order ranks preserve Rust ordering without
                    // relying on compiler-specific variant representations.
                    return format!(
                        "(let tag (value : {ty}) = match value with {tags} in {primitive_compare} (tag {a}) (tag {b}))"
                    );
                }
                format!(
                    "(let tag (value : {ty}) = match value with {tags} in match ({a} : {ty}), ({b} : {ty}) with {arms} | _ -> {primitive_compare} (tag {a}) (tag {b}))"
                )
            }
        }
    }
    pub(super) fn constructor(&self, e: &Entry, v: &str) -> String {
        if e.kind == "enum" && !e.js_value {
            let mut name = snake(v);
            name.get_mut(0..1).unwrap().make_ascii_uppercase();
            return format!("{}.{name}", self.carriers[&e.name]);
        }
        let mut n = snake(if e.module.is_empty() {
            &e.name
        } else {
            &e.member
        });
        n.get_mut(0..1).unwrap().make_ascii_uppercase();
        format!("{n}_{}", snake(v))
    }
    fn record_type(&self, b: &Body) -> String {
        format!(
            "{{ {} }}",
            b.fields
                .iter()
                .map(|f| format!(
                    "{} : {}{}",
                    label(&f.name),
                    self.field_type(f),
                    documentation::attribute(&f.docs, false)
                ))
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
    fn body_type(&self, e: &Entry, b: &Body) -> String {
        if b.fields.is_empty() {
            return format!("= {}", self.constructor(e, "value"));
        }
        match b.style.as_str() {
            "unit" => format!("= {}", self.constructor(e, "value")),
            "named" => format!("= {}", self.record_type(b)),
            _ => format!(
                "= {} of {}",
                self.constructor(e, "value"),
                b.fields
                    .iter()
                    .map(|f| self.field_type(f))
                    .collect::<Vec<_>>()
                    .join(" * ")
            ),
        }
    }
    // Fixed-width reads keep compiler-native construction in ML. Spans never
    // cross a call that can enter Wasm (opaque claims and host acquisition).
    fn span_read(
        &self,
        t: &Ty,
        offset: usize,
        seen: &mut BTreeSet<String>,
    ) -> Option<(usize, String)> {
        let scope = self.scope;
        let get =
            |kind: &str| format!("{scope}r_get_{kind} span_data (span_start +. {offset}.) true");
        if references::host(self.s, t) {
            return None;
        }
        if t.unit() {
            return Some((0, "()".into()));
        }
        if let Some(kind) = self.primitive(t) {
            if matches!(kind, "i64" | "u64") {
                return None;
            }
            let (width, value) = match kind {
                "string" => (4, format!("{scope}r_pool_string r ({})", get("u32"))),
                "bool" if !self.o.paranoid => (1, format!("({} <> 0.)", get("u8"))),
                "bool" => (
                    1,
                    format!(
                        "(match int_of_float ({}) with 0 -> false | 1 -> true | _ -> invalid_arg \"invalid bool\")",
                        get("u8")
                    ),
                ),
                _ => {
                    let (_, width, _, _) = bulk::NUMBERS.iter().find(|(name, ..)| *name == kind)?;
                    (*width, self.from_raw(t, &get(kind)))
                }
            };
            return Some((width, value));
        }
        if t.kind != "named" || (!t.qualified && self.params.contains(&t.name)) {
            return None;
        }
        if matches!(t.name.as_str(), "Box" | "Rc" | "Arc") {
            return self.span_read(&t.args[0], offset, seen);
        }
        if !seen.insert(t.name.clone()) {
            return None;
        }
        let entry = self.s.entries.get(&t.name)?;
        if entry.js_value
            || entry
                .body
                .as_ref()
                .is_some_and(|body| body.fields.iter().any(projection::marked))
        {
            return None;
        }
        let bindings = entry
            .params
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .zip(t.args.iter().cloned())
            .collect();
        let value = if let Some(target) = &entry.ty {
            self.span_read(&collections::substitute(target, &bindings), offset, seen)
        } else if entry.transparent {
            self.span_read(
                &collections::substitute(&entry.body.as_ref()?.fields[0].ty, &bindings),
                offset,
                seen,
            )
        } else {
            None
        };
        seen.remove(&t.name);
        value.map(|(width, value)| {
            // Rust aliases delegate to the target's Encode implementation;
            // only a transparent struct adds its own nested boundary.
            if entry.kind == "alias" || !self.o.paranoid {
                return (width, value);
            }
            (
                width,
                format!("({scope}enter r; let value = {value} in {scope}leave r; value)"),
            )
        })
    }
    fn read_bindings(&self, types: &[&Ty], vars: &[String]) -> String {
        let mut out = String::new();
        let mut index = 0;
        while index < types.len() {
            let mut run = Vec::new();
            let mut width = 0;
            for ty in &types[index..] {
                let Some((size, value)) = self.span_read(ty, width, &mut BTreeSet::new()) else {
                    break;
                };
                width += size;
                run.push(value);
            }
            if run.len() >= 2 && width > 0 {
                write!(
                    out,
                    "let span_start = {}r_span r {width}. in let span_data = {}r_data r in ",
                    self.scope, self.scope
                )
                .unwrap();
                for (var, value) in vars[index..].iter().zip(&run) {
                    write!(out, "let {var} = {value} in ").unwrap();
                }
                index += run.len();
            } else {
                write!(out, "let {} = {} in ", vars[index], self.dec(types[index])).unwrap();
                index += 1;
            }
        }
        out
    }
    fn read_body(&self, b: &Body, prefix: &str) -> String {
        if b.fields.iter().any(projection::marked) {
            return self.read_body_with(b, prefix, |_, field| self.dec_field(field));
        }
        let vars = (0..b.fields.len())
            .map(|i| format!("span_field_{i}"))
            .collect::<Vec<_>>();
        let bindings =
            self.read_bindings(&b.fields.iter().map(|f| &f.ty).collect::<Vec<_>>(), &vars);
        let value = self.read_body_with(b, prefix, |i, _| vars[i].clone());
        format!("({bindings}{value})")
    }
    fn read_body_with(
        &self,
        b: &Body,
        prefix: &str,
        read: impl Fn(usize, &Field) -> String,
    ) -> String {
        if b.fields.is_empty() {
            return prefix.into();
        }
        let vars = (0..b.fields.len())
            .map(|i| format!("f{i}"))
            .collect::<Vec<_>>();
        let lets = b
            .fields
            .iter()
            .zip(&vars)
            .enumerate()
            .map(|(i, (f, v))| format!("let {v} = {} in ", read(i, f)))
            .collect::<String>();
        let value = match b.style.as_str() {
            "unit" => prefix.into(),
            "named" => format!(
                "{prefix} {{ {} }}",
                b.fields
                    .iter()
                    .zip(&vars)
                    .map(|(f, v)| format!("{} = {v}", label(&f.name)))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            _ => format!("{prefix} ({})", vars.join(", ")),
        };
        format!("({lets}{value})")
    }
    fn write_body(&self, b: &Body, prefix: &str) -> (String, String) {
        if b.fields.is_empty() {
            return (prefix.into(), "()".into());
        }
        let vars = (0..b.fields.len())
            .map(|i| format!("f{i}"))
            .collect::<Vec<_>>();
        let pat = match b.style.as_str() {
            "unit" => prefix.into(),
            "named" => format!(
                "{prefix} {{ {} }}",
                b.fields
                    .iter()
                    .zip(&vars)
                    .map(|(f, v)| format!("{} = {v}", label(&f.name)))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            _ => format!("{prefix} ({})", vars.join(", ")),
        };
        let body = b
            .fields
            .iter()
            .zip(&vars)
            .map(|(f, v)| self.enc_field(f, v))
            .collect::<Vec<_>>()
            .join("; ");
        (pat, if body.is_empty() { "()".into() } else { body })
    }
}
pub(super) fn type_name(
    schema: &Schema,
    options: &Options,
    maps: &collections::Maps,
    t: &Ty,
) -> String {
    Gen {
        s: schema,
        o: options,
        maps,
        sequences: &SequenceReaders::default(),
        params: schema
            .entries
            .values()
            .flat_map(|e| e.params.iter().filter_map(Value::as_str).map(str::to_owned))
            .collect(),
        scope: "",
        carriers: &BTreeMap::new(),
    }
    .typ(t)
}
fn native_bulk(s: &Schema, o: &Options, entry: &Entry) -> bool {
    (bulk::record(entry)
        || !o.paranoid && !matches!(o.backend, Backend::JsOfOcaml) && bulk::packet_record(s, entry))
        && !entry.js_value
        && !entry
            .body
            .as_ref()
            .unwrap()
            .fields
            .iter()
            .any(projection::marked)
}

fn ext(o: &Options, name: &str, args: &[&str], ret: &str) -> String {
    external(o, name, name, args, ret)
}
pub(super) fn external(o: &Options, binding: &str, name: &str, args: &[&str], ret: &str) -> String {
    let sig = args
        .iter()
        .chain(std::iter::once(&ret))
        .copied()
        .collect::<Vec<_>>()
        .join(" -> ");
    match o.backend {
        Backend::Melange => format!(
            "external {binding} : {sig} = {name:?} [@@mel.scope {:?}]\n",
            format!("Ocamljs_{}", o.name)
        ),
        Backend::Rescript => format!(
            "external {binding} : {sig} = {name:?} [@@bs.scope {:?}]\n",
            format!("Ocamljs_{}", o.name)
        ),
        Backend::JsOfOcaml => {
            let ps = (0..args.len()).map(|i| format!("x{i}")).collect::<Vec<_>>();
            let convert = |t: &str, v: &str| match t {
                "string" => format!("Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.bytestring {v})"),
                "bool" => format!("Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.bool {v})"),
                "float array" => format!("Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.array {v})"),
                "bytes" => format!(
                    "Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Typed_array.Bytes.to_uint8Array {v})"
                ),
                "(string -> unit)" | "(string -> bool)" => format!(
                    "Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.wrap_callback (fun x -> {v} (Js_of_ocaml.Js.to_bytestring x)))"
                ),
                t if t.starts_with('(') && t.contains("->") => {
                    format!("Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.wrap_callback {v})")
                }
                _ => format!("Js_of_ocaml.Js.Unsafe.inject {v}"),
            };
            let call = format!(
                "Js_of_ocaml.Js.Unsafe.fun_call __ffi_{binding} [|{}|]",
                args.iter()
                    .zip(&ps)
                    .filter(|(t, _)| **t != "unit")
                    .map(|(t, v)| convert(t, v))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            let result = match ret {
                "float array" => format!("Js_of_ocaml.Js.to_array ({call})"),
                "string" => format!("Js_of_ocaml.Js.to_bytestring ({call})"),
                "bool" => format!("Js_of_ocaml.Js.to_bool ({call})"),
                _ => call,
            };
            format!(
                "let __ffi_{binding} = Js_of_ocaml.Js.Unsafe.get runtime {name:?}\nlet {binding} {} : {ret} = {result}\n",
                ps.iter()
                    .zip(args)
                    .map(|(p, t)| format!("({p} : {t})"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
    }
}
fn writable(
    schema: &Schema,
    ty: &Ty,
    params: &[String],
    visiting: &mut std::collections::BTreeSet<String>,
) -> bool {
    if params.contains(&ty.name) && !ty.qualified {
        return true;
    }
    if ty.kind == "ref" || ty.name == "Callback" {
        return false;
    }
    if !ty
        .args
        .iter()
        .all(|arg| writable(schema, arg, params, visiting))
    {
        return false;
    }
    if let Some(inner) = &ty.inner {
        if !writable(schema, inner, params, visiting) {
            return false;
        }
    }
    let Some(entry) = schema.entries.get(&ty.name) else {
        return true;
    };
    if matches!(entry.kind.as_str(), "opaque" | "host") {
        return false;
    }
    if !visiting.insert(ty.name.clone()) {
        return true;
    }
    let parameters = entry
        .params
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let valid = entry
        .ty
        .iter()
        .all(|ty| writable(schema, ty, &parameters, visiting))
        && entry
            .body
            .iter()
            .chain(entry.variants.iter().map(|variant| &variant.body))
            .flat_map(|body| &body.fields)
            .all(|field| writable(schema, &field.ty, &parameters, visiting));
    visiting.remove(&ty.name);
    valid
}
// js_value replaces only the success decoder; native error payloads remain reachable.
fn native_result<'a>(_: &'a Schema, entry: &'a Entry) -> Option<&'a Ty> {
    entry.result.as_ref()
}
#[derive(Default)]
struct Reachability {
    read: std::collections::BTreeSet<String>,
    write: std::collections::BTreeSet<String>,
}
impl Reachability {
    fn mark(&mut self, s: &Schema, t: &Ty, write: bool, params: &[String]) {
        if params.contains(&t.name) && !t.qualified || t.kind == "ref" {
            return;
        }
        if t.name == "Callback" {
            self.mark(s, &t.args[0], false, params);
            return;
        }
        for arg in &t.args {
            self.mark(s, arg, write, params);
        }
        if let Some(inner) = &t.inner {
            self.mark(s, inner, write, params);
        }
        if let Some(entry) = s.entries.get(&t.name) {
            let selected = if write {
                &mut self.write
            } else {
                &mut self.read
            };
            if !selected.insert(entry.name.clone()) {
                return;
            }
            let params = entry
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>();
            for field in collections::fields(entry) {
                self.mark(s, field, write, &params);
            }
        }
    }
    fn new(s: &Schema) -> Result<Self, Error> {
        let mut used = Self::default();
        for entry in s.entries.values().filter(|e| e.kind == "function") {
            for param in &entry.params {
                let t: Ty = serde_json::from_value(param["ty"].clone())?;
                if param["mode"] != "move"
                    && abi::flat(s, &t).is_none()
                    && abi::numeric_input(s, &t).is_none()
                {
                    used.mark(s, &t, true, &[]);
                }
            }
            if let Some(result) = native_result(s, entry) {
                if !(references::host(s, result) && !entry.asynchronous)
                    && !abi::text_output(s, result)
                    && abi::opaque(s, result).is_none()
                    && (!abi::async_direct(s, entry) || abi::text_result(s, result).is_some())
                    && (entry.js_value
                        || entry.asynchronous
                        || abi::direct_result(s, result).is_none())
                {
                    used.mark(s, result, false, &[]);
                }
            }
        }
        Ok(used)
    }
}
fn codec_dependencies(
    s: &Schema,
    t: &Ty,
    params: &[String],
    write: bool,
    sequences: &SequenceReaders,
    out: &mut std::collections::BTreeSet<String>,
) {
    if params.contains(&t.name) && !t.qualified || t.kind == "ref" {
        return;
    }
    if !write
        && let Some(element) = SequenceReaders::element(t)
        && let Some(name) = sequences.name(element, params)
    {
        out.insert(name.to_owned());
        return;
    }
    if let Some(entry) = s.entries.get(&t.name) {
        out.insert(format!(
            "{}_{}",
            if write { "write" } else { "read" },
            snake(&entry.name)
        ));
    }
    for arg in &t.args {
        codec_dependencies(s, arg, params, write, sequences, out);
    }
    if let Some(inner) = &t.inner {
        codec_dependencies(s, inner, params, write, sequences, out);
    }
}
pub fn generate(s: &Schema, o: &Options) -> Result<(String, String, String), Error> {
    let layout = enum_modules::prepare(s)?;
    let s = &layout.schema;
    let mut public_scope = s.clone();
    public_scope.entries.retain(|name, entry| {
        !entry.is_type() || layout.automatic.contains(name) || layout.companions.contains(name)
    });
    let maps = collections::Maps::new(s, o)?;
    let mut type_graph = std::collections::BTreeMap::new();
    for e in s.entries.values().filter(|e| e.is_type()) {
        let params = e
            .params
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let mut dependencies = std::collections::BTreeSet::new();
        if let Some(target) = &e.ty {
            if e.js_value {
                collections::js_type_dependencies(s, target, &params, &mut dependencies);
            } else {
                collections::type_dependencies(s, &maps, target, &params, &mut dependencies);
            }
        }
        for body in e.body.iter().chain(e.variants.iter().map(|v| &v.body)) {
            for field in &body.fields {
                if e.js_value || projection::marked(field) {
                    collections::js_type_dependencies(s, &field.ty, &params, &mut dependencies);
                } else {
                    collections::type_dependencies(s, &maps, &field.ty, &params, &mut dependencies);
                }
            }
        }
        type_graph.insert(e.name.clone(), dependencies);
    }
    for (name, key) in maps.ordered.values() {
        let mut dependencies = std::collections::BTreeSet::new();
        collections::type_dependencies(s, &maps, key, &[], &mut dependencies);
        type_graph.insert(format!("@{name}"), dependencies);
    }
    let mut carriers = BTreeMap::new();
    for component in collections::groups(&type_graph) {
        if component.iter().any(|name| name.starts_with('@')) {
            continue;
        }
        let module = format!("Ocamljs_types_{}", snake(&component[0]));
        for name in component {
            carriers.insert(name, module.clone());
        }
    }
    let reachable = Reachability::new(s)?;
    let sequences = SequenceReaders::new(s, o, &reachable);
    let g = Gen {
        o,
        s,
        maps: &maps,
        sequences: &sequences,
        params: vec![],
        scope: "Internal.",
        carriers: &carriers,
    };
    let jsoo = matches!(o.backend, Backend::JsOfOcaml);
    let asynchronous = s
        .entries
        .values()
        .any(|entry| entry.kind == "function" && entry.asynchronous);
    let native_async = if o.uses_lwt() {
        "Lwt.t"
    } else if jsoo {
        "Js_of_ocaml.Promise.t"
    } else {
        "Js.Promise.t"
    };
    let header = "(* Generated by ocamljsify. *)\n[@@@warning \"-30-32-34-39\"]\n";
    let mut mli = String::from("(* Generated by ocamljsify. *)\n[@@@warning \"-30\"]\n");
    let mut helpers = String::from("type writer\ntype reader\ntype __ocamljs_fields\n");
    let mut types = String::new();
    let mut public_carriers = modules::public_carriers(s);
    for name in layout.automatic.iter().chain(&layout.companions) {
        public_carriers.remove(name);
    }
    let mut interface = enum_modules::Interface::default();
    let mut codecs = String::new();
    let mut public = String::new();
    let mut public_signatures = String::new();
    let mut modules = modules::Modules::default();
    let projection = projection::generate(s, o, &maps)?;
    if jsoo {
        helpers.push_str("type 'a js_promise\n");
        writeln!(
            helpers,
            "let runtime = Js_of_ocaml.Js.Unsafe.get Js_of_ocaml.Js.Unsafe.global {:?}",
            format!("Ocamljs_{}", o.name)
        )?;
    }
    helpers.push_str(&projection.helpers);
    if maps.dict {
        let attr = if matches!(o.backend, Backend::Melange) {
            "mel"
        } else {
            "bs"
        };
        writeln!(
            helpers,
            "external dict_create : unit Js.Nullable.t -> 'a Js.Dict.t = \"create\" [@@{attr}.scope \"Object\"]\nexternal dict_has : 'a Js.Dict.t -> string -> bool = \"hasOwn\" [@@{attr}.scope \"Object\"]"
        )?;
    }
    if asynchronous {
        helpers.push_str("type raw_task\ntype 'a ocamljs_task\n");
    }
    for (n, a, r) in [
        ("writer", vec!["unit"], "writer"),
        ("module_failed", vec!["unit"], "bool"),
        ("module_failure", vec!["unit"], "string"),
        ("finish", vec!["reader"], "unit"),
        ("close", vec!["reader"], "unit"),
        ("enter", vec!["reader"], "unit"),
        ("leave", vec!["reader"], "unit"),
        ("r_length", vec!["reader"], "float"),
        ("w_length", vec!["writer", "float"], "unit"),
        ("w_budget", vec!["writer", "float"], "unit"),
        ("r_budget", vec!["reader", "float"], "unit"),
    ] {
        helpers.push_str(&ext(o, n, &a, r));
    }
    if asynchronous {
        helpers.push_str(&ext(
            o,
            "task_map",
            &["raw_task", "('raw -> 'a)"],
            "'a ocamljs_task",
        ));
        if jsoo {
            let mut receiver_args = vec!["'a ocamljs_task", "('a -> bool)"];
            if o.uses_lwt() {
                receiver_args.push("(unit -> bool)");
            }
            receiver_args.extend(["(exn -> bool)", "(string -> bool)"]);
            helpers.push_str(&ext(o, "task_receive", &receiver_args, "unit"));
        }
    }
    let (load_helpers, mut wasm_signature, mut wasm) = initialization::generate(o);
    helpers.push_str(&load_helpers);
    if asynchronous && jsoo {
        if o.uses_lwt() {
            helpers.push_str(&ext(o, "task_cancel_safe", &["'a ocamljs_task"], "unit"));

            helpers.push_str(
                r###"let await task =
  let promise, resolver = Lwt.task () in
  let deliver action =
    if Lwt.is_sleeping promise then (
      try action (); true with error ->
        if Lwt.is_sleeping promise then Lwt.wakeup_later_exn resolver error;
        false)
    else false
  in
  task_receive task
    (fun value -> deliver (fun () -> Lwt.wakeup_later resolver value))
    (fun () -> deliver (fun () -> Lwt.wakeup_later_exn resolver Lwt.Canceled))
    (fun error -> deliver (fun () -> Lwt.wakeup_later_exn resolver error))
    (fun reason -> deliver (fun () -> Lwt.wakeup_later_exn resolver (Failure reason)));
  Lwt.on_cancel promise (fun () -> task_cancel_safe task);
  promise
"###,
            );
        } else {
            helpers.push_str(
            r###"let await task =
  Js_of_ocaml.Promise.make (fun ~resolve ~reject ->
    let deliver action =
      try action (); true with error ->
        reject (Js_of_ocaml.Promise.error_of_exn error); false
    in
    task_receive task
      (fun value -> deliver (fun () -> resolve value))
      (fun error -> deliver (fun () -> reject (Js_of_ocaml.Promise.error_of_exn error)))
      (fun reason -> deliver (fun () -> reject (Js_of_ocaml.Promise.error_of_exn (Failure reason)))))
"###,
        );
        }
    } else if asynchronous {
        helpers.push_str(&external(
            o,
            "await",
            "task_await",
            &["'a ocamljs_task"],
            "'a Js.Promise.t",
        ));
    }
    for p in [
        "u8", "i8", "u16", "i16", "u32", "i32", "i64", "u64", "f32", "f64", "bool", "string",
    ] {
        let t = match p {
            "i64" | "u64" | "string" => "string",
            "bool" => "bool",
            _ => "float",
        };
        helpers.push_str(&ext(o, &format!("r_{p}"), &["reader"], t));
        helpers.push_str(&ext(o, &format!("w_{p}"), &["writer", t], "unit"));
    }
    helpers.push_str("type __ocamljs_data\n");
    helpers.push_str(&ext(o, "r_span", &["reader", "float"], "float"));
    helpers.push_str(&ext(o, "r_pool_string", &["reader", "float"], "string"));
    if jsoo {
        helpers.push_str(
            "let r_data (r:reader) : __ocamljs_data = Js_of_ocaml.Js.Unsafe.get r \"data\"\n",
        );
    } else {
        let attr = if matches!(o.backend, Backend::Melange) {
            "mel"
        } else {
            "bs"
        };
        writeln!(
            helpers,
            "external r_data : reader -> __ocamljs_data = \"data\" [@@{attr}.get]"
        )?;
    }
    for (kind, _, getter, _) in bulk::NUMBERS {
        if jsoo {
            writeln!(
                helpers,
                "let r_get_{kind} (data:__ocamljs_data) (at:float) (little:bool) : float = Js_of_ocaml.Js.Unsafe.meth_call data {getter:?} [|Js_of_ocaml.Js.Unsafe.inject at; Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.bool little)|]"
            )?;
        } else {
            let attr = if matches!(o.backend, Backend::Melange) {
                "mel"
            } else {
                "bs"
            };
            writeln!(
                helpers,
                "external r_get_{kind} : __ocamljs_data -> float -> bool -> float = {getter:?} [@@{attr}.send]"
            )?;
        }
    }
    for (field, ty) in [("float", "float"), ("bool", "bool"), ("string", "string")] {
        if jsoo {
            let value = match field {
                "string" => "Js_of_ocaml.Js.to_bytestring (Js_of_ocaml.Js.Unsafe.get values index)",
                "bool" => "Js_of_ocaml.Js.to_bool (Js_of_ocaml.Js.Unsafe.get values index)",
                _ => "Js_of_ocaml.Js.Unsafe.get values index",
            };
            writeln!(
                helpers,
                "let r_field_{field} (values:__ocamljs_fields) (index:int) : {ty} = {value}"
            )?;
        } else {
            let attr = if matches!(o.backend, Backend::Melange) {
                "mel"
            } else {
                "bs"
            };
            writeln!(
                helpers,
                "external r_field_{field} : __ocamljs_fields -> int -> {ty} = \"\" [@@{attr}.get_index]"
            )?;
        }
    }
    for (p, ..) in bulk::NUMBERS {
        helpers.push_str(&ext(o, &format!("r_array_{p}"), &["reader"], "float array"));
        helpers.push_str(&ext(
            o,
            &format!("w_array_{p}"),
            &["writer", "float array"],
            "unit",
        ));
    }
    for e in s.entries.values().filter(|e| native_bulk(s, o, e)) {
        let b = e.body.as_ref().unwrap();
        let mut args = vec!["writer".to_string()];
        args.extend(b.fields.iter().map(|f| {
            if bulk::record(e) {
                g.raw_type(&f.ty)
            } else {
                g.typ(&f.ty)
            }
        }));
        helpers.push_str(&ext(
            o,
            &format!("w_record_{}", snake(&e.name)),
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
            "unit",
        ));
    }
    for component in collections::groups(&type_graph) {
        if component.iter().any(|name| name.starts_with('@')) {
            if component.len() != 1 {
                return Err("recursive map-key/type module dependency is unsupported".into());
            }
            let name = component[0].trim_start_matches('@');
            let key = &maps.ordered.values().find(|(n, _)| n == name).unwrap().1;
            let key_type = g.typ(key);
            let compare = g.compare(key, "left", "right");
            writeln!(
                types,
                "module {name} = Map.Make(struct\n  type t = {key_type}\n  let compare (left : t) (right : t) = {compare}\nend)"
            )?;
            interface.module(
                name,
                format!(
                    "module {name} : Map.S with type key = {}",
                    modules::scoped_type(&public_scope, "", &key_type)
                ),
            );
            continue;
        }
        let carrier = carriers[&component[0]].clone();
        writeln!(types, "module {carrier} = struct")?;
        let mut private_types = String::new();
        let mut aliases = String::new();
        let mut declaration = String::new();
        let mut names = Vec::new();
        let mut first = true;
        let mut first_public = true;
        for name in component {
            let e = &s.entries[&name];
            let g = Gen {
                o,
                s,
                maps: &maps,
                sequences: &sequences,
                scope: "",
                carriers: &carriers,
                params: e
                    .params
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            };
            let name = label(&snake(&e.name));
            let params = e
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(|s| format!("'{}", label(&snake(s))))
                .collect::<Vec<_>>();
            let prefix = if params.len() == 1 {
                format!("{} ", params[0])
            } else if params.is_empty() {
                String::new()
            } else {
                format!("({}) ", params.join(", "))
            };
            let def = if e.js_value {
                format!("= {}", projection::definition(s, o, &maps, e))
            } else {
                match e.kind.as_str() {
                    "opaque" | "host" => String::new(),
                    "struct" => {
                        if e.transparent {
                            format!("= {}", g.field_type(&e.body.as_ref().unwrap().fields[0]))
                        } else {
                            g.body_type(e, e.body.as_ref().unwrap())
                        }
                    }
                    "enum" if e.variants.is_empty() => String::new(),
                    "enum" => format!(
                        "=\n{}",
                        e.variants
                            .iter()
                            .map(|v| {
                                let qualified = g.constructor(e, &v.name);
                                let c = qualified.rsplit('.').next().unwrap();
                                if v.body.fields.is_empty() {
                                    return format!(
                                        "| {c}{}",
                                        documentation::attribute(&v.docs, false)
                                    );
                                }
                                let mut declaration = match v.body.style.as_str() {
                                    "unit" => format!("| {c}"),
                                    "named" => format!("| {c} of {}", g.record_type(&v.body)),
                                    _ => format!(
                                        "| {c} of {}",
                                        v.body
                                            .fields
                                            .iter()
                                            .map(|f| g.field_type(f))
                                            .collect::<Vec<_>>()
                                            .join(" * ")
                                    ),
                                };
                                let mut docs = v.docs.clone();
                                if v.body.style == "tuple" {
                                    for f in &v.body.fields {
                                        if !f.docs.is_empty() {
                                            docs.push_str(&format!(
                                                "\n\nField {}: {}",
                                                f.name, f.docs
                                            ));
                                        }
                                    }
                                }
                                declaration.push_str(&documentation::attribute(&docs, false));
                                declaration
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    ),
                    "alias" => format!("= {}", g.typ(e.ty.as_ref().unwrap())),
                    _ => unreachable!(),
                }
            };
            let kw = if first { "type" } else { "and" };
            first = false;
            let separator = if def.is_empty() { "" } else { " " };
            let decl = format!("{kw} {prefix}{name}{separator}{def}\n");
            private_types.push_str(&decl);
            let carrier_type = format!("{prefix}{carrier}.{name}");
            let repeated = if e.kind == "struct" && !e.js_value && !e.transparent {
                format!(" {def}")
            } else {
                String::new()
            };
            writeln!(aliases, "{kw} {prefix}{name} = {carrier_type}{repeated}")?;
            modules.typ(s, e, &prefix, &def, &public_carriers, &carrier)?;
            if (e.module.is_empty() && !matches!(e.kind.as_str(), "opaque" | "host"))
                || public_carriers.contains(&e.name)
                || layout.companions.contains(&e.name)
            {
                let kw = if first_public { "type" } else { "and" };
                first_public = false;
                names.push(name.clone());
                let public_def = modules::scoped_type(&public_scope, "", &def);
                let public_def = if layout.companions.contains(&e.name) {
                    format!("= {prefix}{}.{name} {public_def}", e.module)
                } else {
                    public_def
                };
                writeln!(
                    declaration,
                    "{kw} {prefix}{name}{separator}{public_def}{}",
                    documentation::attribute(&documentation::type_docs(e), true)
                )?;
            }
        }
        types.push_str(&private_types);
        types.push_str("end\n");
        types.push_str(&aliases);
        interface.types(names, declaration);
    }

    for e in s.entries.values().filter(|e| e.kind == "opaque") {
        let name = snake(&e.name);
        if reachable.read.contains(&e.name) {
            codecs.push_str(&ext(
                o,
                &format!("r_handle_{name}"),
                &["reader"],
                &label(&name),
            ));
        }

        codecs.push_str(&external(
            o,
            &format!("dispose_{name}"),
            &format!("dispose_{}", e.name),
            &[&label(&name)],
            "unit",
        ));
    }
    if references::used(s) {
        helpers.push_str(&ext(o, "r_host", &["reader"], "'a"));
    }
    let mut functions = std::collections::BTreeMap::new();
    let mut codec_graph = std::collections::BTreeMap::new();
    for e in s.entries.values().filter(|e| e.is_type()) {
        let g = Gen {
            o,
            s,
            maps: &maps,
            sequences: &sequences,
            scope: "",
            carriers: &carriers,
            params: e
                .params
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
        };
        let name = snake(&e.name);
        let ps = e
            .params
            .iter()
            .filter_map(Value::as_str)
            .map(|s| label(&snake(s)))
            .collect::<Vec<_>>();
        let wargs = ps
            .iter()
            .map(|p| format!(" writer_of_{p}"))
            .collect::<String>();
        let rargs = ps
            .iter()
            .map(|p| format!(" reader_of_{p}"))
            .collect::<String>();
        let type_name = label(&name);
        let applied = if ps.is_empty() {
            type_name.clone()
        } else if ps.len() == 1 {
            format!("'{} {type_name}", ps[0])
        } else {
            format!(
                "({}) {type_name}",
                ps.iter()
                    .map(|p| format!("'{p}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let (enc, dec) = if e.js_value {
            let ty = projection::template(e);
            (g.enc_js(&ty, "value"), g.dec_js(&ty))
        } else {
            match e.kind.as_str() {
                "opaque" => (String::new(), format!("r_handle_{name} r")),
                "host" => (String::new(), "r_host r".into()),
                "alias" => (
                    g.enc(e.ty.as_ref().unwrap(), "value"),
                    g.dec(e.ty.as_ref().unwrap()),
                ),
                "struct" => {
                    let body = e.body.as_ref().unwrap();
                    if e.transparent {
                        (
                            g.enc_field(&body.fields[0], "value"),
                            g.dec_field(&body.fields[0]),
                        )
                    } else {
                        let prefix = if body.style == "named" && !body.fields.is_empty() {
                            String::new()
                        } else {
                            g.constructor(e, "value")
                        };
                        let (pattern, mut write) = g.write_body(body, &prefix);
                        if native_bulk(s, o, e) {
                            write = format!(
                                "w_record_{} w {}",
                                name,
                                body.fields
                                    .iter()
                                    .enumerate()
                                    .map(|(i, f)| format!(
                                        "({})",
                                        g.to_raw(&f.ty, &format!("f{i}"))
                                    ))
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            );
                        }
                        let hint = match ps.len() {
                            0 => type_name.clone(),
                            1 => format!("_ {type_name}"),
                            _ => format!("({}) {type_name}", vec!["_"; ps.len()].join(", ")),
                        };
                        (
                            format!("(let ({pattern} : {hint}) = value in {write})"),
                            format!("({} : {hint})", g.read_body(body, &prefix)),
                        )
                    }
                }
                "enum" if e.variants.is_empty() => (
                    "ignore value; invalid_arg \"uninhabited Rust enum\"".into(),
                    "invalid_arg \"uninhabited Rust enum\"".into(),
                ),
                "enum" => {
                    let writes = e
                        .variants
                        .iter()
                        .enumerate()
                        .map(|(i, variant)| {
                            let (pattern, write) =
                                g.write_body(&variant.body, &g.constructor(e, &variant.name));
                            format!("| {pattern} -> w_u32 w {i}.; {write}")
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    let reads = e
                        .variants
                        .iter()
                        .enumerate()
                        .map(|(i, variant)| {
                            format!(
                                "| {i} -> {}",
                                g.read_body(&variant.body, &g.constructor(e, &variant.name))
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    (
                        format!("(match value with\n{writes})"),
                        format!(
                            "(match int_of_float (r_u32 r) with\n{reads}\n| _ -> invalid_arg \"invalid variant tag\")"
                        ),
                    )
                }
                _ => unreachable!(),
            }
        };
        let poly = if ps.is_empty() {
            String::new()
        } else {
            format!(
                "{}. ",
                ps.iter()
                    .map(|p| format!("'{p}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let wsig = ps
            .iter()
            .map(|p| format!("(writer -> '{p} -> unit) -> "))
            .collect::<String>();
        let rsig = ps
            .iter()
            .map(|p| format!("(reader -> '{p}) -> "))
            .collect::<String>();
        if reachable.write.contains(&e.name)
            && writable(
                s,
                &Ty {
                    kind: "named".into(),
                    name: e.name.clone(),
                    qualified: true,
                    args: vec![],
                    inner: None,
                    len: None,
                    mutable: false,
                    map: String::new(),
                },
                &[],
                &mut std::collections::BTreeSet::new(),
            )
        {
            let function = format!("write_{name}");
            functions.insert(function.clone(),format!("{function} : {poly}{wsig}writer -> {applied} -> unit = fun{wargs} w value -> ignore w; {enc}"));
            let mut dependencies = std::collections::BTreeSet::new();
            for field in collections::fields(e) {
                codec_dependencies(s, field, &g.params, true, &sequences, &mut dependencies);
            }
            codec_graph.insert(function, dependencies);
        }
        if reachable.read.contains(&e.name) {
            let function = format!("read_{name}");
            let hint = match ps.len() {
                0 => type_name.clone(),
                1 => format!("_ {type_name}"),
                n => format!("({}) {type_name}", vec!["_"; n].join(", ")),
            };
            let body = if e.kind == "alias" || e.js_value || !o.paranoid {
                format!("let value : {hint} = {dec} in value")
            } else {
                format!("enter r; let value : {hint} = {dec} in leave r; value")
            };
            functions.insert(
                function.clone(),
                format!("{function} : {poly}{rsig}reader -> {applied} = fun{rargs} r -> ignore r; {body}"),
            );
            let mut dependencies = std::collections::BTreeSet::new();
            for field in collections::fields(e) {
                codec_dependencies(s, field, &g.params, false, &sequences, &mut dependencies);
            }
            codec_graph.insert(function, dependencies);
        }
    }

    let sequence_gen = Gen {
        o,
        s,
        maps: &maps,
        sequences: &sequences,
        params: vec![],
        scope: "",
        carriers: &carriers,
    };
    let sequence_names = sequences
        .entries
        .values()
        .filter(|(_, element)| !bulk::numeric(element))
        .map(|(name, _)| name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    for (name, element) in sequences.entries.values() {
        let element_type = sequence_gen.typ(element);
        if bulk::numeric(element) {
            let kind = sequence_gen.primitive(element).unwrap();
            let (_, width, _, _) = bulk::NUMBERS
                .iter()
                .find(|(name, ..)| *name == kind)
                .unwrap();
            let step = format!("{name}_values");
            let decoded =
                sequence_gen.from_raw(element, &format!("r_get_{kind} data position true"));
            // Reserve the whole span first. These numeric getters/conversions
            // accept every bit pattern and cannot reenter Wasm. Reading in
            // reverse is therefore unobservable; prepend yields wire-order
            // values with one cons each, without callbacks/temporary arrays.
            // Never extend this loop to validating or owning element codecs.
            functions.insert(step.clone(), format!("{step} : __ocamljs_data -> float -> int -> ({element_type}) list -> ({element_type}) list = fun data finish left values -> if left = 0 then values else let position = finish -. {width}. in let value = {decoded} in {step} data position (left - 1) (value :: values)"));
            functions.insert(name.clone(), format!("{name} : reader -> int -> ({element_type}) list = fun r length -> let bytes = float_of_int length *. {width}. in let start = r_span r bytes in let data = r_data r in {step} data (start +. bytes) length []"));
            codec_graph.insert(step.clone(), BTreeSet::from([step.clone()]));
            codec_graph.insert(name.clone(), BTreeSet::from([step]));
            continue;
        }
        let decode = sequence_gen.dec(element);
        functions.insert(name.clone(), format!("{name} : reader -> int -> ({element_type}) list = fun r left -> if left = 0 then [] else let value = {decode} in value :: {name} r (left - 1)"));
        let mut dependencies = std::collections::BTreeSet::from([name.clone()]);
        codec_dependencies(s, element, &[], false, &sequences, &mut dependencies);
        codec_graph.insert(name.clone(), dependencies);
    }
    for component in collections::groups(&codec_graph) {
        let recursive = component.len() > 1 || codec_graph[&component[0]].contains(&component[0]);
        for (index, name) in component.iter().enumerate() {
            let keyword = if index == 0 { "let" } else { "and" };
            let attribute = if sequence_names.contains(name) {
                "[@tail_mod_cons]"
            } else {
                ""
            };
            let recursive = if index == 0 && recursive { "rec " } else { "" };
            writeln!(
                codecs,
                "{keyword}{attribute} {recursive}{}",
                functions[name]
            )?;
        }
        codecs.push('\n');
    }
    if jsoo {
        let mut kinds = BTreeSet::new();
        for e in s.entries.values().filter(|e| e.kind == "function") {
            for p in &e.params {
                let t: Ty = serde_json::from_value(p["ty"].clone())?;
                if let Some(input) = abi::numeric_input(s, &t).filter(|i| !i.js) {
                    kinds.insert(input.kind);
                }
                if abi::text_input(s, &t).is_some_and(|i| !i.js) {
                    kinds.insert(0);
                }
            }
        }
        if !kinds.is_empty() {
            helpers.push_str("type numeric_view\n");
        }
        for kind in kinds {
            helpers.push_str(&ext(
                o,
                &format!("numeric_set_{kind}"),
                &["numeric_view", "float", "float"],
                "unit",
            ));
        }
    }
    let mut raw_externals = String::new();
    let mut exceptions = String::new();
    if jsoo && references::promises(s) {
        helpers.push_str(r#"let register_promise pending : 'a js_promise =
  Js_of_ocaml.Js.Unsafe.fun_call (Js_of_ocaml.Js.Unsafe.get runtime "promise_from_native")
    [|Js_of_ocaml.Promise.to_any pending;
      Js_of_ocaml.Js.Unsafe.inject (Js_of_ocaml.Js.wrap_callback (fun resolve reject ->
        ignore (Js_of_ocaml.Promise.then_
          ~on_error:(fun error -> ignore (Js_of_ocaml.Js.Unsafe.fun_call reject [|Js_of_ocaml.Promise.error_to_any error|]); Js_of_ocaml.Promise.resolve ())
          (fun value -> ignore (Js_of_ocaml.Js.Unsafe.fun_call resolve [|Js_of_ocaml.Js.Unsafe.inject value|]); Js_of_ocaml.Promise.resolve ()) pending)))|]
"#);
        if o.uses_lwt() {
            helpers.push_str("exception Promise_rejected of host_error\n");
            helpers.push_str(&ext(
                o,
                "on_promise",
                &["'a js_promise", "('a -> unit)", "(host_error -> unit)"],
                "unit",
            ));
            helpers.push_str(r#"let promise_from_raw pending =
  let promise, resolver = Lwt.task () in
  on_promise pending
    (fun value -> if Lwt.is_sleeping promise then Lwt.wakeup_later resolver value)
    (fun error -> if Lwt.is_sleeping promise then Lwt.wakeup_later_exn resolver (Promise_rejected error));
  promise
let promise_to_raw pending =
  register_promise (Js_of_ocaml.Promise.make (fun ~resolve ~reject ->
    Lwt.on_any pending
      (fun value -> try resolve value with error -> reject (Js_of_ocaml.Promise.error_of_exn error))
      (fun error -> match error with
        | Promise_rejected value -> reject (Js_of_ocaml.Promise.error_of_any (Js_of_ocaml.Js.Unsafe.inject value))
        | error -> reject (Js_of_ocaml.Promise.error_of_exn error))))
"#);
        } else {
            helpers.push_str("let promise_from_raw pending = Js_of_ocaml.Promise.of_any (Js_of_ocaml.Js.Unsafe.inject pending)\nlet promise_to_raw = register_promise\n");
        }
    }
    if s.entries.values().any(|e| {
        e.kind == "function"
            && !e.buffered
            && abi::text_result(s, e.result.as_ref().unwrap()).is_some()
    }) {
        helpers.push_str(&ext(o, "string_result_ok", &["reader"], "bool"));
    }
    for e in s.entries.values().filter(|e| e.kind == "function") {
        let params = e
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                Ok((
                    format!("__ocamljs_arg_{i}"),
                    serde_json::from_value::<Ty>(p["ty"].clone())?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let ret = e.result.as_ref().unwrap();
        let opaque_return = abi::opaque(s, ret).is_some();
        let text_return = abi::text_output(s, ret);
        let text_result = abi::text_result(s, ret).filter(|_| !e.buffered);
        let text_result_js = abi::text_result_js(s, ret);
        let text_getter = format!("string_result_value_{}", e.name);
        let text_wrap = format!("string_result_wrap_{}", e.name);
        let text_error_wrap = format!("string_result_wrap_error_{}", e.name);
        let text_error_getter = format!("string_result_error_{}", e.name);
        let throwing_exception = e.throws.then(|| {
            let mut name = format!("{}_error", e.name.trim_start_matches('_'));
            name.get_mut(0..1).unwrap().make_ascii_uppercase();
            name
        });
        if let Some(success) = &text_result {
            let raw = if e.js_value || text_result_js && success.name == "String" {
                g.typ_js(success)
            } else if abi::text_input(s, success).is_some_and(|t| t.js) {
                g.typ(success)
            } else {
                "string".into()
            };
            raw_externals.push_str(&external(
                o,
                &text_getter,
                "string_result_value",
                &["reader"],
                &raw,
            ));
            let result = abi::resolve(s, ret);
            let error = abi::resolve(s, &result.args[1]);
            if e.asynchronous && error.scalar() && !error.unit() {
                raw_externals.push_str(&external(
                    o,
                    &text_error_getter,
                    "string_result_error",
                    &["reader"],
                    &g.raw_type(&error),
                ));
            }
            if text_result_js && !e.throws {
                raw_externals.push_str(&external(
                    o,
                    &text_wrap,
                    "string_result_wrap",
                    &[&g.typ_js(success)],
                    &g.typ(ret),
                ));
                if e.asynchronous && error.scalar() {
                    raw_externals.push_str(&external(
                        o,
                        &text_error_wrap,
                        "string_result_wrap_error",
                        &[&g.typ_js(&result.args[1])],
                        &g.typ(ret),
                    ));
                }
            }
        }
        let rawret = if e.asynchronous {
            "raw_task".into()
        } else if text_return {
            if e.js_value {
                g.typ_js(ret)
            } else if abi::text_input(s, ret).is_some_and(|t| t.js) {
                g.typ(ret)
            } else {
                "string".into()
            }
        } else if e.asynchronous {
            "raw_task".into()
        } else if opaque_return {
            g.typ(ret)
        } else if abi::direct_result(s, ret).is_some() {
            g.flat_array_type().into()
        } else if ret.scalar() || references::host(s, ret) {
            g.raw_type(ret)
        } else {
            "reader".into()
        };
        let rawtypes = params
            .iter()
            .enumerate()
            .flat_map(|(i, (_, t))| {
                if jsoo
                    && (abi::numeric_input(s, t).is_some_and(|i| !i.js)
                        || abi::text_input(s, t).is_some_and(|i| !i.js))
                {
                    vec!["float".into(), "(numeric_view -> float -> unit)".into()]
                } else if e.params[i]["mode"] == "move" {
                    vec![g.typ(t)]
                } else {
                    abi::arguments(s, t)
                        .iter()
                        .map(|t| {
                            let raw = g.raw_type(t);
                            if raw == "writer" {
                                "(writer -> unit)".into()
                            } else {
                                raw
                            }
                        })
                        .collect()
                }
            })
            .collect::<Vec<_>>();
        let rawargs = if rawtypes.is_empty() {
            vec!["unit"]
        } else {
            rawtypes.iter().map(String::as_str).collect()
        };
        raw_externals.push_str(&ext(o, &format!("call_{}", e.name), &rawargs, &rawret));
        let args = params
            .iter()
            .map(|(n, t)| format!("({n} : {})", g.typ(t)))
            .collect::<Vec<_>>()
            .join(" ");
        let sig = params.iter().map(|(_, t)| g.typ(t)).collect::<Vec<_>>();
        let projected = e.js_value.then(|| {
            let success = if e.throws {
                &abi::resolve(s, ret).args[0]
            } else {
                projection::success(ret)
            };
            (g.typ_js(success), g.dec_js(success))
        });
        let projected = projected.as_ref();
        let resolved_return = abi::resolve(s, ret);
        let rettype = if let Some((ty, _)) = projected {
            if ret.name == "Result" && !e.throws {
                format!("({ty}, {}) result", g.typ(&resolved_return.args[1]))
            } else {
                ty.clone()
            }
        } else if e.throws {
            g.typ(&resolved_return.args[0])
        } else {
            g.typ(ret)
        };
        let mut types = sig;
        if params.is_empty() {
            types.push("unit".into());
        }
        types.push(if e.asynchronous {
            format!("({rettype}) {native_async}")
        } else {
            rettype
        });
        if e.module.is_empty() {
            writeln!(
                public_signatures,
                "val {} : {}{}",
                label(&e.name),
                modules::scoped_type(s, "", &types.join(" -> ")),
                documentation::attribute(
                    &documentation::function(e, o.backend, o.uses_lwt()),
                    true
                )
            )?;
        } else {
            modules.function(s, e, &types.join(" -> "), o.backend, o.uses_lwt())?;
        }
        let mut prep = String::new();
        let mut pass = Vec::new();
        for (i, (n, t)) in params.iter().enumerate() {
            if let Some(input) = abi::text_input(s, t) {
                if jsoo && !input.js {
                    pass.push(format!("(float_of_int (String.length {n}))"));
                    pass.push(format!("(fun __view __offset -> for __index=0 to String.length {n}-1 do Internal.numeric_set_0 __view (__offset +. float_of_int __index) (float_of_int (Char.code (String.unsafe_get {n} __index))) done)"));
                } else {
                    pass.push(format!("({n})"));
                }
            } else if let Some(input) = abi::numeric_input(s, t) {
                if jsoo && !input.js {
                    let length = if input.bytes {
                        format!("Bytes.length {n}")
                    } else {
                        format!("Array.length {n}")
                    };
                    let value = if input.bytes {
                        format!("Char.code (Bytes.unsafe_get {n} __index)")
                    } else {
                        format!("Array.unsafe_get {n} __index")
                    };
                    pass.push(format!("(float_of_int ({length}))"));
                    pass.push(format!("(fun __view __offset -> for __index = 0 to ({length}) - 1 do Internal.numeric_set_{} __view (__offset +. float_of_int __index) ({}) done)",input.kind,g.to_raw(&input.element,&value)));
                } else {
                    pass.push(n.clone());
                }
            } else if let Some(lanes) = abi::tuple(t) {
                fn pattern(t: &Ty, next: &mut usize, prefix: &str) -> String {
                    if t.kind == "tuple" {
                        let parts = t
                            .args
                            .iter()
                            .map(|t| pattern(t, next, prefix))
                            .collect::<Vec<_>>();
                        if parts.len() == 1 {
                            parts[0].clone()
                        } else {
                            format!("({})", parts.join(", "))
                        }
                    } else {
                        let n = format!("{prefix}{}", *next);
                        *next += 1;
                        n
                    }
                }
                let pattern = pattern(t, &mut 0, &format!("__ocamljs_tuple_{i}_"));
                write!(prep, "let {pattern} = {n} in ")?;
                pass.extend(lanes.iter().enumerate().map(|(j, t)| {
                    format!("({})", g.to_raw(t, &format!("__ocamljs_tuple_{i}_{j}")))
                }));
            } else if let Some(record) = abi::flat(s, t) {
                let b = record.body.as_ref().unwrap();
                let vars = (0..b.fields.len())
                    .map(|j| format!("__ocamljs_flat_{i}_{j}"))
                    .collect::<Vec<_>>();
                let pattern = if record.transparent {
                    vars[0].clone()
                } else if b.style == "named" {
                    format!(
                        "{{ {} }}",
                        b.fields
                            .iter()
                            .zip(&vars)
                            .map(|(f, v)| format!("{} = {v}", label(&f.name)))
                            .collect::<Vec<_>>()
                            .join("; ")
                    )
                } else {
                    format!("{} ({})", g.constructor(record, "value"), vars.join(", "))
                };
                if record.js_value {
                    for (j, var) in vars.iter().enumerate() {
                        write!(
                            prep,
                            "let {var} = Internal.flat_get_{}_{j} {n} in ",
                            label(&snake(&record.name))
                        )?;
                    }
                } else {
                    write!(prep, "let ({pattern} : {}) = {n} in ", g.typ(t))?;
                }
                pass.extend(
                    b.fields
                        .iter()
                        .zip(vars)
                        .map(|(f, v)| format!("({})", g.to_raw_field(record, f, &v))),
                );
            } else if !t.scalar()
                && t.kind != "ref"
                && t.name != "Callback"
                && t.name != "String"
                && !references::host(s, t)
                && e.params[i]["mode"] != "move"
            {
                let v = format!("__ocamljs_encoded_{i}");
                write!(prep, "let {v} w = {} in ", g.enc(t, n))?;
                pass.push(v);
            } else {
                pass.push(format!("({})", g.to_raw(t, n)));
            }
        }
        if pass.is_empty() {
            pass.push("()".into());
        }
        let call = format!("Internal.call_{} {}", e.name, pass.join(" "));
        let direct_result = abi::direct_result(s, ret);
        let completion_call = if e.asynchronous {
            "r".to_owned()
        } else {
            call.clone()
        };
        let scalar_return = ret.scalar() || e.asynchronous && abi::resolve(s, ret).scalar();
        let needs_finish = o.paranoid || abi::owns_result(s, ret);
        let protect = |decode: &str| {
            let finish = if needs_finish {
                "Internal.finish r; "
            } else {
                ""
            };
            if e.asynchronous {
                if needs_finish {
                    format!("let value = {decode} in {finish}value")
                } else {
                    decode.to_owned()
                }
            } else {
                format!(
                    "try let value = {decode} in {finish}Internal.close r; value with error -> Internal.close r; raise error"
                )
            }
        };
        let body = if e.asynchronous && scalar_return {
            let resolved = abi::resolve(s, ret);
            let raw = if resolved.unit() {
                "()"
            } else if jsoo && resolved.name == "bool" {
                "Js_of_ocaml.Js.to_bool r"
            } else if jsoo && matches!(resolved.name.as_str(), "i64" | "u64") {
                "Js_of_ocaml.Js.to_bytestring r"
            } else {
                "r"
            };
            g.scalar_from_raw(ret, raw, e.js_value)
        } else if opaque_return || text_return {
            if e.asynchronous
                && text_return
                && jsoo
                && !e.js_value
                && !abi::text_input(s, ret).is_some_and(|t| t.js)
            {
                "Js_of_ocaml.Js.to_bytestring r".into()
            } else {
                completion_call.clone()
            }
        } else if e.js_value && scalar_return && !e.asynchronous {
            g.from_raw_js(ret, &completion_call)
        } else if let Some(exception) = &throwing_exception {
            let success = projected
                .map(|(_, decode)| decode.clone())
                .unwrap_or_else(|| g.dec(&resolved_return.args[0]));
            let error = g.dec(&resolved_return.args[1]);
            // Decode/free under protection, then raise the typed error outside
            // that handler. Neither branch constructs an intermediate Result.
            let success = protect(&success);
            let error = protect(&error);
            format!(
                "let tag = (try int_of_float (Internal.r_u8 r) with error -> Internal.close r; raise error) in match tag with 0 -> ({success}) | 1 -> let error = ({error}) in raise ({exception} error) | _ -> Internal.close r; invalid_arg \"invalid result tag\""
            )
        } else if let Some((_, decode)) = projected.filter(|_| direct_result.is_none()) {
            let result = if ret.name == "Result" {
                format!(
                    "(match int_of_float (Internal.r_u8 r) with 0 -> Ok ({decode}) | 1 -> Error ({}) | _ -> invalid_arg \"invalid result tag\")",
                    g.dec(&resolved_return.args[1])
                )
            } else {
                decode.clone()
            };
            protect(&result)
        } else if let Some(record) = direct_result {
            let b = record.body.as_ref().unwrap();
            let values = b
                .fields
                .iter()
                .enumerate()
                .map(|(j, f)| g.from_raw_field(record, f, &g.flat_at(j)))
                .collect::<Vec<_>>();
            let value = if record.js_value {
                format!(
                    "Internal.flat_make_{} {}",
                    label(&snake(&record.name)),
                    values
                        .iter()
                        .map(|v| format!("({v})"))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            } else if record.transparent {
                values[0].clone()
            } else if b.style == "named" {
                format!(
                    "{{ {} }}",
                    b.fields
                        .iter()
                        .zip(&values)
                        .map(|(f, v)| format!("{} = {v}", label(&f.name)))
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            } else {
                format!("{} ({})", g.constructor(record, "value"), values.join(", "))
            };
            format!(
                "let values = {completion_call} in ({value} : {})",
                g.typ(ret)
            )
        } else if scalar_return || references::host(s, ret) {
            let resolved = abi::resolve(s, ret);
            if e.asynchronous && resolved.unit() {
                "()".into()
            } else if e.asynchronous && jsoo && resolved.name == "bool" {
                "Js_of_ocaml.Js.to_bool r".into()
            } else if e.asynchronous && jsoo && matches!(resolved.name.as_str(), "i64" | "u64") {
                "Int64.of_string (Js_of_ocaml.Js.to_bytestring r)".into()
            } else {
                g.from_raw(&resolved, &completion_call)
            }
        } else {
            protect(&g.dec(ret))
        };
        let body = if !o.paranoid && !abi::owns_result(s, ret) {
            body.replace("Internal.finish r; ", "")
        } else {
            body
        };
        let body = if e.asynchronous {
            body.replace("Internal.close r; ", "")
        } else {
            body
        };
        let body = if text_result.is_some() {
            let result = abi::resolve(s, ret);
            let scalar_error = abi::resolve(s, &result.args[1]);
            let direct_error = e.asynchronous && scalar_error.scalar();
            let error = g.dec(&result.args[1]);
            let tag = if o.paranoid {
                "if Internal.r_u8 r <> 1. then invalid_arg \"invalid string error tag\"; "
            } else {
                "ignore (Internal.r_u8 r); "
            };
            let error = if direct_error {
                let raw = if scalar_error.unit() {
                    "()".into()
                } else {
                    format!("Internal.{text_error_getter} r")
                };
                let error = g.scalar_from_raw(&result.args[1], &raw, text_result_js && !e.throws);
                if let Some(exception) = &throwing_exception {
                    format!("raise ({exception} ({error}))")
                } else if text_result_js {
                    format!("Internal.{text_error_wrap} ({error})")
                } else {
                    format!("Error ({error})")
                }
            } else if let Some(exception) = &throwing_exception {
                format!(
                    "let error = ({}) in raise ({exception} error)",
                    protect(&format!("{tag}{error}"))
                )
            } else {
                if text_result_js {
                    protect(&g.dec_js(ret))
                } else {
                    protect(&format!("{tag}Error ({error})"))
                }
            };
            let success = if e.throws {
                format!("Internal.{text_getter} r")
            } else if text_result_js {
                format!("Internal.{text_wrap} (Internal.{text_getter} r)")
            } else {
                format!("Ok (Internal.{text_getter} r)")
            };
            format!("if Internal.string_result_ok r then {success} else ({error})")
        } else {
            body
        };
        if let Some(exception) = &throwing_exception {
            writeln!(
                exceptions,
                "exception {exception} of {}",
                g.typ(&resolved_return.args[1])
            )?;
            if e.module.is_empty() {
                writeln!(
                    public_signatures,
                    "exception {exception} of {}{}",
                    modules::scoped_type(s, "", &g.typ(&resolved_return.args[1])),
                    documentation::attribute(
                        "Carries the Rust Err payload for this throwing function.",
                        true
                    )
                )?;
            } else {
                modules.exception(s, e, &exception, &g.typ(&resolved_return.args[1]))?;
            }
        }
        let body = if e.asynchronous {
            format!("Internal.task_map ({call}) (fun r -> {body})")
        } else if !scalar_return
            && !text_return
            && !opaque_return
            && !references::host(s, ret)
            && direct_result.is_none()
        {
            format!("let r = {call} in {body}")
        } else {
            body
        };
        let unit_arg = if params.is_empty() { "()" } else { "" };
        if e.asynchronous {
            let failed = if o.uses_lwt() {
                "Lwt.fail"
            } else if matches!(o.backend, Backend::JsOfOcaml) {
                "(fun error -> Js_of_ocaml.Promise.reject (Js_of_ocaml.Promise.error_of_exn error))"
            } else {
                "Js.Promise.reject"
            };
            writeln!(
                public,
                "let {} {args} {unit_arg} = try {prep}Internal.await ({body}) with error -> {failed} error",
                label(&e.name)
            )?;
        } else {
            writeln!(
                public,
                "let {} {args} {unit_arg} = {prep}{body}\n",
                label(&e.name)
            )?;
        }
    }
    if o.uses_lwt() && references::promises(s) {
        wasm = wasm.replacen(
            "module Wasm = struct\n",
            "module Wasm = struct\n  exception Promise_rejected = Internal.Promise_rejected\n",
            1,
        );
    }
    let module_implementations = modules.finish(s, &mut interface)?;
    mli.push_str(&interface.finish()?);
    if o.uses_lwt() && references::promises(s) {
        wasm_signature = wasm_signature.replacen(
            "module Wasm : sig\n",
            "module Wasm : sig\n  exception Promise_rejected of HostError.t\n",
            1,
        );
    }
    mli.push_str(&wasm_signature);
    mli.push_str(&public_signatures);
    let roots = format!("{codecs}{public}");
    let used = roots
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .collect::<std::collections::BTreeSet<_>>();
    let mut filtered = String::new();
    for line in helpers.lines() {
        let binding = line
            .strip_prefix("let __ffi_")
            .or_else(|| line.strip_prefix("let "))
            .or_else(|| line.strip_prefix("external "));
        if let Some(binding) = binding {
            let name = binding
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
                .unwrap();
            if (name.starts_with("r_")
                || name.starts_with("w_")
                || matches!(name, "writer" | "finish" | "close" | "enter" | "leave"))
                && !used.contains(name)
            {
                continue;
            }
        }
        filtered.push_str(line);
        filtered.push('\n');
    }
    let mut helpers = filtered;
    if !used.contains("writer") {
        helpers = helpers.replace("type writer\n", "");
    }
    for name in ["reader", "__ocamljs_fields", "__ocamljs_data"] {
        if format!("{helpers}{codecs}{raw_externals}")
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .filter(|token| *token == name)
            .count()
            == 1
        {
            helpers = helpers.replace(&format!("type {name}\n"), "");
        }
    }
    let mut ml = format!(
        "{header}{types}{exceptions}module Internal = struct\n{helpers}{codecs}{raw_externals}end\n{wasm}\n{public}{module_implementations}"
    );
    ml.truncate(ml.trim_end().len());
    ml.push('\n');
    mli.truncate(mli.trim_end().len());
    mli.push('\n');
    Ok((ml, mli, projection.javascript))
}
