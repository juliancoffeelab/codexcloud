//! Compile validated authored data into typed Rust construction expressions.
use crate::data::{ContentError, Rules};
use crate::frozen::Freeze;
use databake::Bake;
use std::path::Path;
pub fn compile(root: &Path) -> Result<String, ContentError> {
    let rules = Rules::load_from(root)?;
    let context = Default::default();
    let mut expression: syn::ExprStruct =
        syn::parse2(rules.freeze().bake(&context)).expect("baked Rules structure");
    for field in &mut expression.fields {
        if let syn::Member::Named(name) = &field.member {
            if ["arena", "campaign_defaults", "training"].contains(&name.to_string().as_str()) {
                field
                    .attrs
                    .push(syn::parse_quote!(#[cfg(not(target_arch="wasm32"))]));
            }
        }
    }
    let expression = databake::quote! {#expression};
    let source = format!(
        "pub static BAKED_RULES:hex_command_catalog::data::FrozenRules = {}; pub fn baked_rules()->hex_command_catalog::data::Rules {{ <hex_command_catalog::data::Rules as hex_command_catalog::frozen::Freeze>::thaw(&BAKED_RULES) }}",
        expression
    );
    Ok(prettyplease::unparse(
        &syn::parse_file(&source).expect("generated Rust"),
    ))
}
