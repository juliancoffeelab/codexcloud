//! Typed host declarations. Imports catch JS exceptions before returning to Rust.
use super::*;
pub(super) fn support(prefix: &str) -> proc_macro2::TokenStream {
    let alloc = format_ident!("{prefix}_alloc");
    let release = format_ident!("{prefix}_release");
    let live = format_ident!("{prefix}_live");
    let output = format_ident!("{prefix}_output");
    let settle = format_ident!("{prefix}_settle");
    quote! {
        #[doc(hidden)] #[unsafe(no_mangle)] pub extern "C" fn #alloc()->u32{::ocamljs_bindgen::host::allocate()}
        #[doc(hidden)] #[unsafe(no_mangle)] pub extern "C" fn #release(id:u32){::ocamljs_bindgen::host::release(id)}
        #[doc(hidden)] #[unsafe(no_mangle)] pub extern "C" fn #live()->u32{::ocamljs_bindgen::host::live()}
        #[doc(hidden)] #[unsafe(no_mangle)] pub extern "C" fn #output(output:u32,id:u32)->u32{::ocamljs_bindgen::host::output(output,id)}
        #[doc(hidden)] #[unsafe(no_mangle)] pub unsafe extern "C" fn #settle(channel:u32,value:u32,ok:u32){unsafe { ::ocamljs_bindgen::host::settle(channel,value,ok) }}
    }
}

pub(super) fn host_type(
    s: syn::ItemStruct,
    scope: Option<&groups::Scope>,
) -> syn::Result<proc_macro2::TokenStream> {
    if !matches!(s.fields, Fields::Unit) || !s.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &s,
            "host types are concrete unit declarations",
        ));
    }
    let name = &s.ident;
    let vis = &s.vis;
    let attrs = &s.attrs;
    let mut description = json!({"kind":"host","name":name.to_string(),"docs":docs(attrs)});
    let abi_name = scope
        .map(|s| {
            s.describe(description.clone(), true)["name"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .unwrap_or_else(|| name.to_string());
    let prefix = format!("__ocamljs_ref_{abi_name}");
    description["reference_support"] = json!(prefix);
    let support = support(&prefix);
    let md = described_section(&name.to_string(), description, "", scope);
    Ok(quote! {
        #(#attrs)* #vis struct #name(::ocamljs_bindgen::host::Ref);
        impl Clone for #name { fn clone(&self)->Self { Self(self.0.clone()) } }
        impl ::ocamljs_bindgen::host::Value for #name {
            fn reference(&self)->&::ocamljs_bindgen::host::Ref { &self.0 }
            fn into_reference(self)->::ocamljs_bindgen::host::Ref { self.0 }
            fn from_reference(value: ::ocamljs_bindgen::host::Ref)->Self { Self(value) }
        }
        impl ::ocamljs_bindgen::Encode for #name {
            type Return = ::ocamljs_bindgen::host::Direct<Self>;
            fn encode(self,out:&mut ::ocamljs_bindgen::Encoder){::ocamljs_bindgen::host::encode(self,out);}
        }
        impl ::ocamljs_bindgen::Decode for #name {
            type Abi = ::ocamljs_bindgen::host::Direct<Self>;
            fn decode(input:&mut ::ocamljs_bindgen::Decoder<'_>)->::ocamljs_bindgen::BoundaryResult<Self>{::ocamljs_bindgen::host::decode(input)}
        }
        impl ::ocamljs_bindgen::host::Borrowable for #name {
            type A1=(); type A2=(); type Adopted=();
            type Anchor = ::ocamljs_bindgen::host::Anchor<Self>;
            unsafe fn anchor(id:usize,_:(),_:(),_:())->::ocamljs_bindgen::BoundaryResult<Self::Anchor>{::ocamljs_bindgen::host::anchor(id as u32)}
            unsafe fn with_shared<R>(id:usize,_:(),_:(),_:(),f:impl FnOnce(&Self)->R)->::ocamljs_bindgen::BoundaryResult<R>{::ocamljs_bindgen::host::with_shared(id as u32,f)}
        }
        #md
        #support
    })
}

pub(super) fn imports(
    options: &str,
    block: syn::ItemForeignMod,
) -> syn::Result<proc_macro2::TokenStream> {
    use syn::parse::Parser;
    let parse = |s: &str| {
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated.parse_str(s)
    };
    let defaults = parse(options)?;
    let block_cfg = block
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr"))
        .collect::<Vec<_>>();
    let mut output = proc_macro2::TokenStream::new();
    for item in block.items {
        let syn::ForeignItem::Fn(f) = item else {
            return Err(syn::Error::new_spanned(
                item,
                "host blocks contain function declarations",
            ));
        };
        if !f.sig.generics.params.is_empty()
            || f.sig.variadic.is_some()
            || f.sig.asyncness.is_some()
        {
            return Err(syn::Error::new_spanned(
                &f.sig,
                "host imports are concrete synchronous calls; return Promise<T> for asynchronous JS",
            ));
        }
        let name = &f.sig.ident;
        let abi_name = format!("__ocamljs_host_{name}");
        let raw = format_ident!("__ocamljs_import_{name}");
        let mut js_name = name.to_string();
        let mut namespace = String::new();
        let mut js_module = String::new();
        let mut operation = "function";
        let mut opts = defaults.clone();
        for a in f.attrs.iter().filter(|a| groups::binding_attribute(a)) {
            if let syn::Meta::List(l) = &a.meta {
                opts.extend(parse(&l.tokens.to_string())?);
            }
        }
        for option in opts {
            match &option {
                syn::Meta::NameValue(v)
                    if v.path.is_ident("js_name")
                        || v.path.is_ident("js_namespace")
                        || v.path.is_ident("js_module") =>
                {
                    let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(value),
                        ..
                    }) = &v.value
                    else {
                        return Err(syn::Error::new_spanned(v, "expected string literal"));
                    };
                    if v.path.is_ident("js_name") {
                        js_name = value.value();
                    } else if v.path.is_ident("js_module") {
                        js_module = value.value();
                    } else {
                        namespace = value.value();
                    }
                }
                syn::Meta::Path(p)
                    if [
                        "method",
                        "getter",
                        "setter",
                        "call",
                        "constructor",
                        "indexing_getter",
                    ]
                    .iter()
                    .any(|n| p.is_ident(n)) =>
                {
                    if operation != "function" {
                        return Err(syn::Error::new_spanned(option, "choose one host operation"));
                    }
                    operation = if p.is_ident("method") {
                        "method"
                    } else if p.is_ident("getter") {
                        "getter"
                    } else if p.is_ident("setter") {
                        "setter"
                    } else if p.is_ident("call") {
                        "call"
                    } else if p.is_ident("constructor") {
                        "constructor"
                    } else {
                        "indexing_getter"
                    };
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        option,
                        "host options: js_name, js_namespace, method, getter, setter, call, constructor, indexing_getter",
                    ));
                }
            }
        }
        let ReturnType::Type(_, result) = &f.sig.output else {
            return Err(syn::Error::new_spanned(
                &f.sig,
                "host imports return Result<T, HostError>",
            ));
        };
        let Type::Path(result) = &**result else {
            return Err(syn::Error::new_spanned(
                result,
                "expected Result<T, HostError>",
            ));
        };
        let segment = result.path.segments.last().unwrap();
        let syn::PathArguments::AngleBracketed(a) = &segment.arguments else {
            return Err(syn::Error::new_spanned(
                segment,
                "expected Result<T, HostError>",
            ));
        };
        let types = a
            .args
            .iter()
            .filter_map(|a| {
                if let syn::GenericArgument::Type(t) = a {
                    Some(t)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if segment.ident != "Result" || types.len() != 2 || type_name(types[1]) != "HostError" {
            return Err(syn::Error::new_spanned(
                segment,
                "host imports return Result<T, HostError>",
            ));
        }
        let ret = types[0];
        let mut params = Vec::new();
        let mut arguments = Vec::new();
        let mut raw_params = Vec::new();
        let error_slot = syn::Ident::new("__ocamljs_import_error", proc_macro2::Span::mixed_site());
        let imported_value =
            syn::Ident::new("__ocamljs_import_value", proc_macro2::Span::mixed_site());
        for p in &f.sig.inputs {
            let FnArg::Typed(p) = p else {
                return Err(syn::Error::new_spanned(p, "use an explicit host receiver"));
            };
            let Pat::Ident(n) = &*p.pat else {
                return Err(syn::Error::new_spanned(p, "use named host arguments"));
            };
            let n = &n.ident;
            let t = &p.ty;
            let raw_name = format_ident!("__ocamljs_arg_{}", raw_params.len());
            params.push(json!({"name":n.to_string(),"ty":ty(t)?}));
            if let Type::Reference(r) = &**t {
                if r.mutability.is_some() || r.lifetime.is_some() {
                    return Err(syn::Error::new_spanned(
                        r,
                        "host parameters use shared call-scoped references",
                    ));
                }
                if type_name(&r.elem) == "str" {
                    let len = format_ident!("__ocamljs_arg_{}", raw_params.len() + 1);
                    raw_params.push(quote!(#raw_name:usize));
                    raw_params.push(quote!(#len:usize));
                    arguments.push(quote!(#n.as_ptr() as usize));
                    arguments.push(quote!(#n.len()));
                } else {
                    raw_params.push(quote!(#raw_name:u32));
                    arguments.push(quote!(::ocamljs_bindgen::host::Value::reference(#n).id()));
                }
            } else if scalar(t) && !matches!(&**t, Type::Tuple(_)) {
                let at = scalar_abi(t);
                raw_params.push(quote!(#raw_name:#at));
                arguments.push(if type_name(t) == "bool" {
                    quote!(u32::from(#n))
                } else {
                    quote!(#n)
                });
            } else {
                return Err(syn::Error::new_spanned(
                    t,
                    "host inputs are scalars, &str, or &HostType",
                ));
            }
        }
        let receiver = matches!(
            operation,
            "method" | "getter" | "setter" | "call" | "indexing_getter"
        );
        if receiver
            && params
                .first()
                .is_none_or(|p| p["ty"]["kind"] != "ref" || p["ty"]["inner"]["name"] == "str")
        {
            return Err(syn::Error::new_spanned(
                &f.sig,
                "host operation requires a host receiver as its first argument",
            ));
        }
        if operation == "getter" && params.len() != 1
            || matches!(operation, "setter" | "indexing_getter") && params.len() != 2
        {
            return Err(syn::Error::new_spanned(
                &f.sig,
                "invalid host property arity",
            ));
        }
        let raw_ret = if scalar(ret) {
            scalar_abi(ret)
        } else {
            quote!(u32)
        };
        let value = if scalar(ret) {
            if type_name(ret) == "bool" {
                quote!(#imported_value != 0)
            } else {
                quote!(#imported_value)
            }
        } else {
            quote!(<#ret as ::ocamljs_bindgen::host::Value>::from_reference(unsafe{::ocamljs_bindgen::host::Ref::from_owned(#imported_value)}))
        };
        let vis = &f.vis;
        let sig = &f.sig;
        let attrs = f.attrs.iter().filter(|a| !groups::binding_attribute(a));
        let cfg = block_cfg
            .iter()
            .copied()
            .chain(
                f.attrs
                    .iter()
                    .filter(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr")),
            )
            .collect::<Vec<_>>();
        let prefix = format!("__ocamljs_ref_import_{name}");
        let support = support(&prefix);
        let md = section(
            &abi_name,
            json!({"reference_support":prefix,"kind":"import","name":abi_name,"member":name.to_string(),"js_name":js_name,"js_namespace":namespace,"js_module":js_module,"operation":operation,"params":params,"result":ty(ret)?,"docs":docs(&f.attrs)}),
        );
        output.extend(quote!{
            #(#cfg)* #[cfg(target_arch="wasm32")] #[link(wasm_import_module="ocamljs_host")]
            unsafe extern "C" {#[link_name=#abi_name] fn #raw(#(#raw_params,)*  #error_slot:usize)->#raw_ret;}
            #(#block_cfg)* #[allow(unused_variables)] #(#attrs)* #vis #sig {
                #[cfg(target_arch="wasm32")] {
                    let mut #error_slot=0u32;
                    let #imported_value=unsafe{#raw(#(#arguments,)* &mut #error_slot as *mut u32 as usize)};
                    if #error_slot==0 {Ok(#value)} else {Err(<::ocamljs_bindgen::HostError as ::ocamljs_bindgen::host::Value>::from_reference(unsafe{::ocamljs_bindgen::host::Ref::from_owned(#error_slot)}))}
                }
                #[cfg(not(target_arch="wasm32"))] {panic!("JavaScript imports require packaged Wasm")}
            }
            #(#cfg)* #md
            #(#cfg)* const _: () = { #support };
        });
    }
    Ok(output)
}
