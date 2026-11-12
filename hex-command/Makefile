# Dune package actions hash their environment. Keep Make's own bookkeeping
# out of native tool builds so direct and nested invocations reuse the same cache.
ARGS ?=
unexport ARGS
DUNE_ENV := env -u MAKEFLAGS -u MFLAGS -u MAKEOVERRIDES -u MAKELEVEL

.PHONY: help format format-check dev full-verify

help:
	@printf '%s\n' 'Required in every new shell: source /workspace/.setup/env.sh' 'Project commands require the tool paths and cache variables loaded by env.sh.' 'make dev' 'make format | format-check' 'make full-verify'

format:
	cargo fmt --all --manifest-path rust/Cargo.toml
	$(DUNE_ENV) dune fmt --root=. || $(DUNE_ENV) dune build --root=. @fmt
	node_modules/.bin/prettier --write --cache --cache-strategy content src packages/save-migrations/src "scripts/*.{mjs,js}" "tests/*.mjs" "*.config.mjs" "*.json" AGENTS.md ".agents/skills/**/*.{md,json,mjs}" pixel/art/factions.json "pixel/art/units/lineups/*.json" "pixel/art/units/*/*/palette.json" "pixel/art/terrain/*/palette.json" "pixel/art/portraits/*/palette.json" "docs/*.{md,json}" "rust/data/**/*.{json,md}"

format-check:
	cargo fmt --all --manifest-path rust/Cargo.toml -- --check
	$(DUNE_ENV) dune build --root=. @fmt
	node_modules/.bin/prettier --check --cache --cache-strategy content src packages/save-migrations/src "scripts/*.{mjs,js}" "tests/*.mjs" "*.config.mjs" "*.json" AGENTS.md ".agents/skills/**/*.{md,json,mjs}" pixel/art/factions.json "pixel/art/units/lineups/*.json" "pixel/art/units/*/*/palette.json" "pixel/art/terrain/*/palette.json" "pixel/art/portraits/*/palette.json" "docs/*.{md,json}" "rust/data/**/*.{json,md}"
	git diff --check -- .
	git diff --cached --check -- .

dev:
	$(DUNE_ENV) node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh dev
	$(DUNE_ENV) dune build --root=. @melange @wasm-dev @artwork
	$(DUNE_ENV) node scripts/dev.mjs $(ARGS)

full-verify:
	$(MAKE) format-check
	node scripts/content.mjs --check
	$(DUNE_ENV) node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh dev
	$(DUNE_ENV) dune build --root=. @melange @wasm-dev @artwork scripts/migration_import_check.exe
	@status=0; \
	RUST_TEST_THREADS=2 cargo test --locked --manifest-path rust/Cargo.toml --workspace --features wasm & rust_pid=$$!; \
	HEX_COMMAND_PROFILE=dev node node_modules/vitest/vitest.mjs run & frontend_pid=$$!; \
	(set -e; python3 -m unittest discover -s scripts -p 'test*.py'; \
	python3 -m unittest discover -s .agents/skills/pinkbox-generation/tests -p 'test*.py'; \
	python3 -m unittest discover -s .agents/skills/pixel-cleaning/tests -p 'test*.py'; \
	python3 -m unittest discover -s .agents/skills/combat-simulation/tests -p 'test*.py') & python_pid=$$!; \
	HEX_COMMAND_PROFILE=dev node scripts/check-wasm.mjs || status=1; \
	HEX_COMMAND_PROFILE=dev $(DUNE_ENV) dune exec --root=. -- node scripts/check-migration-contract.mjs || status=1; \
	for pid in $$rust_pid $$frontend_pid $$python_pid; do wait $$pid || status=1; done; exit $$status
