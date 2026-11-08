//! Procedural API description and checked Rust export wrappers, written from scratch.
use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde_json::{Value, json};
use syn::{Attribute, Fields, FnArg, GenericParam, Item, Pat, ReturnType, Type, parse_macro_input};

mod groups;
mod host;
use groups::identifier_option;

fn docs(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|a| {
            if a.path().is_ident("doc") {
                if let syn::Meta::NameValue(v) = &a.meta {
                    if let syn::Expr::Lit(v) = &v.value {
                        if let syn::Lit::Str(s) = &v.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                }
                None
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
fn meta(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("serde"))
        .map(|a| a.meta.to_token_stream().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
fn serde_value(attrs: &[Attribute], key: &str) -> syn::Result<Option<String>> {
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let metadata = attr.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        )?;
        for item in metadata {
            if let syn::Meta::List(ref list) = item {
                if list.path.is_ident(key) {
                    return Err(syn::Error::new_spanned(
                        list,
                        "directional serde enum names are unsupported",
                    ));
                }
            }
            if let syn::Meta::NameValue(value) = item {
                if value.path.is_ident(key) {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(text),
                        ..
                    }) = value.value
                    {
                        return Ok(Some(text.value()));
                    }
                    return Err(syn::Error::new_spanned(value, "expected a string literal"));
                }
            }
        }
    }
    Ok(None)
}
fn enum_name(
    name: &syn::Ident,
    attrs: &[Attribute],
    enum_attrs: &[Attribute],
) -> syn::Result<String> {
    if let Some(rename) = serde_value(attrs, "rename")? {
        return Ok(rename);
    }
    let name = name.to_string().trim_start_matches("r#").to_owned();
    let rule = serde_value(enum_attrs, "rename_all")?;
    let snake = || {
        name.chars()
            .enumerate()
            .fold(String::new(), |mut text, (index, c)| {
                if index > 0 && c.is_ascii_uppercase() {
                    text.push('_');
                }
                text.push(c.to_ascii_lowercase());
                text
            })
    };
    Ok(match rule.as_deref() {
        None | Some("PascalCase") => name,
        Some("lowercase") => name.to_ascii_lowercase(),
        Some("UPPERCASE") => name.to_ascii_uppercase(),
        Some("camelCase") => {
            let mut chars = name.chars();
            let first = chars.next().unwrap().to_ascii_lowercase();
            format!("{first}{}", chars.as_str())
        }
        Some("snake_case") => snake(),
        Some("SCREAMING_SNAKE_CASE") => snake().to_ascii_uppercase(),
        Some("kebab-case") => snake().replace('_', "-"),
        Some("SCREAMING-KEBAB-CASE") => snake().replace('_', "-").to_ascii_uppercase(),
        Some(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "unsupported serde rename_all rule",
            ));
        }
    })
}
fn derived(attrs: &[Attribute], trait_name: &str) -> bool {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .any(|a| {
            a.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| {
                paths
                    .iter()
                    .any(|p| p.segments.last().is_some_and(|p| p.ident == trait_name))
            })
        })
}
fn structural_key_traits(attrs: &[Attribute], contract: bool) -> (bool, bool) {
    let equality = contract || derived(attrs, "PartialEq") && derived(attrs, "Eq");
    let ordering = contract || derived(attrs, "PartialOrd") && derived(attrs, "Ord");
    (equality, ordering)
}
fn representation_options(options: &str) -> syn::Result<(String, bool, bool, String)> {
    use syn::parse::Parser;
    let parsed = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
        .parse_str(options)?;
    let (mut js_value, mut js_value_rec) = (false, false);
    let mut selector = String::new();
    let mut rest = Vec::new();
    for option in parsed {
        match &option {
            syn::Meta::Path(path) if path.is_ident("js_value") || path.is_ident("js_value_rec") => {
                if js_value || js_value_rec {
                    return Err(syn::Error::new_spanned(
                        option,
                        "choose js_value or js_value_rec once",
                    ));
                }
                js_value = path.is_ident("js_value");
                js_value_rec = path.is_ident("js_value_rec");
            }
            syn::Meta::List(list) if list.path.is_ident("js_value") => {
                if js_value || js_value_rec {
                    return Err(syn::Error::new_spanned(option, "choose one representation"));
                }
                selector = js_selector(list)?;
                js_value = true;
            }
            _ => rest.push(option.to_token_stream().to_string()),
        }
    }
    Ok((rest.join(", "), js_value, js_value_rec, selector))
}
fn js_selector(list: &syn::MetaList) -> syn::Result<String> {
    let value = list.parse_args::<syn::LitStr>()?;
    let selector = value.value();
    if !matches!(
        selector.as_str(),
        "number"
            | "bigint"
            | "boolean"
            | "string"
            | "array"
            | "object"
            | "dict"
            | "map"
            | "typed_array"
            | "nullable"
            | "optional"
    ) {
        return Err(syn::Error::new_spanned(
            value,
            "unknown JavaScript representation selector",
        ));
    }
    Ok(selector)
}
fn representation(mut value: Value, js: bool, recursive: bool, selector: &str) -> Value {
    if !selector.is_empty() {
        value["js_selector"] = json!(selector);
    }
    if js {
        value["js_value"] = json!(true);
    }
    if recursive {
        value["js_value_rec"] = json!(true);
    }
    value
}
fn field_representation(attrs: &[Attribute]) -> syn::Result<(bool, bool, String)> {
    let (mut js, mut recursive) = (false, false);
    let mut selector = String::new();
    for attr in attrs.iter().filter(|attr| {
        attr.path().segments.last().is_some_and(|segment| {
            segment.ident == "ocamljsify" || segment.ident == "ocamljs_bindgen"
        })
    }) {
        let tokens = match &attr.meta {
            syn::Meta::List(list) => list.tokens.to_string(),
            _ => {
                return Err(syn::Error::new_spanned(
                    attr,
                    "field options: js_value or js_value_rec",
                ));
            }
        };
        let (rest, selected, deep, selection) = representation_options(&tokens)?;
        if !rest.is_empty() || !(selected || deep) || js || recursive {
            return Err(syn::Error::new_spanned(
                attr,
                "field options: js_value or js_value_rec once",
            ));
        }
        js = selected;
        recursive = deep;
        selector = selection;
    }
    Ok((js, recursive, selector))
}
fn strip_field_representation(fields: &mut Fields) {
    for field in fields {
        field.attrs.retain(|attr| {
            !attr.path().segments.last().is_some_and(|segment| {
                segment.ident == "ocamljsify" || segment.ident == "ocamljs_bindgen"
            })
        });
    }
}
fn collection_options(options: &str) -> syn::Result<(String, bool, String, bool)> {
    use syn::parse::Parser;
    let parsed = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
        .parse_str(options)?;
    let mut rest = Vec::new();
    let mut structural = false;
    let mut borrowed = false;
    let mut map = String::new();
    for item in parsed {
        if matches!(&item,syn::Meta::Path(path) if path.is_ident("borrow")) {
            borrowed = true;
            continue;
        }
        if let syn::Meta::NameValue(value) = &item {
            if value.path.is_ident("map") || value.path.is_ident("key") {
                let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(text),
                    ..
                }) = &value.value
                else {
                    return Err(syn::Error::new_spanned(value, "expected string literal"));
                };
                if value.path.is_ident("map") {
                    if text.value() != "dict" {
                        return Err(syn::Error::new_spanned(
                            text,
                            "supported map representation: dict",
                        ));
                    }
                    map = text.value();
                } else {
                    if text.value() != "structural" {
                        return Err(syn::Error::new_spanned(
                            text,
                            "supported key contract: structural",
                        ));
                    }
                    structural = true;
                }
                continue;
            }
        }
        rest.push(item.to_token_stream().to_string());
    }
    Ok((rest.join(", "), structural, map, borrowed))
}
fn map_policy(value: &mut Value, policy: &str) {
    if policy.is_empty() {
        return;
    }
    if let Some(object) = value.as_object_mut() {
        if matches!(
            object.get("name").and_then(Value::as_str),
            Some("BTreeMap" | "HashMap")
        ) && object
            .get("args")
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(|k| k.get("name"))
            .and_then(Value::as_str)
            == Some("String")
        {
            object.insert("map".into(), Value::String(policy.into()));
        }
        for child in object.values_mut() {
            map_policy(child, policy);
        }
    } else if let Some(array) = value.as_array_mut() {
        for child in array {
            map_policy(child, policy);
        }
    }
}
fn described_section(
    name: &str,
    mut value: Value,
    map: &str,
    scope: Option<&groups::Scope>,
) -> proc_macro2::TokenStream {
    map_policy(&mut value, map);
    let value = if let Some(scope) = scope {
        scope.describe(value, true)
    } else {
        value
    };
    section(name, value)
}
fn ty(t: &Type) -> syn::Result<Value> {
    Ok(match t {
        Type::Path(p) if p.qself.is_none() => {
            let s = p.path.segments.last().unwrap();
            let name = s.ident.to_string();
            let mut args = vec![];
            if let syn::PathArguments::AngleBracketed(a) = &s.arguments {
                for a in &a.args {
                    if let syn::GenericArgument::Type(t) = a {
                        args.push(ty(t)?);
                    } else {
                        return Err(syn::Error::new_spanned(
                            a,
                            "only type generic arguments are supported",
                        ));
                    }
                }
            }
            {
                let mut value = json!({"kind":"named","name":name,"qualified":p.path.segments.len()>1,"args":args});
                if p.path.segments.len() > 1 {
                    value["path"] = json!(
                        p.path
                            .segments
                            .iter()
                            .map(|p| p.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::")
                    );
                }
                value
            }
        }
        Type::Tuple(t) => {
            json!({"kind":"tuple","args":t.elems.iter().map(ty).collect::<syn::Result<Vec<_>>>()?})
        }
        Type::Reference(r) => {
            json!({"kind":"ref","mutable":r.mutability.is_some(),"inner":ty(&r.elem)?})
        }
        Type::Array(a) => {
            let n = if let syn::Expr::Lit(l) = &a.len {
                if let syn::Lit::Int(n) = &l.lit {
                    n.base10_parse::<usize>()?
                } else {
                    return Err(syn::Error::new_spanned(
                        &a.len,
                        "array length must be an integer",
                    ));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    &a.len,
                    "array length must be literal",
                ));
            };
            json!({"kind":"array","inner":ty(&a.elem)?,"len":n})
        }
        Type::Slice(s) => json!({"kind":"slice","inner":ty(&s.elem)?}),
        _ => {
            return Err(syn::Error::new_spanned(
                t,
                "unsupported ABI type; use owned data, primitives, or a call-scoped opaque reference",
            ));
        }
    })
}
fn fields(f: &Fields) -> syn::Result<Value> {
    let fields = f.iter().enumerate().map(|(index, field)| {
        let (js, recursive, selector) = field_representation(&field.attrs)?;
        Ok(representation(json!({"name":field.ident.as_ref().map(|ident|ident.to_string().trim_start_matches("r#").to_owned()).unwrap_or(index.to_string()),"ty":ty(&field.ty)?,"docs":docs(&field.attrs),"serde":meta(&field.attrs)}), js, recursive, &selector))
    }).collect::<syn::Result<Vec<_>>>()?;
    Ok(
        json!({"style":match f {Fields::Named(_)=>"named",Fields::Unnamed(_)=>"tuple",Fields::Unit=>"unit"},"fields":fields}),
    )
}

fn section(name: &str, mut v: Value) -> proc_macro2::TokenStream {
    // New association fields are optional. Do not retain redundant defaults in
    // Rust's metadata statics or the extracted sidecar.
    if let Some(object) = v.as_object_mut() {
        for key in ["module"] {
            if object.get(key).is_some_and(|value| value == "") {
                object.remove(key);
            }
        }
        if object.get("member") == object.get("name") {
            object.remove("member");
        }
    }
    let s = format!("{}\n", v);
    let b = syn::LitByteStr::new(s.as_bytes(), proc_macro2::Span::call_site());
    let n = s.len();
    let id = format_ident!("__OCAMLJS_METADATA_{}", name);
    quote! { #[cfg(target_arch="wasm32")] #[used] #[unsafe(link_section="ocamljs.schema")] static #id: [u8;#n] = *#b; }
}
fn scalar(t: &Type) -> bool {
    if let Type::Path(p) = t {
        matches!(
            p.path.segments.last().unwrap().ident.to_string().as_str(),
            "bool"
                | "u8"
                | "i8"
                | "u16"
                | "i16"
                | "u32"
                | "i32"
                | "usize"
                | "isize"
                | "u64"
                | "i64"
                | "f32"
                | "f64"
        )
    } else {
        matches!(t,Type::Tuple(t) if t.elems.is_empty())
    }
}
// Eager projections are complete buffered Result returns. Constrain the
// export position so scalar/flat ABI selection and nested collection limits
// cannot accidentally disagree with the published result schema.
fn result_ty(t: &Type) -> syn::Result<Value> {
    if let Type::Path(p) = t {
        let segment = p.path.segments.last().unwrap();
        if segment.ident == "Encoded" {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if args.args.len() == 1 {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        let schema = ty(inner)?;
                        if schema["name"] == "Result"
                            && schema["args"]
                                .as_array()
                                .is_some_and(|args| args.len() == 2)
                        {
                            return Ok(schema);
                        }
                    }
                }
            }
            return Err(syn::Error::new_spanned(
                t,
                "Encoded exports require a complete Result<T, E> return",
            ));
        }
    }
    ty(t)
}
fn type_name(t: &Type) -> String {
    if let Type::Path(p) = t {
        p.path.segments.last().unwrap().ident.to_string()
    } else {
        String::new()
    }
}
fn scalar_abi(t: &Type) -> proc_macro2::TokenStream {
    if type_name(t) == "bool" {
        quote!(u32)
    } else if matches!(t,Type::Tuple(t) if t.elems.is_empty()) {
        quote!(())
    } else {
        quote!(#t)
    }
}
fn tuple_lanes(t: &Type, values: &mut Vec<Type>) -> bool {
    if scalar(t) && !matches!(t, Type::Tuple(_)) {
        values.push(t.clone());
        return values.len() <= 8;
    }
    matches!(t,Type::Tuple(tuple) if !tuple.elems.is_empty() && tuple.elems.iter().all(|t|tuple_lanes(t,values)))
}
fn tuple_value(
    t: &Type,
    values: &mut impl Iterator<Item = syn::Ident>,
) -> proc_macro2::TokenStream {
    if let Type::Tuple(tuple) = t {
        let parts = tuple
            .elems
            .iter()
            .map(|t| tuple_value(t, values))
            .collect::<Vec<_>>();
        quote!((#(#parts,)*))
    } else {
        let value = values.next().unwrap();
        if type_name(t) == "bool" {
            quote!(match #value {0=>false,1=>true,_=>return Err(::ocamljs_bindgen::BoundaryError("invalid bool"))})
        } else {
            quote!(#value)
        }
    }
}
fn flat_fields<'a>(g: &syn::Generics, data: &'a syn::Data) -> Option<&'a Fields> {
    if !g.params.is_empty() {
        return None;
    }
    let syn::Data::Struct(s) = data else {
        return None;
    };
    if s.fields.is_empty()
        || s.fields.len() > 8
        || !s
            .fields
            .iter()
            .all(|f| scalar(&f.ty) && !matches!(&f.ty, Type::Tuple(_)))
    {
        return None;
    }
    Some(&s.fields)
}
fn codec(
    name: &syn::Ident,
    g: &syn::Generics,
    data: &syn::Data,
    borrowed: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut eg = g.clone();
    let mut dg = g.clone();
    let mut rg = g.clone();
    for p in &mut rg.params {
        if let GenericParam::Type(t) = p {
            t.bounds
                .push(syn::parse_quote!(::ocamljs_bindgen::EncodeRef));
        }
    }
    let (ri, _, rw) = rg.split_for_impl();
    for p in &mut eg.params {
        if let GenericParam::Type(t) = p {
            t.bounds.push(syn::parse_quote!(::ocamljs_bindgen::Encode));
        }
    }
    for p in &mut dg.params {
        if let GenericParam::Type(t) = p {
            t.bounds.push(syn::parse_quote!(::ocamljs_bindgen::Decode));
        }
    }
    let (ei, tg, ew) = eg.split_for_impl();
    let (di, _, dw) = dg.split_for_impl();
    let arm = |prefix: proc_macro2::TokenStream, f: &Fields| {
        let vars = (0..f.len())
            .map(|i| format_ident!("field_{i}"))
            .collect::<Vec<_>>();
        let (pat, build) = match f {
            Fields::Named(fs) => {
                let names = fs
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect::<Vec<_>>();
                (
                    quote!(#prefix { #(#names: #vars),* }),
                    quote!(#prefix { #(#names: ::ocamljs_bindgen::Decode::decode(input)?),* }),
                )
            }
            Fields::Unnamed(_) => (
                quote!(#prefix(#(#vars),*)),
                quote!(#prefix(#({ let value = ::ocamljs_bindgen::Decode::decode(input)?; let _ = stringify!(#vars); value }),*)),
            ),
            Fields::Unit => (quote!(#prefix), quote!(#prefix)),
        };
        (
            pat,
            build,
            quote!(#(::ocamljs_bindgen::Encode::encode(#vars,out);)*),
            quote!(#(::ocamljs_bindgen::EncodeRef::encode_ref(#vars,out);)*),
        )
    };
    let (encode, decode, borrowed_encode) = match data {
        syn::Data::Struct(s) => {
            let (p, b, e, r) = arm(quote!(Self), &s.fields);
            (
                quote!(let #p = self; #e),
                quote!(Ok(#b)),
                quote!(let #p = self; #r),
            )
        }
        syn::Data::Enum(e) => {
            let mut en = vec![];
            let mut de = vec![];
            let mut borrowed = vec![];
            for (i, v) in e.variants.iter().enumerate() {
                let vi = &v.ident;
                let (p, b, body, read) = arm(quote!(Self::#vi), &v.fields);
                let i = i as u32;
                en.push(quote!(#p => { ::ocamljs_bindgen::Encode::encode(#i,out); #body }));
                de.push(quote!(#i => Ok(#b)));
                borrowed.push(quote!(#p => { ::ocamljs_bindgen::Encode::encode(#i,out); #read }));
            }
            (
                quote!(match self { #(#en),* }),
                quote!(match <u32 as ::ocamljs_bindgen::Decode>::decode(input)? { #(#de,)* _ => Err(::ocamljs_bindgen::BoundaryError("invalid variant tag")) }),
                quote!(match self { #(#borrowed),* }),
            )
        }
        _ => return Err(syn::Error::new_spanned(name, "unions are not supported")),
    };
    let (lower, abi_impl) = if let Some(fields) = flat_fields(g, data) {
        let lanes = (0..8).map(|i| format_ident!("A{i}")).collect::<Vec<_>>();
        let values = (0..8).map(|i| format_ident!("a{i}")).collect::<Vec<_>>();
        let types = (0..8)
            .map(|i| {
                fields
                    .iter()
                    .nth(i)
                    .map(|f| scalar_abi(&f.ty))
                    .unwrap_or(quote!(()))
            })
            .collect::<Vec<_>>();
        let lifted = fields.iter().zip(&values).map(|(f,v)| if type_name(&f.ty)=="bool" {quote!(match #v {0=>false,1=>true,_=>return Err(::ocamljs_bindgen::BoundaryError("invalid bool"))})} else {quote!(#v)}).collect::<Vec<_>>();
        let build = match fields {
            Fields::Named(fs) => {
                let names = fs.named.iter().map(|f| f.ident.as_ref().unwrap());
                quote!(#name {#(#names:#lifted),*})
            }
            _ => quote!(#name(#(#lifted),*)),
        };
        (
            quote!(::ocamljs_bindgen::abi::Flat<Self>),
            quote!(
                impl ::ocamljs_bindgen::abi::Argument<#name> for ::ocamljs_bindgen::abi::Flat<#name> {
                    type Adopted=();
                    #(type #lanes = #types;)*
                    unsafe fn lift(_: (), #(#values:Self::#lanes),*) -> ::ocamljs_bindgen::BoundaryResult<#name> {Ok(#build)}
                }
            ),
        )
    } else {
        (quote!(::ocamljs_bindgen::abi::Buffer<Self>), quote!())
    };
    let mut return_impl = quote!();
    let return_abi = if let Some(fields) = flat_fields(g, data).filter(|fs| {
        fs.iter()
            .all(|f| !matches!(type_name(&f.ty).as_str(), "bool" | "u64" | "i64"))
    }) {
        let mut offset = 0usize;
        let mut stores = Vec::new();
        for (i, f) in fields.iter().enumerate() {
            let size = match type_name(&f.ty).as_str() {
                "u8" | "i8" => 1,
                "u16" | "i16" => 2,
                "f64" => 8,
                _ => 4,
            };
            let end = offset + size;
            let member = f.ident.as_ref().map(|id| quote!(#id)).unwrap_or_else(|| {
                let i = syn::Index::from(i);
                quote!(#i)
            });
            // wasm32 usize/isize have the wire width used above.
            let value = match type_name(&f.ty).as_str() {
                "usize" => quote!(value.#member as u32),
                "isize" => quote!(value.#member as i32),
                _ => quote!(value.#member),
            };
            stores.push(quote!(bytes[#offset..#end].copy_from_slice(&(#value).to_le_bytes());));
            offset = end;
        }
        return_impl = quote!(impl ::ocamljs_bindgen::abi::ReturnValue<#name> for ::ocamljs_bindgen::abi::Flat<#name> {
            fn lower(value:#name)->::ocamljs_bindgen::BoundaryResult<u32>{Ok(::ocamljs_bindgen::abi::flat_output(|bytes|{#(#stores)*}))}
            fn complete(value:#name)->::ocamljs_bindgen::BoundaryResult<::ocamljs_bindgen::abi::Completion>{Ok(::ocamljs_bindgen::abi::Completion::write(|bytes|{#(#stores)*}))}
        });
        quote!(::ocamljs_bindgen::abi::Flat<Self>)
    } else {
        quote!(::ocamljs_bindgen::abi::Buffer<Self>)
    };
    let reference_codec = if borrowed {
        quote! { impl #ri ::ocamljs_bindgen::EncodeRef for #name #tg #rw { fn encode_ref(&self,out:&mut ::ocamljs_bindgen::Encoder) { out.nested(|out| { #borrowed_encode }); } } }
    } else {
        quote!()
    };
    Ok(
        quote! { #reference_codec impl #ei ::ocamljs_bindgen::Encode for #name #tg #ew { type Return = #return_abi; fn encode(self,out:&mut ::ocamljs_bindgen::Encoder) { out.nested(|out| { #encode }); } } impl #di ::ocamljs_bindgen::Decode for #name #tg #dw { type Abi = #lower; fn decode(input:&mut ::ocamljs_bindgen::Decoder<'_>)->::ocamljs_bindgen::BoundaryResult<Self> { input.nested(|input| { #decode }) } } #abi_impl #return_impl },
    )
}
#[proc_macro_attribute]
pub fn ocamljs_bindgen(options: TokenStream, input: TokenStream) -> TokenStream {
    let options = options.to_string();
    let item = parse_macro_input!(input as Item);
    match expand(options, item) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
fn expand(options: String, item: Item) -> syn::Result<proc_macro2::TokenStream> {
    expand_in(options, item, None)
}
fn expand_in(
    options: String,
    item: Item,
    scope: Option<&groups::Scope>,
) -> syn::Result<proc_macro2::TokenStream> {
    if let Item::Mod(module) = item {
        return groups::module(&options, module);
    }
    if let Item::Impl(block) = item {
        return groups::inherent_impl(&options, block);
    }
    if let Item::ForeignMod(block) = item {
        return host::imports(&options, block);
    }
    let (options, type_js, type_js_rec, type_js_selector) =
        if matches!(&item, Item::Struct(_) | Item::Enum(_) | Item::Type(_)) {
            representation_options(&options)?
        } else {
            (options, false, false, String::new())
        };
    let (options, structural, map, borrowed) = collection_options(&options)?;
    match item {
        Item::Struct(s) => {
            let name = &s.ident;
            if borrowed && matches!(options.trim(), "host" | "opaque") {
                return Err(syn::Error::new_spanned(
                    name,
                    "borrow requires structural data; owned handles cannot be borrowed as data",
                ));
            }
            if options.trim() == "host" {
                return host::host_type(s, scope);
            }
            let opaque = options.trim() == "opaque";
            let transparent = options.trim() == "transparent";
            if !options.trim().is_empty() && !opaque && !transparent {
                return Err(syn::Error::new_spanned(
                    name,
                    "struct options: opaque, transparent",
                ));
            }
            if opaque && !s.generics.params.is_empty() {
                return Err(syn::Error::new_spanned(
                    name,
                    "opaque types must be concrete",
                ));
            }
            if transparent && s.fields.len() != 1 {
                return Err(syn::Error::new_spanned(
                    name,
                    "transparent requires exactly one field",
                ));
            }
            let md = described_section(
                &name.to_string(),
                representation(
                    json!({"key_eq":structural_key_traits(&s.attrs,structural).0,"key_ord":structural_key_traits(&s.attrs,structural).1,"flat":!opaque && flat_fields(&s.generics, &syn::Data::Struct(syn::DataStruct {struct_token:s.struct_token,fields:s.fields.clone(),semi_token:s.semi_token})).is_some(),"transparent":transparent,"kind":if opaque {"opaque"} else {"struct"},"name":name.to_string(),"params":s.generics.type_params().map(|p|p.ident.to_string()).collect::<Vec<_>>(),"body":if opaque {Value::Null}else{fields(&s.fields)?},"docs":docs(&s.attrs),"serde":meta(&s.attrs)}),
                    type_js,
                    type_js_rec,
                    &type_js_selector,
                ),
                &map,
                scope,
            );
            let c = if opaque {
                quote!(::ocamljs_bindgen::opaque_storage!(#name); impl ::ocamljs_bindgen::Encode for #name { type Return = ::ocamljs_bindgen::abi::Opaque<Self>; fn encode(self,out:&mut ::ocamljs_bindgen::Encoder) { out.nested(|out|out.opaque(self)); } }
                impl ::ocamljs_bindgen::Decode for #name {type Abi = ::ocamljs_bindgen::abi::Buffer<Self>; fn decode(_: &mut ::ocamljs_bindgen::Decoder<'_>)->::ocamljs_bindgen::BoundaryResult<Self>{Err(::ocamljs_bindgen::BoundaryError("opaque inputs require borrowed references"))}}
                impl ::ocamljs_bindgen::host::Borrowable for #name {
                    type A1=(); type A2=(); type Adopted=();
                    type Anchor = ::ocamljs_bindgen::scoped::SharedAnchor<Self>;
                    unsafe fn anchor(id:usize,_:(),_:(),_:())->::ocamljs_bindgen::BoundaryResult<Self::Anchor>{unsafe { Self::Anchor::new(id as u32) }}
                    unsafe fn with_shared<R>(id:usize,_:(),_:(),_:(),f:impl FnOnce(&Self)->R)->::ocamljs_bindgen::BoundaryResult<R>{unsafe { ::ocamljs_bindgen::with_ref(id as u32,f) }}
                })
            } else {
                codec(
                    name,
                    &s.generics,
                    &syn::Data::Struct(syn::DataStruct {
                        struct_token: s.struct_token,
                        fields: s.fields.clone(),
                        semi_token: s.semi_token,
                    }),
                    borrowed,
                )?
            };
            let mut s = s;
            strip_field_representation(&mut s.fields);
            Ok(quote!(#s #c #md))
        }
        Item::Enum(e) => {
            if !options.trim().is_empty() {
                return Err(syn::Error::new_spanned(
                    &e.ident,
                    "enum options are not supported",
                ));
            }
            let name = &e.ident;
            let md = described_section(
                &name.to_string(),
                representation(
                    json!({"key_eq":structural_key_traits(&e.attrs,structural).0,"key_ord":!e.variants.iter().any(|v|v.discriminant.is_some()) && structural_key_traits(&e.attrs,structural).1,"kind":"enum","name":name.to_string(),"params":e.generics.type_params().map(|p|p.ident.to_string()).collect::<Vec<_>>(),"variants":e.variants.iter().map(|v| { let mut value=json!({"name":v.ident.to_string(),"serde_name":enum_name(&v.ident, &v.attrs, &e.attrs)?,"body":fields(&v.fields)?,"docs":docs(&v.attrs),"serde":meta(&v.attrs)}); if v.attrs.iter().any(|a|a.path().is_ident("default")){value["rust_default"]=json!(true);} Ok(value) }).collect::<syn::Result<Vec<_>>>()?,"docs":docs(&e.attrs),"serde":meta(&e.attrs)}),
                    type_js,
                    type_js_rec,
                    &type_js_selector,
                ),
                &map,
                scope,
            );
            let c = codec(
                name,
                &e.generics,
                &syn::Data::Enum(syn::DataEnum {
                    enum_token: e.enum_token,
                    brace_token: e.brace_token,
                    variants: e.variants.clone(),
                }),
                borrowed,
            )?;
            let mut e = e;
            for variant in &mut e.variants {
                strip_field_representation(&mut variant.fields);
            }
            Ok(quote!(#e #c #md))
        }
        Item::Type(t) => {
            if borrowed {
                return Err(syn::Error::new_spanned(
                    &t.ident,
                    "borrow applies to records and enums",
                ));
            }
            if !options.trim().is_empty() {
                return Err(syn::Error::new_spanned(
                    &t.ident,
                    "type alias options are not supported",
                ));
            }
            let name = &t.ident;
            let described_ty = ty(&t.ty)?;
            let needs_support = ["Promise", "HostError", "JsString", "Uint8ClampedArray"]
                .iter()
                .any(|n| described_ty.to_string().contains(&format!("\"{n}\"")));
            let mut description = representation(
                json!({"kind":"alias","name":name.to_string(),"params":t.generics.type_params().map(|p|p.ident.to_string()).collect::<Vec<_>>(),"ty":described_ty,"docs":docs(&t.attrs)}),
                type_js,
                type_js_rec,
                &type_js_selector,
            );
            let support = if needs_support {
                let abi_name = scope
                    .map(|s| {
                        s.describe(description.clone(), true)["name"]
                            .as_str()
                            .unwrap()
                            .to_owned()
                    })
                    .unwrap_or_else(|| name.to_string());
                let prefix = format!("__ocamljs_ref_alias_{abi_name}");
                description["reference_support"] = json!(prefix);
                host::support(&prefix)
            } else {
                quote!()
            };
            let md = described_section(&name.to_string(), description, &map, scope);
            Ok(quote!(#t #md const _: () = { #support };))
        }
        Item::Fn(f) => {
            if borrowed {
                return Err(syn::Error::new_spanned(
                    &f.sig,
                    "borrow applies to records and enums",
                ));
            }
            if !map.is_empty() || structural {
                return Err(syn::Error::new_spanned(
                    &f.sig,
                    "map/key options belong on described containers or aliases",
                ));
            }
            export_function(options, f)
        }
        _ => Err(syn::Error::new_spanned(
            item,
            "apply to a struct, enum, type alias, function, inherent impl or inline module",
        )),
    }
}

fn export_function(options: String, f: syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    export_function_scoped(options, f, String::new(), String::new())
}
fn export_function_scoped(
    options: String,
    f: syn::ItemFn,
    module: String,
    member: String,
) -> syn::Result<proc_macro2::TokenStream> {
    export_function_in(options, f, module, member, None)
}
fn export_function_in(
    options: String,
    f: syn::ItemFn,
    mut module: String,
    mut member: String,
    scope: Option<&groups::Scope>,
) -> syn::Result<proc_macro2::TokenStream> {
    let namespaced = !module.is_empty();
    use syn::parse::Parser;
    if f.sig.unsafety.is_some() || !f.sig.generics.params.is_empty() || f.sig.variadic.is_some() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "exports must be safe and concrete",
        ));
    }
    let opts = syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
        .parse_str(&options)?;
    let mut throws = false;
    let mut js_value = false;
    let mut js_value_rec = false;
    let mut selector = String::new();
    let mut consumed = std::collections::BTreeSet::new();
    for option in opts {
        if let Some(value) = identifier_option(&option, "module")? {
            if !module.is_empty() {
                return Err(syn::Error::new_spanned(
                    option,
                    "duplicate module association",
                ));
            }
            module = value.to_string();
            continue;
        }
        if let syn::Meta::NameValue(value) = &option {
            if value.path.is_ident("name") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(name),
                    ..
                }) = &value.value
                {
                    member = name.value();
                    continue;
                }
                return Err(syn::Error::new_spanned(
                    value,
                    "name requires a string literal",
                ));
            }
        }
        match option {
            syn::Meta::Path(p) if p.is_ident("throws") => throws = true,
            syn::Meta::Path(p) if p.is_ident("js_value") || p.is_ident("js_value_rec") => {
                if js_value || js_value_rec {
                    return Err(syn::Error::new_spanned(
                        p,
                        "choose js_value or js_value_rec once",
                    ));
                }
                js_value = p.is_ident("js_value");
                js_value_rec = p.is_ident("js_value_rec");
            }
            syn::Meta::List(list) if list.path.is_ident("js_value") => {
                if js_value || js_value_rec {
                    return Err(syn::Error::new_spanned(list, "choose one representation"));
                }
                selector = js_selector(&list)?;
                js_value = true;
            }
            syn::Meta::List(l) if l.path.is_ident("consume") => {
                for n in syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated
                    .parse2(l.tokens)?
                {
                    if !consumed.insert(n.to_string()) {
                        return Err(syn::Error::new_spanned(n, "duplicate consuming argument"));
                    }
                }
            }
            o => {
                return Err(syn::Error::new_spanned(
                    o,
                    "function options: throws, js_value, js_value_rec, consume(argument, ...), module = Type, name = \"name\"",
                ));
            }
        }
    }
    let asynchronous = f.sig.asyncness.is_some();
    let anchored = asynchronous || !consumed.is_empty();
    let name = &f.sig.ident;
    if member.is_empty() {
        member = name.to_string();
    }
    let abi_name = if namespaced {
        format!("__ocamljs_{module}_{member}")
    } else {
        name.to_string()
    };
    let export = format_ident!("__ocamljs_fn_{}", abi_name);
    let mut prefix = "__ocamljs_internal_".to_string();
    while f.sig.inputs.iter().any(|a| matches!(a, FnArg::Typed(a) if matches!(&*a.pat, Pat::Ident(p) if p.ident.to_string().starts_with(&prefix)))) { prefix.push('_'); }
    let mut params = vec![];
    let mut abi = vec![];
    let mut decode = vec![];
    let mut adopted_inputs = vec![];
    let mut reservations = vec![];
    let mut transfers = vec![];
    let mut anchors = vec![];
    let mut args = vec![];
    let mut borrows = vec![];
    for (index, arg) in f.sig.inputs.iter().enumerate() {
        let FnArg::Typed(a) = arg else {
            return Err(syn::Error::new_spanned(arg, "export free functions"));
        };
        let Pat::Ident(p) = &*a.pat else {
            return Err(syn::Error::new_spanned(&a.pat, "use a named ABI argument"));
        };
        let n = &p.ident;
        let t = &a.ty;
        let moving = consumed.remove(&n.to_string());
        if matches!(&**t,Type::Tuple(t) if t.elems.is_empty()) {
            return Err(syn::Error::new_spanned(
                t,
                "omit explicit unit parameters from exported functions",
            ));
        }
        params
            .push(json!({"name":n.to_string(),"ty":ty(t)?,"mode":if moving {"move"}else{"value"}}));
        let anchor = format_ident!("{prefix}anchor_{index}");
        if let Type::Reference(r) = &**t {
            if moving {
                return Err(syn::Error::new_spanned(
                    t,
                    "consume requires a by-value opaque argument",
                ));
            }
            if r.lifetime.is_some() {
                return Err(syn::Error::new_spanned(
                    r,
                    "explicit lifetimes are not supported at the ABI",
                ));
            }
            let rt = &r.elem;
            let string = type_name(rt) == "str";
            let bytes = matches!(&**rt,Type::Slice(s) if type_name(&s.elem)=="u8");
            if r.mutability.is_some() && (string || bytes) {
                return Err(syn::Error::new_spanned(
                    r,
                    "mutable strings/slices are unsupported; use an opaque buffer",
                ));
            }
            if matches!(&**rt, Type::Slice(_)) && r.mutability.is_some() {
                return Err(syn::Error::new_spanned(
                    r,
                    "mutable numeric slices are unsupported",
                ));
            }
            if r.mutability.is_none() {
                let length = format_ident!("{prefix}length_{index}");
                let capacity = format_ident!("{prefix}capacity_{index}");
                let adopted = format_ident!("{prefix}adopted_{index}");
                abi.push(quote!(#n:usize));
                abi.push(quote!(#length:<#rt as ::ocamljs_bindgen::host::Borrowable>::A1));
                abi.push(quote!(#capacity:<#rt as ::ocamljs_bindgen::host::Borrowable>::A2));
                adopted_inputs.push(quote!(let #adopted = unsafe { <#rt as ::ocamljs_bindgen::host::Borrowable>::adopt(#n,#length,#capacity) };));
                if anchored {
                    reservations.push(quote!(let mut #anchor = unsafe { <#rt as ::ocamljs_bindgen::host::Borrowable>::anchor(#n,#length,#capacity,#adopted)? };));
                    anchors.push(anchor.clone());
                    args.push(quote!(::std::ops::Deref::deref(&#anchor)));
                } else {
                    args.push(quote!(#n));
                    borrows.push((
                        quote!(shared_adopted),
                        rt.clone(),
                        n.clone(),
                        Some((length, capacity, adopted)),
                    ));
                }
            } else {
                abi.push(quote!(#n:u32));
                if anchored {
                    let create = if string {
                        decode.push(quote!(::ocamljs_bindgen::with_bytes(#n,|b| unsafe { ::ocamljs_bindgen::boundary_str(b) }.map(|_|()))??;));
                        quote!(::ocamljs_bindgen::scoped::BytesAnchor::new(#n)?)
                    } else {
                        quote!(unsafe { ::ocamljs_bindgen::scoped::MutAnchor::<#rt>::new(#n)? })
                    };
                    reservations.push(quote!(let mut #anchor = #create;));
                    args.push(if string {
                        quote!(unsafe { std::str::from_utf8_unchecked(#anchor.get()) })
                    } else {
                        quote!(#anchor.get_mut())
                    });
                    anchors.push(anchor);
                } else {
                    args.push(quote!(#n));
                    borrows.push((
                        if string {
                            quote!(str)
                        } else {
                            quote!(::ocamljs_bindgen::with_mut)
                        },
                        rt.clone(),
                        n.clone(),
                        None,
                    ));
                }
            }
        } else if moving {
            abi.push(quote!(#n:u32));
            reservations
                .push(quote!(let #anchor = ::ocamljs_bindgen::scoped::MoveAnchor::<#t>::new(#n)?;));
            transfers
                .push(quote!(let #n = #anchor.take(); ::ocamljs_bindgen::objects::record_move();));
            args.push(quote!(#n));
        } else if type_name(t) == "Callback" {
            abi.push(quote!(#n:u32));
            decode.push(quote!(let #n:#t=::ocamljs_bindgen::Callback::acquire(#n)?;));
            args.push(quote!(#n));
        } else if matches!(&**t, Type::Tuple(_)) && tuple_lanes(t, &mut Vec::new()) {
            let mut types = Vec::new();
            tuple_lanes(t, &mut types);
            let values = types
                .iter()
                .enumerate()
                .map(|(j, _)| format_ident!("{prefix}tuple_{index}_{j}"))
                .collect::<Vec<_>>();
            for (t, v) in types.iter().zip(&values) {
                let at = scalar_abi(t);
                abi.push(quote!(#v:#at));
            }
            let value = tuple_value(t, &mut values.into_iter());
            decode.push(quote!(let #n:#t=#value;));
            args.push(quote!(#n));
        } else if scalar(t) {
            let at = scalar_abi(t);
            abi.push(quote!(#n:#at));
            if type_name(t) == "bool" {
                decode.push(quote!(let #n=match #n {0=>false,1=>true,_=>return Err(::ocamljs_bindgen::BoundaryError("invalid bool"))};));
            }
            args.push(quote!(#n));
        } else {
            let values = (0..8)
                .map(|i| format_ident!("{prefix}arg_{index}_{i}"))
                .collect::<Vec<_>>();
            let lanes = (0..8).map(|i| format_ident!("A{i}")).collect::<Vec<_>>();
            for (value, lane) in values.iter().zip(&lanes) {
                abi.push(quote!(#value:<<#t as ::ocamljs_bindgen::Decode>::Abi as ::ocamljs_bindgen::abi::Argument<#t>>::#lane));
            }
            let adopted = format_ident!("{prefix}adopted_{index}");
            adopted_inputs.push(quote!(let #adopted = unsafe { <<#t as ::ocamljs_bindgen::Decode>::Abi as ::ocamljs_bindgen::abi::Argument<#t>>::adopt(#(#values),*) };));
            decode.push(quote!(let #n:#t=unsafe { <<#t as ::ocamljs_bindgen::Decode>::Abi as ::ocamljs_bindgen::abi::Argument<#t>>::lift(#adopted,#(#values),*)? };));
            args.push(quote!(#n));
        }
    }
    if !consumed.is_empty() {
        return Err(syn::Error::new_spanned(
            &f.sig,
            "consume names must refer to parameters",
        ));
    }
    let ret: Type = match &f.sig.output {
        ReturnType::Default => syn::parse_quote!(()),
        ReturnType::Type(_, t) => (**t).clone(),
    };
    if matches!(&ret, Type::Reference(_)) {
        return Err(syn::Error::new_spanned(
            &ret,
            "returned references are unsupported; return an owned value or opaque view",
        ));
    }
    if asynchronous && type_name(&ret) == "Encoded" {
        return Err(syn::Error::new_spanned(
            &ret,
            "Encoded requires a synchronous export",
        ));
    }
    let reservation = format_ident!("{prefix}task_reservation");
    let move_reset = if transfers.is_empty() {
        quote!()
    } else {
        quote!(::ocamljs_bindgen::objects::reset_moves();)
    };
    let (at, body) = if asynchronous {
        (
            quote!(u32),
            quote!({
                let #reservation = ::ocamljs_bindgen::tasks::reserve_task()?;
                #(#decode)* #(#reservations)* #(#transfers)*
                #reservation.spawn_completion((#(#anchors,)*), move |(#(#anchors,)*)| {
                    Box::pin(async move { let value = self::#name(#(#args),*).await; ::ocamljs_bindgen::abi::complete(value) })
                })?
            }),
        )
    } else {
        let mut call = quote!(Ok(self::#name(#(#args),*)));
        for (helper, rt, n, adopted) in borrows.into_iter().rev() {
            let scratch = format_ident!("{prefix}bytes");
            call = if let Some((length, capacity, guard)) = adopted {
                quote!(unsafe { <#rt as ::ocamljs_bindgen::host::Borrowable>::with_shared(#n,#length,#capacity,#guard,|#n| {#call}) }?)
            } else if helper.to_string() == "str" {
                quote!(::ocamljs_bindgen::with_bytes(#n,|#scratch| { let #n=unsafe { ::ocamljs_bindgen::boundary_str(#scratch) }?; #call })?)
            } else if helper.to_string() == "bytes" {
                quote!(::ocamljs_bindgen::with_bytes(#n,|#n| {#call})?)
            } else {
                quote!(#helper::<#rt,_>(#n,|#n| {#call})?)
            };
        }
        let call = quote!((#call)?);
        let (at, result) = if scalar(&ret) {
            (
                scalar_abi(&ret),
                if type_name(&ret) == "bool" {
                    quote!(u32::from(#call))
                } else {
                    call
                },
            )
        } else {
            (
                quote!(u32),
                quote!(::ocamljs_bindgen::abi::lower_output(#call)?),
            )
        };
        (
            at,
            quote!({ #(#decode)* #(#reservations)* #(#transfers)* #result }),
        )
    };
    let mut description = json!({"kind":"function","name":abi_name,"module":module,"member":member,"params":params,"result":result_ty(&ret)?,"throws":throws,"asynchronous":asynchronous,"docs":docs(&f.attrs)});
    if type_name(&ret) == "Encoded" {
        description["buffered"] = json!(true);
    }
    if !selector.is_empty() {
        description["js_selector"] = json!(selector);
    }
    if js_value {
        description["js_value"] = json!(true);
    }
    if js_value_rec {
        description["js_value_rec"] = json!(true);
    }
    fn reference_builtin(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Object(v) => {
                v.get("name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|n| {
                        matches!(
                            n,
                            "Callback" | "Promise" | "HostError" | "JsString" | "Uint8ClampedArray"
                        )
                    })
                    || v.values().any(reference_builtin)
            }
            serde_json::Value::Array(v) => v.iter().any(reference_builtin),
            _ => false,
        }
    }
    let support = if reference_builtin(&description) {
        let prefix = format!("__ocamljs_ref_{abi_name}");
        description["reference_support"] = json!(prefix);
        host::support(&prefix)
    } else {
        quote!()
    };
    let description = if let Some(scope) = scope {
        scope.describe(description, false)
    } else {
        description
    };
    let md = section(&abi_name, description);
    // Only an exported async declaration roots the task polling ABI. Global
    // no_mangle hooks would keep task vtables alive in synchronous modules.
    let task_support = if asynchronous {
        let poll = format_ident!("__ocamljs_task_poll_for_{}", abi_name);
        let cancel = format_ident!("__ocamljs_task_cancel_for_{}", abi_name);
        quote! {
            #[doc(hidden)] #[unsafe(no_mangle)] pub extern "C" fn #poll(id:u32)->u32 {
                ::ocamljs_bindgen::tasks::__ocamljs_task_poll(id)
            }
            #[doc(hidden)] #[unsafe(no_mangle)] pub extern "C" fn #cancel(id:u32)->u32 {
                ::ocamljs_bindgen::tasks::__ocamljs_task_cancel(id)
            }
        }
    } else {
        quote!()
    };
    let cfg = f
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        .collect::<Vec<_>>();
    Ok(
        quote!(#f #(#cfg)* #[allow(improper_ctypes_definitions, unused_variables)] #[unsafe(no_mangle)] pub unsafe extern "C" fn #export(#(#abi),*)->#at { let result=(||->::ocamljs_bindgen::BoundaryResult<#at>{ #move_reset #(#adopted_inputs)* Ok(unsafe { #body }) })(); ::ocamljs_bindgen::boundary(result) } #(#cfg)* #md #(#cfg)* const _: () = { #support #task_support };),
    )
}

#[cfg(test)]
mod signature_tests {
    use super::export_function;
    #[test]
    fn rejects_signatures_whose_ownership_cannot_cross_the_abi() {
        for (options, source, diagnostic) in [
            (
                "",
                "pub async fn bad<'a>(x: &'a Item) -> u32 { 0 }",
                "safe and concrete",
            ),
            (
                "",
                "pub async fn bad(x: &Item) -> &Item { x }",
                "returned references",
            ),
            (
                "",
                "pub async fn bad(x: &mut [u8]) {}",
                "mutable strings/slices",
            ),
            ("", "pub async fn bad<T>(x: T) {}", "safe and concrete"),
            (
                "consume(missing)",
                "pub fn bad(x: Item) {}",
                "consume names",
            ),
            (
                "consume(x,x)",
                "pub fn bad(x: Item) {}",
                "duplicate consuming",
            ),
        ] {
            let error =
                export_function(options.into(), syn::parse_str(source).unwrap()).unwrap_err();
            assert!(error.to_string().contains(diagnostic), "{source}: {error}");
        }
    }
}

#[cfg(test)]
mod naming_tests {
    use super::*;
    #[test]
    fn serde_enum_names_decode_literals_and_handle_acronyms() {
        let name: syn::Ident = syn::parse_quote!(HTTPServer);
        let snake: Attribute = syn::parse_quote!(#[serde(rename_all = "snake_case")]);
        let camel: Attribute = syn::parse_quote!(#[serde(rename_all = "camelCase")]);
        let explicit: Attribute = syn::parse_quote!(#[serde(rename = r#"quote \" raw"#)]);
        assert_eq!(enum_name(&name, &[], &[snake]).unwrap(), "h_t_t_p_server");
        assert_eq!(enum_name(&name, &[], &[camel]).unwrap(), "hTTPServer");
        assert_eq!(
            enum_name(&name, &[explicit], &[]).unwrap(),
            r#"quote \" raw"#
        );
    }
    #[test]
    fn serde_directional_enum_names_are_explicitly_rejected() {
        let name: syn::Ident = syn::parse_quote!(Value);
        let directional: Attribute =
            syn::parse_quote!(#[serde(rename(serialize="out",deserialize="in"))]);
        assert!(
            enum_name(&name, &[directional], &[])
                .unwrap_err()
                .to_string()
                .contains("directional")
        );
    }
}

#[cfg(test)]
mod collection_tests {
    use super::*;
    #[test]
    fn marker_traits_cannot_certify_custom_equality() {
        let marker: Attribute = syn::parse_quote!(#[derive(Eq,Ord)]);
        assert_eq!(structural_key_traits(&[marker], false), (false, false));
        let structural: Attribute = syn::parse_quote!(#[derive(PartialEq,Eq,PartialOrd,Ord)]);
        assert_eq!(structural_key_traits(&[structural], false), (true, true));
        assert_eq!(structural_key_traits(&[], true), (true, true));
    }
    #[test]
    fn dictionary_policy_only_projects_string_maps() {
        let mut schema = json!({"name":"BTreeMap","args":[{"name":"String"},{"name":"HashMap","args":[{"name":"u32"},{"name":"String"}]}]});
        map_policy(&mut schema, "dict");
        assert_eq!(schema["map"], "dict");
        assert!(schema["args"][1]["map"].is_null());
    }
}

/// Derive a typed encoding projection from an owned record or enum declaration.
/// Projected fields must encode as their corresponding owned types; record field
/// order and enum tags come from that same declaration. This permits borrowed
/// storage without duplicate wire layouts or omitted publication fields.
#[proc_macro_derive(EncodeProjection)]
pub fn encode_projection(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let expand = || -> syn::Result<proc_macro2::TokenStream> {
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                "projection records must be concrete",
            ));
        }
        let name = &input.ident;
        let visibility = &input.vis;
        let projected = format_ident!("{}Projection", name);
        let mut parameters = Vec::new();
        let mut types = Vec::new();
        let mut project_fields =
            |fields: &Fields, prefix: proc_macro2::TokenStream, public: bool| {
                let params: Vec<_> = fields
                    .iter()
                    .map(|field| {
                        let p = format_ident!("P{}", parameters.len());
                        parameters.push(p.clone());
                        types.push(field.ty.clone());
                        p
                    })
                    .collect();
                let names: Vec<_> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        field
                            .ident
                            .clone()
                            .unwrap_or_else(|| format_ident!("field_{i}"))
                    })
                    .collect();
                let visibility = if public { quote!(pub) } else { quote!() };
                let (declarations, pattern) = match fields {
                    Fields::Named(_) => (
                        quote!({#(#visibility #names:#params),*}),
                        quote!(#prefix {#(#names),*}),
                    ),
                    Fields::Unnamed(_) => (
                        quote!((#(#visibility #params),*)),
                        quote!(#prefix(#(#names),*)),
                    ),
                    Fields::Unit => (quote!(), quote!(#prefix)),
                };
                let encode = quote!(#(::ocamljs_bindgen::EncodeAs::encode_as(#names,out);)*);
                (declarations, pattern, encode)
            };
        let (declaration, encoding) = match &input.data {
            syn::Data::Struct(data) => {
                let (declarations, pattern, encode) =
                    project_fields(&data.fields, quote!(Self), true);
                let terminator = if matches!(data.fields, Fields::Named(_)) {
                    quote!()
                } else {
                    quote!(;)
                };
                (
                    quote!(struct #projected<#(#parameters),*> #declarations #terminator),
                    quote!(let #pattern=self; #encode),
                )
            }
            syn::Data::Enum(data) => {
                let mut variants = Vec::new();
                let mut arms = Vec::new();
                for (tag, variant) in data.variants.iter().enumerate() {
                    let id = &variant.ident;
                    let (declarations, pattern, encode) =
                        project_fields(&variant.fields, quote!(Self::#id), false);
                    variants.push(quote!(#id #declarations));
                    let tag = tag as u32;
                    arms.push(
                        quote!(#pattern => {::ocamljs_bindgen::Encode::encode(#tag,out); #encode}),
                    );
                }
                (
                    quote!(enum #projected<#(#parameters),*> {#(#variants),*}),
                    quote!(match self {#(#arms),*}),
                )
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "projection requires a record or enum",
                ));
            }
        };
        Ok(quote! {
            #visibility #declaration
            impl<#(#parameters: ::ocamljs_bindgen::EncodeAs<#types>),*> ::ocamljs_bindgen::EncodeAs<#name> for #projected<#(#parameters),*> {
                fn encode_as(self, out: &mut ::ocamljs_bindgen::Encoder) {
                    out.nested(|out| { #encoding });
                }
            }
        })
    };
    expand()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(test)]
mod projection_schema_tests {
    use super::*;
    #[test]
    fn eager_result_exposes_owned_schema_and_rejects_flat_abi_mismatch() {
        let projected: Type = syn::parse_quote!(Encoded<Result<DispatchUpdate, EngineError>>);
        let owned: Type = syn::parse_quote!(Result<DispatchUpdate, EngineError>);
        assert_eq!(result_ty(&projected).unwrap(), ty(&owned).unwrap());
        assert!(result_ty(&syn::parse_quote!(Encoded<u32>)).is_err());
        assert!(
            export_function(
                String::new(),
                syn::parse_quote!(
                    pub async fn bad() -> Encoded<Result<Record, Error>> {
                        todo!()
                    }
                )
            )
            .is_err()
        );
    }
}
