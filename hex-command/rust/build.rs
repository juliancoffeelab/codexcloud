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
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("crate directory"));
    let mut files = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("Cargo.lock"),
        PathBuf::from("build.rs"),
    ];
    sources(&root, &root.join("src"), &mut files);
    files.sort();
    let mut hash = Sha256::new();
    for file in files {
        println!("cargo:rerun-if-changed={}", file.display());
        hash.update(file.to_string_lossy().as_bytes());
        hash.update([0]);
        hash.update(fs::read(root.join(file)).expect("engine source"));
    }
    println!("cargo:rerun-if-changed=src");
    println!(
        "cargo:rustc-env=HEX_ENGINE_FINGERPRINT={:x}",
        hash.finalize()
    );
}
