//! Identify the compiled engine independently of mutable authored JSON content.
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn sources(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("Rust source directory") {
        let path = entry.expect("Rust source entry").path();
        if path.is_dir() {
            sources(root, &path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(
                path.strip_prefix(root)
                    .expect("source within crate")
                    .to_owned(),
            );
        }
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=HEX_SIM_PROFILE_KEY");
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("crate directory"));
    let mut files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.lock"),
        PathBuf::from("build.rs"),
        PathBuf::from("../pixel/art/terrain/road/dead-ends.json"),
    ];
    sources(&root, &root.join("src"), &mut files);
    sources(&root, &root.join("catalog/src"), &mut files);
    files.push(PathBuf::from("catalog/Cargo.toml"));
    files.push(PathBuf::from("catalog/build.rs"));
    files.sort();
    let mut hash = Sha256::new();
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        hash.update(file.to_string_lossy().as_bytes());
        hash.update([0]);
        let bytes = fs::read(root.join(&file)).expect("engine source");
        if file.starts_with("src/ai") && file.extension().is_some_and(|ext| ext == "rs") {
            let source = String::from_utf8_lossy(&bytes);
            let compact: String = source.split_whitespace().collect();
            assert!(
                !compact.contains("matches!"),
                "{}: AI filters must use nested exhaustive match functions",
                file.display()
            );
        }
        hash.update(bytes);
    }
    println!("cargo:rerun-if-changed=src");
    println!(
        "cargo:rustc-env=HEX_ENGINE_FINGERPRINT={:x}",
        hash.finalize()
    );
    // Native production catalogs remain disk-loaded. Only WASM builds bake content.
    if std::env::var_os("CARGO_FEATURE_WASM").is_some() {
        println!("cargo:rerun-if-changed=data");
        let generated = hex_command_catalog::compiler::compile(&root.join("data"))
            .unwrap_or_else(|error| panic!("invalid catalog: {error}"));
        let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("build output"));
        fs::write(output.join("catalog_baked.rs"), generated).expect("baked catalog");
    }
}
