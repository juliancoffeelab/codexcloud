#!/usr/bin/env bash
# Build native-owned WASM inputs and let Dune generate/package the interface.
set -euo pipefail
cd "$(dirname "$0")/.."
profile=${1:-dev}
case "$profile" in dev|release) ;; *) echo 'Expected dev or release' >&2; exit 2 ;; esac
rust_target=${CARGO_TARGET_DIR:-"$PWD/rust/target"}
sdk_target=${OCAMLJS_PACK_TARGET_DIR:-${CARGO_TARGET_DIR:-"$PWD/../ocaml-js-bindgen/target"}}
cargo build --quiet --locked --manifest-path ../ocaml-js-bindgen/Cargo.toml --package ocamljs_pack --release --target-dir "$sdk_target"
mkdir -p "rust/pkg/$profile"
install -C -m 755 "$sdk_target/release/ocamljs_pack" rust/pkg/ocamljs_pack
cargo build --locked --manifest-path rust/Cargo.toml --lib --target wasm32-unknown-unknown --features wasm --profile "wasm-$profile" --target-dir "$rust_target"
install -C -m 644 "$rust_target/wasm32-unknown-unknown/wasm-$profile/hex_command_core.wasm" "rust/pkg/$profile/input.wasm"
install -C -m 644 "rust/pkg/$profile/input.wasm" rust/pkg/api-input.wasm
dune build --root=. "@wasm-$profile"
