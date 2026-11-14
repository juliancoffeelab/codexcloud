//! Compile validated authored data into typed Rust construction expressions.
use crate::data::{ContentError, Rules};
use crate::frozen::Freeze;
use databake::Bake;
use std::collections::HashMap;
use std::path::Path;
use syn::visit_mut::{self, VisitMut};

#[derive(Default)]
struct TileConstants {
    names: HashMap<String, syn::Ident>,
    definitions: Vec<syn::ItemConst>,
}
impl VisitMut for TileConstants {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        if let syn::Expr::Struct(tile) = expression {
            if tile
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == "FrozenTile")
            {
                let key = databake::quote!(#tile).to_string();
                let name = if let Some(name) = self.names.get(&key) {
                    name.clone()
                } else {
                    let name: syn::Ident =
                        syn::parse_str(&format!("BAKED_TILE_{}", self.definitions.len())).unwrap();
                    let ty = &tile.path;
                    self.definitions
                        .push(syn::parse_quote!(const #name: #ty = #tile;));
                    self.names.insert(key, name.clone());
                    name
                };
                *expression = syn::parse_quote!(#name);
                return;
            }
        }
        visit_mut::visit_expr_mut(self, expression);
    }
}
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
    // Maps also occur inside campaign definitions. Intern repeated immutable
    // tiles so large catalogs do not duplicate their full AST in one initializer.
    let mut expression = syn::Expr::Struct(expression);
    let mut tiles = TileConstants::default();
    tiles.visit_expr_mut(&mut expression);
    let definitions = tiles.definitions;
    let constants = databake::quote!(#(#definitions)*);
    let expression = databake::quote! {#expression};
    let source = format!(
        "{} pub static BAKED_RULES:hex_command_catalog::data::FrozenRules = {}; pub fn baked_rules()->hex_command_catalog::data::Rules {{ <hex_command_catalog::data::Rules as hex_command_catalog::frozen::Freeze>::thaw(&BAKED_RULES) }}",
        constants, expression
    );
    Ok(prettyplease::unparse(
        &syn::parse_file(&source).expect("generated Rust"),
    ))
}
