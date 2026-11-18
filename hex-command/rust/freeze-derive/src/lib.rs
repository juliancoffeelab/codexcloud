//! Derive borrowed battle frames from their owned runtime state.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

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
        let visibility = &input.vis;
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
            #visibility struct #borrowed<'a> { #(#declarations,)* }
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
