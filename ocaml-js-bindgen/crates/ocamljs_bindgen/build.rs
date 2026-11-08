fn main() {
    println!("cargo:rerun-if-env-changed=OCAMLJS_RESOURCE_DIAGNOSTICS");
    println!("cargo:rustc-check-cfg=cfg(ocamljs_resource_diagnostics)");
    if std::env::var("OCAMLJS_RESOURCE_DIAGNOSTICS").as_deref() == Ok("1") {
        println!("cargo:rustc-cfg=ocamljs_resource_diagnostics");
    }
    println!("cargo:rerun-if-env-changed=OCAMLJS_PARANOID");
    println!("cargo:rustc-check-cfg=cfg(ocamljs_paranoid)");
    if std::env::var("OCAMLJS_PARANOID").as_deref() == Ok("1") {
        println!("cargo:rustc-cfg=ocamljs_paranoid");
    }
}
