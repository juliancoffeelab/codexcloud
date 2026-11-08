fn main() {
    println!("cargo:rerun-if-env-changed=GLUE_BENCH_UPSTREAM_WIRE");
    if std::env::var_os("CARGO_FEATURE_BASELINE").is_some() {
        let path =
            std::env::var("GLUE_BENCH_UPSTREAM_WIRE").expect("Supply pinned upstream wire.rs");
        println!("cargo:rerun-if-changed={path}");
        std::fs::write(
            std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("reference.rs"),
            format!("#[path = {path:?}] mod upstream_wire;"),
        )
        .unwrap();
    }
}
