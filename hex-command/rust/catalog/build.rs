use std::{
    fs,
    path::{Path, PathBuf},
};
fn collect(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("catalogue document directory") {
        let path = entry.expect("catalogue document entry").path();
        if path.is_dir() {
            collect(root, &path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(
                path.strip_prefix(root)
                    .expect("document within catalogue")
                    .to_owned(),
            );
        }
    }
}
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_WASM").is_none() {
        return;
    }
    let root =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("crate path")).join("../data");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    files.sort();
    let mut source = String::from("const EMBEDDED_DOCUMENTS: &[(&str, &str)] = &[\n");
    for file in files {
        source.push_str(&format!(
            "({:?}, include_str!({:?})),\n",
            file.to_str()
                .expect("UTF-8 document path")
                .replace('\\', "/"),
            root.join(file).to_str().expect("UTF-8 include path")
        ));
    }
    source.push_str("];\n");
    fs::write(
        PathBuf::from(std::env::var_os("OUT_DIR").expect("build output")).join("documents.rs"),
        source,
    )
    .expect("embedded catalogue source");
}
