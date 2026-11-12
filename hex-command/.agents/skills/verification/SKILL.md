---
name: verification
description: Use for implementation validation, targeted checks and tests, final verification, and deciding whether to add tests.
---

# Hex Command verification

Tests must be small, fast, and lightweight.

- Data-only changes do not justify new behavioral tests or extra named-content
  cases in existing test matrices.
- Validate content through generic schema and consistency checks. Measure
  balance through simulations.
- A test belongs in the codebase only if the agent can promise its value is
  greater than its total burden, including runtime, maintenance, brittleness and
  duplication. If that promise cannot be made, use a disposable probe. Reproducing
  a bug alone does not satisfy this rule.
- For bug fixes, use a disposable test or probe: reproduce the failure, fix the
  code, verify that the probe passes, then delete it. Run the relevant existing
  suites and remove obsolete expectations rather than preserving incorrect behavior.
- Use a small, fixed set of representative fixtures. Do not freeze current
  unit stats, promotion choices, recruitment lists or roster sizes.
- Do not duplicate existing coverage merely because new content uses it.
  Remove redundant cases when found.

- Keep browser automation out of the default test suite. Playwright is available
  for explicitly requested browser spot checks; keep scratch probes in ignored
  `tmp/`. Do not add mandatory browser installs, extra services, large migration
  suites, or Git-history fixtures. Use focused checks for meaningful rules,
  state transitions, publication, and cancellation behavior.

## Verification

Run targeted checks while working. When the work is ready to present, run
`make full-verify` to catch unexpected regressions. Fix failures and repeat.

- `cargo check --locked --manifest-path rust/Cargo.toml --workspace`: Rust compilation.
- `cargo test --locked --manifest-path rust/Cargo.toml --workspace --features wasm [filter]`: Rust tests.
- `dune build --root=. @frontend/check`: OCaml/Reason typechecking without JavaScript emission.
- `node node_modules/vitest/vitest.mjs run [test-file]`: tests against prepared frontend output.
- `python3 -m unittest discover -s scripts -p 'test*.py'`: Python tests; select the affected directory/file.
- `make format` / `make format-check`: apply / check formatting.

Before frontend tests, compile changed OCaml with `dune build --root=. @melange @artwork`. For changed Rust exports, first run `node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh dev` to prepare WASM/bindings.

If a forwarded Dune build reports an internal error such as `Build via RPC failed, but the RPC server did not send an error message.`, save the watcher output and `_build/trace.csexp` before running `dune shutdown --root=.`. Rerun the failed command outside the watcher, fix any reported source error, then restart `make dev`. If the internal error recurs, investigate the saved diagnostics. Traces can contain environment secrets; sanitize them before sharing.
