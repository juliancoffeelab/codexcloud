//! Host-only compiler; emitted expressions borrow literals instead of constructing collections.
use crate::data::{ContentError, Rules};
use databake::Bake;
use std::path::Path;

/// Validate the catalog with the runtime loader and emit its fully resolved static representation.
pub fn compile(root: &Path) -> Result<String, ContentError> {
    let rules = Rules::load_from(root)?;
    let expression = rules.bake(&Default::default());
    Ok(format!(
        "pub static BAKED_RULES: hex_command_catalog::data::Rules = {expression};\n"
    ))
}
