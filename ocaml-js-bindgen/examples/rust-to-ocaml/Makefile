.DELETE_ON_ERROR:
ESBUILD := node_modules/.bin/esbuild
PORT ?= 8000

.PHONY: build bindings deps test serve clean
build: bindings
	dune build --profile release Frontend.bc.js
	rm -rf dist
	$(ESBUILD) frontend=_build/default/Frontend.bc.js bindings.runtime=generated/bindings.runtime.mjs \
	  --bundle --splitting --format=esm --minify --external:node:* \
	  --inject:./generated/bindings.runtime.mjs --out-extension:.js=.mjs --outdir=dist
	cp index.html generated/backend.wasm dist/

bindings:
	cd rust && ocamljs_pack build --backend js_of_ocaml --name example --out-dir ../generated
	cp generated/Bindings.ml generated/Bindings.mli generated/backend.wasm generated/schema.json .
	cp generated/bindings.runtime.mjs bindings.runtime.js

deps:
	npm ci

test:
	node test.mjs

serve: build
	python3 -m http.server $(PORT) --directory dist

# Readable checked-in generated bindings are intentionally retained.
clean:
	rm -rf generated _build dist rust/target backend.wasm Frontend.js
