use std::{
    fs,
    path::{Path, PathBuf},
};
fn main() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let path = root.join("../../pixel/art/terrain/road/dead-ends.json");
    println!("cargo:rerun-if-changed={}", path.display());
    let pairs: Vec<(u8, u8)> = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    fs::write(
        output.join("road_caps.rs"),
        format!("const ROAD_DEAD_ENDS: &[(u8,u8)] = &{pairs:?};"),
    )
    .unwrap();
    let mut geometry = format!("const ROAD_DEAD_ENDS: &[(u8,u8)] = &{pairs:?};\n");
    for (name, folder) in [
        ("CASTLE_WALL_VARIANTS", "castle-wall"),
        ("CASTLE_BELL_VARIANTS", "castle-bell"),
    ] {
        let path = root.join(format!("../../pixel/art/terrain/{folder}/variants.json"));
        println!("cargo:rerun-if-changed={}", path.display());
        let variants: Vec<serde_json::Value> =
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        let masks: Vec<u8> = variants
            .iter()
            .map(|v| {
                v["walls"].as_array().unwrap().iter().fold(0, |mask, edge| {
                    mask | (1
                        << ["e", "se", "sw", "w", "nw", "ne"]
                            .iter()
                            .position(|s| Some(*s) == edge.as_str())
                            .unwrap())
                })
            })
            .collect();
        geometry.push_str(&format!("const {name}: &[u8] = &{masks:?};\n"));
    }
    fs::write(output.join("terrain_geometry.rs"), geometry).unwrap();
}
