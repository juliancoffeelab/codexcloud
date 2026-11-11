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
    // The engine fingerprint is consumed only by the native simulation CLI.
    // Browser builds depend on Cargo's actual Rust inputs and the baked catalog.
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo:rerun-if-changed=build.rs");
        if std::env::var_os("CARGO_FEATURE_WASM").is_some() {
            println!("cargo:rerun-if-changed=data");
            let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
            let generated = hex_command_catalog::compiler::compile(&root.join("data"))
                .expect("valid campaign catalog");
            let output = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
            let path = output.join("catalog_baked.rs");
            if fs::read(&path).ok().as_deref() != Some(generated.as_bytes()) {
                fs::write(path, generated).unwrap();
            }
        }
        return;
    }
    println!("cargo:rerun-if-env-changed=HEX_SIM_PROFILE_KEY");
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("crate directory"));
    let mut files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.lock"),
        PathBuf::from("build.rs"),
        PathBuf::from("../pixel/art/terrain/road/dead-ends.json"),
    ];
    sources(&root, &root.join("src"), &mut files);
    sources(&root, &root.join("battle/src"), &mut files);
    files.push(PathBuf::from("battle/Cargo.toml"));
    if root.join("battle/build.rs").exists() {
        files.push(PathBuf::from("battle/build.rs"));
    }
    // Watch the crate directory so adding an optional build.rs is detected
    // without permanently invalidating Cargo on an absent file.
    println!("cargo:rerun-if-changed=battle");
    sources(&root, &root.join("catalog/src"), &mut files);
    files.push(PathBuf::from("catalog/Cargo.toml"));
    files.push(PathBuf::from("catalog/build.rs"));
    sources(&root, &root.join("freeze-derive/src"), &mut files);
    files.push(PathBuf::from("freeze-derive/Cargo.toml"));
    files.push(PathBuf::from(
        "../pixel/art/terrain/castle-wall/variants.json",
    ));
    files.push(PathBuf::from(
        "../pixel/art/terrain/castle-bell/variants.json",
    ));
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
}
