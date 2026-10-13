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
        PathBuf::from("terrain/dead-ends.json"),
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
    // Build a file-path table, not exported rules. Only test/WASM code includes
    // these JSON bytes; native production catalogs still load authored files.
    fn documents(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("catalog directory") {
            let path = entry.expect("catalog entry").path();
            if path.is_dir() {
                documents(&path, files);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                files.push(path);
            }
        }
    }
    let data = root.join("data");
    let mut files = Vec::new();
    documents(&data, &mut files);
    files.sort();
    let mut generated = String::from("const EMBEDDED_DOCUMENTS: &[(&str, &str)] = &[\n");
    for file in files {
        if std::env::var_os("CARGO_FEATURE_WASM").is_some() {
            println!("cargo:rerun-if-changed={}", file.display());
        }
        let name = file
            .strip_prefix(&data)
            .expect("document within catalog")
            .to_string_lossy()
            .replace('\\', "/");
        generated.push_str(&format!(
            "({name:?}, include_str!({:?})),\n",
            file.to_string_lossy()
        ));
    }
    generated.push_str("];\n");
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("build output"));
    fs::write(output.join("catalog_documents.rs"), generated).expect("catalog file table");
    if std::env::var_os("CARGO_FEATURE_WASM").is_some() {
        println!("cargo:rerun-if-changed=data");
    }
}
