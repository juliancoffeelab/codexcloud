# Resolve supplied tools for direct Make use as well as npm commands.
SHELL := $(abspath scripts/toolchain/with-toolchain.mjs)
.SHELLFLAGS := /bin/sh -c

.DELETE_ON_ERROR:
.NOTPARALLEL:
PROFILE ?= wasm-release
PORT ?= 8000

.PHONY: build content wasm frontend assets bundle fixtures native-tests prepare test verify format serve clean help
build: bundle

content:
	node scripts/content.mjs --check

wasm: content
	cd rust && node ../scripts/toolchain/packager.mjs build --features wasm --name hex_command \
	  --out-dir pkg $(if $(filter dev,$(PROFILE)),--dev,--profile $(PROFILE))
	$(if $(filter dev,$(PROFILE)),,node scripts/optimize-wasm.mjs rust/pkg/backend.wasm)
	cp rust/pkg/Bindings.ml frontend/engine/Domain.ml
	cp rust/pkg/Bindings.mli frontend/engine/Domain.mli
	node scripts/generate-resource-visitors.mjs

frontend: fixtures
	node scripts/check-migrations.mjs
	dune build --root . @melange --display short

assets:
	python3 scripts/build-pixel-art.py

bundle: frontend assets
	node_modules/.bin/vite build

fixtures: wasm
	node scripts/fixture-json.mjs

native-tests:
	mkdir -p tmp
	cd rust && cargo test --workspace --no-run --locked --features wasm --message-format=json > ../tmp/native-test-artifacts.jsonl

# JS below only records hashes and reads Cargo's artifact manifest; it builds nothing.
prepare: build fixtures
	dune exec --root . -- which refmt ocamlformat
	node scripts/test-budget.mjs --begin-preparation
	$(MAKE) native-tests
	node scripts/test-budget.mjs --record-preparation

test:
	node scripts/test-budget.mjs

verify:
	node scripts/verify.mjs

format:
	node scripts/style.mjs

serve:
	node_modules/.bin/vite preview --host 0.0.0.0 --port $(PORT)

clean:
	rm -rf dist rust/pkg
	rm -f tmp/verification-prepared.json tmp/verification-start.json tmp/native-test-artifacts.jsonl
	dune clean --root .

help:
	@printf '%s\n' 'make build | wasm [PROFILE=dev] | frontend | assets | prepare | test | verify | format | clean' 'make serve [PORT=8000]'
