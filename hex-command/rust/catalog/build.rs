//! Small artwork vocabulary shared by catalog validation and live snapshot validation.
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("../../pixel/art/terrain/road/dead-ends.json");
    println!("cargo:rerun-if-changed={}", path.display());
    let pairs: Vec<(u8, u8)> = serde_json::from_slice(&std::fs::read(path).expect("road caps"))
        .expect("registered road caps");
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("build output"));
    std::fs::write(
        output.join("road_caps.rs"),
        format!("const ROAD_DEAD_ENDS: &[(u8,u8)] = &{pairs:?};"),
    )
    .expect("road cap table");
}
