use super::*;
use syn::{parse::Parser, visit_mut::VisitMut};

pub(super) fn identifier_option(meta: &syn::Meta, key: &str) -> syn::Result<Option<syn::Ident>> {
    if let syn::Meta::NameValue(value) = meta {
        if value.path.is_ident(key) {
            if let syn::Expr::Path(path) = &value.value {
                if let Some(name) = path.path.get_ident() {
                    return Ok(Some(name.clone()));
                }
            }
            return Err(syn::Error::new_spanned(
                value,
                "expected an unqualified identifier",
            ));
        }
    }
    Ok(None)
}
pub(super) fn binding_attribute(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "ocamljs_bindgen" || s.ident == "ocamljsify")
}
fn attribute_options(attr: &Attribute) -> proc_macro2::TokenStream {
    match &attr.meta {
        syn::Meta::List(list) => list.tokens.clone(),
        _ => quote!(),
    }
}
/// The descriptor carries only names and type relationships; it adds no ABI calls.
#[derive(Default)]
pub(super) struct Scope {
    pub module: String,
    pub types: std::collections::BTreeMap<String, String>,
    pub t: Option<String>,
}
impl Scope {
    pub fn describe(&self, mut value: Value, declaration: bool) -> Value {
        fn walk(
            value: &mut Value,
            names: &std::collections::BTreeMap<String, String>,
            params: &[String],
        ) {
            if value["kind"] == "named" {
                if let Some(name) = value["name"].as_str() {
                    let path = value["path"].as_str().unwrap_or(name);
                    if (!value["qualified"].as_bool().unwrap_or(false)
                        && !params.iter().any(|p| p == name))
                        || path == format!("self::{name}")
                    {
                        if let Some(scoped) = names.get(name) {
                            value["name"] = json!(scoped);
                        }
                    }
                }
            }
            match value {
                Value::Object(fields) => {
                    for value in fields.values_mut() {
                        walk(value, names, params);
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        walk(value, names, params);
                    }
                }
                _ => {}
            }
        }
        let params = if declaration {
            value["params"]
                .as_array()
                .map(|p| {
                    p.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        walk(&mut value, &self.types, &params);
        if declaration {
            let rust_name = value["name"].as_str().unwrap().to_owned();
            value["name"] = json!(self.types[&rust_name]);
            value["module"] = json!(self.module);
            value["member"] = json!(rust_name);
        }
        value
    }
}
fn module_name(name: &syn::Ident) -> String {
    let mut uppercase = true;
    name.to_string()
        .trim_start_matches("r#")
        .chars()
        .filter_map(|c| {
            if c == '_' {
                uppercase = true;
                None
            } else if uppercase {
                uppercase = false;
                Some(c.to_ascii_uppercase())
            } else {
                Some(c)
            }
        })
        .collect()
}
pub(super) fn module(
    options: &str,
    mut module: syn::ItemMod,
) -> syn::Result<proc_macro2::TokenStream> {
    let options = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
        .parse_str(options)?;
    let mut t = None;
    let mut opaque_t = false;
    for option in options {
        let is_opaque = identifier_option(&option, "opaque_t")?;
        if let Some(value) = identifier_option(&option, "t")?.or_else(|| is_opaque.clone()) {
            if t.replace(value).is_some() {
                return Err(syn::Error::new_spanned(
                    option,
                    "select either t or opaque_t once",
                ));
            }
            opaque_t = is_opaque.is_some();
        } else {
            return Err(syn::Error::new_spanned(
                option,
                "module options: t = Type or opaque_t = Type (optional)",
            ));
        }
    }
    let name = module_name(&module.ident);
    let rust_path = module.ident.to_string().trim_start_matches("r#").to_owned();
    let module_docs = docs(&module.attrs);
    let (_, items) = module
        .content
        .as_mut()
        .ok_or_else(|| syn::Error::new_spanned(&module.ident, "binding modules must be inline"))?;
    if opaque_t {
        let selected = t.as_ref().unwrap();
        let item = items
            .iter_mut()
            .find(|item| matches!(item, Item::Struct(s) if s.ident == *selected))
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    selected,
                    "opaque_t must select a struct declared in this module",
                )
            })?;
        let Item::Struct(item) = item else {
            unreachable!()
        };
        if !item.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                selected,
                "opaque types must be concrete",
            ));
        }
        let attributes = item
            .attrs
            .iter()
            .filter(|a| binding_attribute(a))
            .collect::<Vec<_>>();
        if attributes.len() > 1 {
            return Err(syn::Error::new_spanned(
                selected,
                "duplicate export attributes",
            ));
        }
        if let Some(attr) = attributes.first() {
            let options = attribute_options(attr);
            if !options.is_empty() && options.to_string().trim() != "opaque" {
                return Err(syn::Error::new_spanned(
                    attr,
                    "opaque_t conflicts with type representation options",
                ));
            }
        }
        item.attrs.retain(|a| !binding_attribute(a));
        item.attrs
            .push(syn::parse_quote!(#[::ocamljs_bindgen::ocamljs_bindgen(opaque)]));
    }
    let mut scope = Scope {
        module: name.clone(),
        t: t.as_ref().map(ToString::to_string),
        ..Scope::default()
    };
    for item in items.iter() {
        let described = match item {
            Item::Struct(s) => Some((&s.ident, &s.attrs)),
            Item::Enum(e) => Some((&e.ident, &e.attrs)),
            Item::Type(t) => Some((&t.ident, &t.attrs)),
            _ => None,
        };
        if let Some((ident, attrs)) = described {
            if attrs.iter().any(binding_attribute) {
                let local = ident.to_string();
                // Keep existing resource ABI labels stable. Native module types are
                // namespaced, so unrelated modules may both declare Point or T.
                let opaque = attrs.iter().filter(|a| binding_attribute(a)).any(|a| {
                    attribute_options(a)
                        .to_string()
                        .split(',')
                        .any(|o| o.trim() == "opaque")
                });
                let canonical = if opaque {
                    local.clone()
                } else {
                    format!("{name}__{local}")
                };
                scope.types.insert(local, canonical);
            }
        }
    }
    if let Some(t) = &scope.t {
        if !scope.types.contains_key(t) {
            return Err(syn::Error::new_spanned(
                &module.ident,
                "t must name an annotated type declared in this module",
            ));
        }
    }
    for item in items.iter_mut() {
        let attrs = match item {
            Item::Struct(s) => Some(&mut s.attrs),
            Item::Enum(e) => Some(&mut e.attrs),
            Item::Type(t) => Some(&mut t.attrs),
            Item::Fn(f) => Some(&mut f.attrs),
            _ => None,
        };
        let Some(attrs) = attrs else {
            continue;
        };
        let exports = attrs
            .iter()
            .filter(|a| binding_attribute(a))
            .cloned()
            .collect::<Vec<_>>();
        if exports.len() > 1 {
            return Err(syn::Error::new_spanned(item, "duplicate export attributes"));
        }
        let Some(attr) = exports.first() else {
            continue;
        };
        let options = attribute_options(attr);
        attrs.retain(|a| !binding_attribute(a));
        if let Item::Fn(function) = item {
            if !matches!(function.vis, syn::Visibility::Public(_)) {
                return Err(syn::Error::new_spanned(
                    function,
                    "module exports must be public",
                ));
            }
            let member = function.sig.ident.to_string();
            *item = Item::Verbatim(export_function_in(
                options.to_string(),
                function.clone(),
                name.clone(),
                member,
                Some(&scope),
            )?);
        } else {
            let (ident, vis) = match item {
                Item::Struct(s) => (&s.ident, &s.vis),
                Item::Enum(e) => (&e.ident, &e.vis),
                Item::Type(t) => (&t.ident, &t.vis),
                _ => unreachable!(),
            };
            if !matches!(vis, syn::Visibility::Public(_)) {
                return Err(syn::Error::new_spanned(
                    ident,
                    "module exports must be public",
                ));
            }
            *item = Item::Verbatim(expand_in(options.to_string(), item.clone(), Some(&scope))?);
        }
    }
    let descriptor = section(
        &format!("MODULE_{name}"),
        json!({"kind":"module","docs":module_docs,"name":format!("OcamljsModule__{name}"),"member":name,"rust_path":rust_path,"ty":scope.t.as_ref().map(|t|json!({"kind":"named","name":scope.types[t]}))}),
    );
    items.push(Item::Verbatim(descriptor));
    Ok(quote!(#module))
}
pub(super) fn inherent_impl(
    options: &str,
    mut block: syn::ItemImpl,
) -> syn::Result<proc_macro2::TokenStream> {
    if !options.trim().is_empty() || block.trait_.is_some() || !block.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &block.self_ty,
            "export a concrete inherent impl",
        ));
    }
    let target = *block.self_ty.clone();
    let t = type_name(&target);
    if t.is_empty() {
        return Err(syn::Error::new_spanned(
            target,
            "expected a named type for t",
        ));
    }
    struct ResolveSelf(Type);
    impl VisitMut for ResolveSelf {
        fn visit_type_mut(&mut self, ty: &mut Type) {
            if matches!(ty,Type::Path(p) if p.path.is_ident("Self")) {
                *ty = self.0.clone();
            } else {
                syn::visit_mut::visit_type_mut(self, ty);
            }
        }
    }
    let mut exports = Vec::new();
    for item in &mut block.items {
        let syn::ImplItem::Fn(method) = item else {
            continue;
        };
        let attrs = method
            .attrs
            .iter()
            .filter(|a| binding_attribute(a))
            .cloned()
            .collect::<Vec<_>>();
        method.attrs.retain(|a| !binding_attribute(a));
        if !matches!(method.vis, syn::Visibility::Public(_)) {
            continue;
        }
        if attrs.len() > 1 {
            return Err(syn::Error::new_spanned(
                method,
                "duplicate export attributes",
            ));
        }
        let mut options = attrs
            .first()
            .map(attribute_options)
            .unwrap_or_default()
            .to_string();
        let member = method.sig.ident.clone();
        if member == "dispose" {
            return Err(syn::Error::new_spanned(
                member,
                "dispose is generated from Rust drop semantics",
            ));
        }
        let name = format_ident!("__ocamljs_method_{}_{}", t, member);
        let mut sig = method.sig.clone();
        sig.ident = name;
        let mut receiver_name = "__ocamljs_self".to_string();
        while sig.inputs.iter().any(|a|matches!(a,FnArg::Typed(a) if matches!(&*a.pat,Pat::Ident(p) if p.ident==receiver_name))) {receiver_name.push('_');}
        let receiver_name = format_ident!("{receiver_name}");
        let mut arguments = Vec::new();
        for argument in &mut sig.inputs {
            if let FnArg::Receiver(receiver) = argument {
                if receiver
                    .reference
                    .as_ref()
                    .is_some_and(|(_, lifetime)| lifetime.is_some())
                {
                    return Err(syn::Error::new_spanned(
                        receiver,
                        "explicit self lifetimes are unsupported",
                    ));
                }
                if receiver.colon_token.is_some() {
                    return Err(syn::Error::new_spanned(
                        receiver,
                        "typed self receivers are unsupported",
                    ));
                }
                let ty: Type = if receiver.reference.is_some() {
                    if receiver.mutability.is_some() {
                        syn::parse_quote!(&mut #target)
                    } else {
                        syn::parse_quote!(&#target)
                    }
                } else {
                    if !options.is_empty() {
                        options.push(',');
                    }
                    options.push_str(&format!("consume({receiver_name})"));
                    target.clone()
                };
                *argument = syn::parse_quote!(#receiver_name: #ty);
            }
            if let FnArg::Typed(argument) = argument {
                ResolveSelf(target.clone()).visit_type_mut(&mut argument.ty);
                let Pat::Ident(name) = &*argument.pat else {
                    return Err(syn::Error::new_spanned(
                        argument,
                        "use named method arguments",
                    ));
                };
                let name = &name.ident;
                arguments.push(quote!(#name));
            }
        }
        if let ReturnType::Type(_, ty) = &mut sig.output {
            ResolveSelf(target.clone()).visit_type_mut(ty);
        }
        let call = if sig.asyncness.is_some() {
            quote!(#target::#member(#(#arguments),*).await)
        } else {
            quote!(#target::#member(#(#arguments),*))
        };
        let attrs = &method.attrs;
        let cfg = block
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
            .collect::<Vec<_>>();
        let f: syn::ItemFn =
            syn::parse_quote!(#[allow(non_snake_case)] #(#cfg)* #(#attrs)* #sig { #call });
        exports.push(export_function_scoped(
            options,
            f,
            t.clone(),
            member.to_string(),
        )?);
    }
    Ok(quote!(#block #(#exports)*))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_ambiguous_or_incomplete_group_declarations() {
        assert!(
            inherent_impl(
                "",
                syn::parse_quote!(impl Foo {pub fn bad(&'static self) {}})
            )
            .is_err()
        );
        assert!(
            module(
                "t = Foo, t = Bar",
                syn::parse_quote!(
                    mod api {}
                )
            )
            .is_err()
        );
        for source in [
            "impl<T> Foo<T> { pub fn new() -> Self { todo!() } }",
            "impl Clone for Foo { fn clone(&self)->Self { todo!() } }",
            "impl Foo { pub fn dispose(self) {} }",
        ] {
            assert!(inherent_impl("", syn::parse_str(source).unwrap()).is_err());
        }
        assert!(
            module(
                "t = Foo",
                syn::parse_quote!(
                    mod api {}
                )
            )
            .is_err()
        );
        assert!(
            module(
                "opaque_t = Foo",
                syn::parse_quote!(
                    mod api;
                )
            )
            .is_err()
        );
        assert!(
            module(
                "opaque_t = Foo",
                syn::parse_quote!(
                    mod api {
                        struct Bar;
                    }
                )
            )
            .is_err()
        );
    }
    #[test]
    fn ordinary_modules_do_not_require_t_or_constructors() {
        for source in [
            "pub mod empty {}",
            "pub mod arithmetic { #[ocamljs_bindgen] pub fn add(a:f64,b:f64)->f64 {a+b} }",
            "pub mod labels { #[ocamljsify] pub struct Label<T> { pub value:T } }",
        ] {
            assert!(module("", syn::parse_str(source).unwrap()).is_ok());
        }
        assert!(
            module(
                "t = Point",
                syn::parse_quote!(
                    pub mod geometry {
                        #[ocamljsify]
                        pub struct Point {
                            pub x: f64,
                        }
                        #[ocamljs_bindgen]
                        pub fn origin() -> Point {
                            todo!()
                        }
                    }
                )
            )
            .is_ok()
        );
        for (options, source) in [
            ("main = Point", "pub mod geometry {}"),
            ("constructor = create", "pub mod geometry {}"),
            ("", "pub mod geometry { #[ocamljsify] struct Point; }"),
            ("", "pub mod geometry { #[ocamljs_bindgen] fn hidden() {} }"),
            (
                "",
                "pub mod geometry { #[ocamljsify] #[ocamljs_bindgen] pub struct Point; }",
            ),
        ] {
            assert!(
                module(options, syn::parse_str(source).unwrap()).is_err(),
                "{options}: {source}"
            );
        }
    }
    #[test]
    fn opaque_t_selects_representation_without_a_constructor() {
        for source in [
            "pub mod point { pub struct Point { value:u32 } }",
            "pub mod point { #[ocamljsify] pub struct Point { value:u32 } }",
            "pub mod point { #[ocamljs_bindgen(opaque)] pub struct Point { value:u32 } }",
        ] {
            assert!(module("opaque_t = Point", syn::parse_str(source).unwrap()).is_ok());
        }
        for (options, source) in [
            (
                "t = Point, opaque_t = Point",
                "pub mod point {pub struct Point; }",
            ),
            ("opaque_t = Missing", "pub mod point {pub struct Point; }"),
            (
                "opaque_t = Point",
                "pub mod point {pub struct Point<T> { value:T } }",
            ),
            ("opaque_t = Point", "pub mod point {struct Point; }"),
            (
                "opaque_t = Point",
                "pub mod point { #[ocamljsify(transparent)] pub struct Point(u32); }",
            ),
            (
                "opaque_t = Point",
                "pub mod point { #[ocamljsify] #[ocamljs_bindgen] pub struct Point; }",
            ),
        ] {
            assert!(
                module(options, syn::parse_str(source).unwrap()).is_err(),
                "{options}: {source}"
            );
        }
        for source in [
            "pub struct Point;",
            "pub enum Point { Origin }",
            "pub type Point = u32;",
        ] {
            assert!(
                expand(
                    "constructor = create".into(),
                    syn::parse_str(source).unwrap()
                )
                .is_err()
            );
        }
    }
    #[test]
    fn ordinary_type_descriptions_namespace_locals_without_capturing_generics() {
        let scope = Scope {
            module: "Geometry".into(),
            types: std::collections::BTreeMap::from([
                ("Point".into(), "Geometry__Point".into()),
                ("T".into(), "Geometry__T".into()),
            ]),
            t: None,
        };
        let description = scope.describe(
            json!({"kind":"struct","name":"Point","params":["T"],"body":{"fields":[
                {"ty":{"kind":"named","name":"T","qualified":false}},
                {"ty":{"kind":"named","name":"T","qualified":true,"path":"self::T"}},
                {"ty":{"kind":"named","name":"Point","qualified":true,"path":"super::Point"}}
            ]}}),
            true,
        );
        assert_eq!(description["name"], "Geometry__Point");
        assert_eq!(description["member"], "Point");
        assert_eq!(description["body"]["fields"][0]["ty"]["name"], "T");
        assert_eq!(
            description["body"]["fields"][1]["ty"]["name"],
            "Geometry__T"
        );
        assert_eq!(description["body"]["fields"][2]["ty"]["name"], "Point");
    }
    #[test]
    fn inherent_methods_preserve_receivers_self_types_and_owned_consumption() {
        let output = inherent_impl(
            "",
            syn::parse_quote!(impl Foo {
                pub fn new() -> Self {todo!()}
                pub async fn copy(&self, other:&Self) -> Result<Self, Error> {todo!()}
                pub fn take(self) -> u32 {todo!()}
            }),
        )
        .unwrap();
        let file: syn::File = syn::parse2(output).unwrap();
        let funcs = file
            .items
            .iter()
            .filter_map(|i| if let Item::Fn(f) = i { Some(f) } else { None })
            .collect::<Vec<_>>();
        assert!(
            funcs
                .iter()
                .any(|f| f.sig.ident == "__ocamljs_method_Foo_copy" && f.sig.asyncness.is_some())
        );
        assert!(
            funcs
                .iter()
                .all(|f| !f.sig.inputs.iter().any(|p| matches!(p, FnArg::Receiver(_))))
        );
        let text = quote!(#file).to_string();
        assert!(text.contains("MoveAnchor"));
        assert!(text.contains("Borrowable"));
    }
    #[test]
    fn same_method_names_have_distinct_export_symbols() {
        let left = inherent_impl(
            "",
            syn::parse_quote!(impl Left {pub fn new()->Self {todo!()}}),
        )
        .unwrap();
        let right = inherent_impl(
            "",
            syn::parse_quote!(impl Right {pub fn new()->Self {todo!()}}),
        )
        .unwrap();
        let file: syn::File = syn::parse2(quote!(#left #right)).unwrap();
        let names = file
            .items
            .iter()
            .filter_map(|i| {
                if let Item::Fn(f) = i {
                    Some(f.sig.ident.to_string())
                } else {
                    None
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert!(names.contains("__ocamljs_fn___ocamljs_Left_new"));
        assert!(names.contains("__ocamljs_fn___ocamljs_Right_new"));
    }
}

#[cfg(test)]
mod receiver_hygiene {
    use super::*;
    #[test]
    fn receiver_does_not_capture_user_argument_names() {
        let file: syn::File = syn::parse2(
            inherent_impl(
                "",
                syn::parse_quote!(impl Foo {
                    pub fn method(&self, this:u32, __ocamljs_self:u32)->u32 {this+__ocamljs_self}
                }),
            )
            .unwrap(),
        )
        .unwrap();
        let wrapper = file
            .items
            .iter()
            .find_map(|i| match i {
                Item::Fn(f) if f.sig.ident == "__ocamljs_method_Foo_method" => Some(f),
                _ => None,
            })
            .unwrap();
        let names = wrapper
            .sig
            .inputs
            .iter()
            .map(|a| match a {
                FnArg::Typed(a) => a.pat.to_token_stream().to_string(),
                _ => panic!(),
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(names.len(), 3);
        assert!(names.contains("this"));
        assert!(names.contains("__ocamljs_self"));
    }
}
