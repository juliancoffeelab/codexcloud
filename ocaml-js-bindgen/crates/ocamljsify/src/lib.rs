//! Native OCaml data descriptions for Rust client-side WebAssembly libraries.
//! Mark records, enums and aliases with `#[ocamljsify::ocamljsify]`. The same
//! compiler-checked codec/schema powers all OCaml JavaScript backends.
pub use ocamljs_bindgen::ocamljs_bindgen as ocamljsify;
#[cfg(feature = "generator")]
mod generator;
#[cfg(feature = "generator")]
pub use generator::*;
