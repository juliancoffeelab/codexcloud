---
name: rust-toolchain
description: Use for Hex Command Rust or WASM builds, tests, formatting and toolchain troubleshooting.
---

# Rust toolchain

Use existing npm commands (`wasm:build`, `check:rust`, `ai:rust:build`) when
possible. Their wrapper finds Cargo/Rustup under `/workspace`, preserves explicit
environment overrides, and resolves the project's pinned wasm-pack binary or
versioned cache. It avoids relying on exports from a previous shell.

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
