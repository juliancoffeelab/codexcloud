ESBUILD := node_modules/.bin/esbuild
PORT ?= 8000
FRONTEND_JS := _build/default/output/src/ocaml/Main.js
WORKER_JS := _build/default/output/src/ocaml/Worker.js

.PHONY: build wasm bindings frontend bundle assets deps serve clean
build: assets

# Compile the Rust library into a WebAssembly binary.
wasm:
	cd rust && cargo build --locked --release --target wasm32-unknown-unknown

# Generate OCaml bindings and the JavaScript bridge from that binary.
bindings: wasm
	ocamljs_pack generate \
	  --wasm rust/target/wasm32-unknown-unknown/release/scripted_game_demo.wasm \
	  --backend melange --name scripted_game_demo --out-dir generated/melange

# Compile the OCaml application to JavaScript; Test.ml is not part of this build.
frontend: bindings
	dune build @frontend

# Bundle the compiled JavaScript for the browser.
bundle: frontend
	rm -rf dist/melange
	mkdir -p dist/melange
	$(ESBUILD) frontend=$(FRONTEND_JS) bindings.runtime=generated/melange/bindings.runtime.mjs worker=$(WORKER_JS) \
	  --bundle --splitting --format=esm --minify --external:node:* \
	  --inject:./generated/melange/bindings.runtime.mjs \
	  --out-extension:.js=.mjs --outdir=dist/melange

# HTML and CSS load the compiler output; the Wasm runtime is linked by esbuild.
assets: bundle
	cp web/index.html web/style.css generated/melange/backend.wasm dist/melange/

serve: build
	python3 -m http.server $(PORT) --directory dist/melange

deps:
	npm ci

clean:
	rm -rf generated dist lib _build rust/target
