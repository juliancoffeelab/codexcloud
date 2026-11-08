
.PHONY: build wasm bindings deps checks clean
build: checks

# Compile the Rust library into a WebAssembly binary.
wasm:
	cd rust && cargo build --locked --release --target wasm32-unknown-unknown

# Generate OCaml bindings and the JavaScript bridge from that binary.
bindings: wasm
	ocamljs_pack generate \
	  --wasm rust/target/wasm32-unknown-unknown/release/reference_conformance.wasm \
	  --backend melange --name reference_conformance --out-dir generated/melange
	cp tests/third-party.mjs generated/melange/

# Compile this project's Melange conformance checks without running them.
checks: bindings
	dune build @checks

deps:
	npm ci

clean:
	rm -rf generated dist lib _build rust/target
