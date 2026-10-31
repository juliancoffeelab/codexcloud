//! Host-only compiler; emitted expressions borrow literals instead of constructing collections.
use crate::data::{ContentError, Rules};
use databake::Bake;
use std::path::Path;

/// Validate the catalog and emit runtime fields, retaining simulation-only fields
/// in native consistency checks while excluding them from the browser catalog.
pub fn compile(root: &Path) -> Result<String, ContentError> {
    let Rules {
        content_fingerprint,
        recruitment,
        rosters,
        terrain_names,
        terrain_types,
        unit_log_names,
        weapon_log_names,
        terrain_log_labels,
        terrain_definitions,
        race_definitions,
        map_definitions,
        units,
        movement_classes,
        route_classes,
        types,
        terrains,
        maps,
        campaigns,
        arena,
        campaign_defaults,
        arena_maps,
    } = Rules::load_from(root)?;
    let context = Default::default();
    let mut fields = String::new();
    macro_rules! emit {
        ($attribute:literal; $($field:ident),* $(,)?) => {
            $(fields.push_str(&format!("{} {}: {},", $attribute, stringify!($field), $field.bake(&context)));)*
        };
    }
    emit!("";
        content_fingerprint,
        recruitment,
        rosters,
        terrain_names,
        terrain_types,
        unit_log_names,
        weapon_log_names,
        terrain_log_labels,
        terrain_definitions,
        race_definitions,
        map_definitions,
        units,
        movement_classes,
        route_classes,
        types,
        terrains,
        maps,
        campaigns,
    );
    emit!("#[cfg(not(target_arch = \"wasm32\"))]"; arena, campaign_defaults, arena_maps);
    Ok(format!(
        "pub static BAKED_RULES: hex_command_catalog::data::Rules = hex_command_catalog::data::Rules {{ {fields} }};\n"
    ))
}
