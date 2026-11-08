.NOTPARALLEL:
PORT ?= 8000
DEMOS := hello-world calculator todo game-of-life image-editor document-explorer async-search scripted-game cat-pics reference-conformance

.PHONY: build packager demos $(DEMOS) example matrix reference-fixtures deps install prepare test test-browser serve clean
build: packager demos reference-conformance

demos:
	node scripts/projects.mjs demos

$(DEMOS):
	$(MAKE) -C demos/$@ build

packager:
	cargo build --locked -p ocamljs_pack

example:
	$(MAKE) -C examples/rust-to-ocaml build

# Portability validation belongs here. Demo Makefiles build their normal frontend.
matrix: demos
	node scripts/projects.mjs matrix

reference-fixtures:
	node scripts/projects.mjs reference-fixtures

install:
	cargo install --locked --path crates/ocamljs_pack

deps:
	node scripts/projects.mjs deps

prepare: install
	node scripts/preparation.mjs begin
	$(MAKE) packager
	cargo test --locked --workspace --no-run
	node scripts/projects.mjs native-tests
	$(MAKE) example
	$(MAKE) matrix
	$(MAKE) reference-fixtures
	node scripts/preparation.mjs finish

test:
	node scripts/run-tests.mjs

test-browser:
	node scripts/browser.mjs

serve:
	python3 -m http.server $(PORT) --directory demos

clean:
	cargo clean
	$(MAKE) -C examples/rust-to-ocaml clean
	node scripts/projects.mjs clean
	rm -f tmp/test-preparation.json tmp/test-preparation-start.json
