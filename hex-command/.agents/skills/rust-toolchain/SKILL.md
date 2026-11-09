---
name: rust-toolchain
description: Use for Hex Command Rust or WASM builds, tests, formatting and toolchain troubleshooting.
---

# Rust toolchain

Use existing npm commands (`wasm:build`, `check:rust`, `ai:rust:build`) when
possible. Their wrapper finds Cargo/Rustup under `/workspace`, preserves explicit
environment overrides, and avoids relying on exports from a previous shell.

WASM builds, binding checks and migration type generation use
`scripts/toolchain/packager.mjs`. It runs the sibling SDK's `ocamljs_pack` from
source in release mode, with Cargo checking freshness and reusing its build cache.
No globally installed generator is needed. Release preparation stages the SDK at its pinned source
revision, then invokes the same Make build targets as local development.

Keep Dune invocations bound to this project with `--root`. A nested Site checkout
otherwise selects the outer workspace, even with its own Git repository and
`dune-workspace` file.

```sh
node scripts/toolchain/with-toolchain.mjs \
  cargo fmt --manifest-path rust/Cargo.toml --check
```

Only when persistent direct-shell setup is requested, use
`npm run toolchain:setup`. This installs environment hooks and managed wrappers
in the user's home; it leaves existing unmanaged wrappers alone. Routine build
and formatting need only the command wrapper.

Native simulation builds default to an optimized incremental profile without LTO.
PGO is opt-in (`ai:rust:build -- --pgo`, or `HEX_COMMAND_SIM_PGO=1` for automatic
build hooks); do not select it implicitly for routine reports or source edits.
