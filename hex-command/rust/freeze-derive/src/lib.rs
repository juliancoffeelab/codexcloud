//! Derive a static catalog representation without changing the owned wire model.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Index, parse_macro_input};

#[proc_macro_derive(Freeze)]
pub fn freeze(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let frozen = format_ident!("Frozen{name}");
    if !input.generics.params.is_empty() {
        return syn::Error::new_spanned(input.generics, "catalog models must have concrete fields")
            .to_compile_error()
            .into();
    }
    let convert = |fields: &Fields| {
        let declarations = fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let ty = &field.ty;
                let attributes = field.attrs.iter().filter(|a| a.path().is_ident("cfg"));
                if let Some(id) = &field.ident {
                    quote! { #(#attributes)* pub #id: <#ty as crate::frozen::Freeze>::Frozen }
                } else {
                    let _ = i;
                    quote! { #(#attributes)* pub <#ty as crate::frozen::Freeze>::Frozen }
                }
            })
            .collect::<Vec<_>>();
        let bindings = fields
            .iter()
            .enumerate()
            .map(|(i, f)| f.ident.clone().unwrap_or_else(|| format_ident!("field{i}")))
            .collect::<Vec<_>>();
        let freeze = fields
            .iter()
            .zip(&bindings)
            .map(|(f, b)| {
                let ty = &f.ty;
                quote! {<#ty as crate::frozen::Freeze>::freeze(#b)}
            })
            .collect::<Vec<_>>();
        let thaw = fields
            .iter()
            .zip(&bindings)
            .map(|(f, b)| {
                let ty = &f.ty;
                quote! {<#ty as crate::frozen::Freeze>::thaw(#b)}
            })
            .collect::<Vec<_>>();
        (declarations, bindings, freeze, thaw)
    };
    let (declaration, freeze, thaw) = match &input.data {
        Data::Struct(data) => {
            let (fields, _, _, _) = convert(&data.fields);
            let freeze = data.fields.iter().enumerate().map(|(i, f)| {
                let ty = &f.ty;
                let member = f.ident.as_ref().map(|id| quote! {#id}).unwrap_or_else(|| {
                    let i = Index::from(i);
                    quote! {#i}
                });
                let expr = quote! {<#ty as crate::frozen::Freeze>::freeze(&self.#member)};
                let attributes = f.attrs.iter().filter(|a| a.path().is_ident("cfg"));
                if f.ident.is_some() {
                    quote! {#(#attributes)* #member:#expr}
                } else {
                    expr
                }
            });
            let thaw = data.fields.iter().enumerate().map(|(i, f)| {
                let ty = &f.ty;
                let member = f.ident.as_ref().map(|id| quote! {#id}).unwrap_or_else(|| {
                    let i = Index::from(i);
                    quote! {#i}
                });
                let expr = quote! {<#ty as crate::frozen::Freeze>::thaw(&value.#member)};
                let attributes = f.attrs.iter().filter(|a| a.path().is_ident("cfg"));
                if f.ident.is_some() {
                    quote! {#(#attributes)* #member:#expr}
                } else {
                    expr
                }
            });
            match data.fields {
                Fields::Named(_) => (
                    quote! {pub struct #frozen{#(#fields,)*}},
                    quote! {#frozen{#(#freeze,)*}},
                    quote! {Self{#(#thaw,)*}},
                ),
                Fields::Unnamed(_) => (
                    quote! {pub struct #frozen(#(#fields,)*);},
                    quote! {#frozen(#(#freeze,)*)},
                    quote! {Self(#(#thaw,)*)},
                ),
                Fields::Unit => (
                    quote! {pub struct #frozen;},
                    quote! {#frozen},
                    quote! {Self},
                ),
            }
        }
        Data::Enum(data) => {
            let mut variants = Vec::new();
            let mut freeze = Vec::new();
            let mut thaw = Vec::new();
            for variant in &data.variants {
                let id = &variant.ident;
                let (_, bindings, fr, th) = convert(&variant.fields);
                let types = variant.fields.iter().map(|f| {
                    let ty = &f.ty;
                    quote! {<#ty as crate::frozen::Freeze>::Frozen}
                });
                match &variant.fields {
                    Fields::Named(_) => {
                        variants.push(quote! {#id{#(#bindings:#types,)*}});
                        freeze.push(
                            quote! {Self::#id{#(#bindings,)*}=>#frozen::#id{#(#bindings:#fr,)*}},
                        );
                        thaw.push(
                            quote! {#frozen::#id{#(#bindings,)*}=>Self::#id{#(#bindings:#th,)*}},
                        );
                    }
                    Fields::Unnamed(_) => {
                        variants.push(quote! {#id(#(#types,)*)});
                        freeze.push(quote! {Self::#id(#(#bindings,)*)=>#frozen::#id(#(#fr,)*)});
                        thaw.push(quote! {#frozen::#id(#(#bindings,)*)=>Self::#id(#(#th,)*)});
                    }
                    Fields::Unit => {
                        variants.push(quote! {#id});
                        freeze.push(quote! {Self::#id=>#frozen::#id});
                        thaw.push(quote! {#frozen::#id=>Self::#id});
                    }
                }
            }
            (
                quote! {pub enum #frozen{#(#variants,)*}},
                quote! {match self{#(#freeze,)*}},
                quote! {match value{#(#thaw,)*}},
            )
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "catalog unions are unsupported")
                .to_compile_error()
                .into();
        }
    };
    quote! {
        #[derive(Clone,Debug)]
        #[cfg_attr(feature="bake",derive(databake::Bake))]
        #[cfg_attr(feature="bake",databake(path=hex_command_catalog::data))]
        #declaration
        impl crate::frozen::Freeze for #name{
            type Frozen=#frozen;
            #[cfg(feature="bake")]
            fn freeze(&self)->Self::Frozen{#freeze}
            fn thaw(value:&Self::Frozen)->Self{#thaw}
        }
    }
    .into()
}

/// Borrow a wire frame's fields from its authoritative owned state without cloning.
#[proc_macro_derive(BorrowFields, attributes(borrow_from))]
pub fn borrow_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expand = || -> syn::Result<proc_macro2::TokenStream> {
        if !input.generics.params.is_empty() {
            return Err(syn::Error::new_spanned(
                &input.generics,
                "borrowed frames require concrete fields",
            ));
        }
        let source: syn::Type = input
            .attrs
            .iter()
            .find(|a| a.path().is_ident("borrow_from"))
            .ok_or_else(|| {
                syn::Error::new_spanned(&input.ident, "expected #[borrow_from(Source)]")
            })?
            .parse_args()?;
        let Data::Struct(data) = &input.data else {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "borrowed frames require a struct",
            ));
        };
        let Fields::Named(fields) = &data.fields else {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "borrowed frames require named fields",
            ));
        };
        let borrowed = format_ident!("Borrowed{}", input.ident);
        let attrs = input.attrs.iter().filter(|a| a.path().is_ident("serde"));
        let declarations = fields.named.iter().map(|field| {
            let id = &field.ident;
            let ty = &field.ty;
            let attrs = field
                .attrs
                .iter()
                .filter(|a| a.path().is_ident("serde") || a.path().is_ident("cfg"));
            quote! { #(#attrs)* #id: &'a #ty }
        });
        let bindings = fields.named.iter().map(|field| {
            let id = &field.ident;
            let attrs = field.attrs.iter().filter(|a| a.path().is_ident("cfg"));
            quote! { #(#attrs)* #id: &state.#id }
        });
        let name = &input.ident;
        let cloned = fields.named.iter().map(|field| {
            let id = &field.ident;
            let attrs = field.attrs.iter().filter(|a| a.path().is_ident("cfg"));
            quote! { #(#attrs)* #id: state.#id.clone() }
        });
        Ok(quote! {
            #[derive(serde::Serialize)]
            #(#attrs)*
            pub(crate) struct #borrowed<'a> { #(#declarations,)* }
            impl From<&#source> for #name {
                fn from(state: &#source) -> Self { Self { #(#cloned,)* } }
            }
            impl<'a> From<&'a #source> for #borrowed<'a> {
                fn from(state: &'a #source) -> Self { Self { #(#bindings,)* } }
            }
        })
    };
    expand()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
