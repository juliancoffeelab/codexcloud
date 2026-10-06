//! Regenerate the browser's transfer declarations from Rust, or check for drift.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    let check = arguments.iter().any(|argument| argument == "--check");
    let paths: Vec<_> = arguments
        .iter()
        .filter(|argument| argument.as_str() != "--check")
        .collect();
    if paths.len() > 1 {
        return Err("usage: export-types [--check] [output.ts]".into());
    }
    let destination = paths
        .first()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/game/native-types.ts")
        });
    let source = hex_command_core::protocol::typescript();
    if check {
        if std::fs::read_to_string(&destination)? != source {
            return Err(format!("generated types have drifted: {}", destination.display()).into());
        }
    } else {
        std::fs::write(&destination, source)?;
    }
    Ok(())
}
