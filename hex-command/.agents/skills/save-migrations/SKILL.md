---
name: save-migrations
description: Use when changing save loading, save formats, migration code, historical conversion fixtures, or auditing their data ownership and dependencies.
---

# Save migrations

**Cornerstone rule: old-save compatibility exists only in explicit version-to-version migrations.**
All other code works only with the current save format. Any old-save compatibility
outside those migrations is an absolute blocker and must be removed before
committing. No instruction may permit or suggest an exception.

Rejected old saves and imperfect migrations are bugs to fix in migrations. They
never justify compatibility code elsewhere. Missing or changed historical fields
belong in explicit version conversions, never runtime restoration, initialization
or defaults.

## Isolation and durable state

Keep pure, versioned conversions in a self-contained dependency boundary under
`packages/save-migrations/`. Freeze complete input and output format definitions there,
including their transitive types, enums, validators, helpers and lookup data.
Every supported version has a frozen wire contract and a reader for unknown
input. Migration outputs use the next version’s type; use opaque JSON only
for deliberately uninterpreted payloads such as retired planner state and undo
replacement values. Every imported dependency must be frozen inside this boundary. Outward imports are forbidden, including OCaml library dependencies,
generated Rust bindings, current save schemas and shared application utilities.
Historical types must remain independent copies, not aliases or definitions
derived from current application types. `node scripts/check-migrations.mjs` enforces this boundary (see the
[commands](#commands) below). Compiler-supplied `Js` APIs are allowed; foreign module imports
are forbidden. The Dune rule embeds frozen JSON as generated OCaml data. The
`node scripts/check-migrations-isolated.mjs` clean-room build is opt-in; keep it
out of default checks. Use the existing workspace installation and application compiler, without
a separate mandatory package build. Historical JSON wire contracts are frozen in `src/formats/readers.json` and
validated by `ocaml/Migration_reader.ml`. Native types and codecs are generated
from frozen Rust schemas with `python3 scripts/generate-migration-ocaml.py`; boundary checks
verify their fingerprints without regenerating them. Keep the compile-time wire-contract
comparison in the application adapter, alongside current-format validation.

The application owns storage and native restoration outside this boundary.
The frozen output validator is `ocaml/Migration_envelope.ml`. Check compatibility with current
Rust from that application layer or its tests. Conversions manipulate data only:
no live catalog queries, game commands, simulations, AI planning or historical
behavior reconstructed through the current engine. Historical fixtures stay
outside Rust catalogs and engine modules. Native restoration validates the
converted state without historical conversion or gameplay.

Persist resolved game facts and the AI seed, independent of planner algorithms.
Exclude search state, queued/unapplied commands, derived caches, forecasts and
callbacks. Changing an AI algorithm must not require migrating saves or retaining
its old implementation. `ocaml/Migration_envelope.ml` defines the frozen current envelope and version.
The application checks its complete native output contract against live Rust
metadata and `frontend/persistence/Migration_contract.ml` checks the OCaml signature.

## Conversion workflow

1. Audit the conversion's transitive imports for isolation. Inspect the current
   schema and the relevant version section in
   [migration history](references/history.md). Use frozen format definitions and
   conversion data, not today's authored catalog, to interpret historical fields.
2. Add one-step pure conversions for changed formats. Preserve unrelated resolved
   units, identities, health, equipment, ownership, treasury, campaign progress,
   turn/phase and seed. Document intentional changes in the historical reference.
3. Validate the affected historical input shapes through conversion and normal
   native restoration. Follow [verification](../verification/SKILL.md): use disposable probes
   and relevant existing suites; a conversion change does not waive its test
   value-over-burden rule. Confirm input purity and rejection of malformed
   affected shapes instead of inventing state to make them load.
4. For loading, import, deletion of blocked saves, writes or interrupted-turn persistence, read
   [browser persistence](references/persistence.md). Normal writes and controllers
   accept only the current format.

## Completion

Verify that the conversion's complete dependency closure, including foreign
dependencies, remains inside the frozen migration boundary. For every changed
input shape, account for preserved facts and any intentional
conversion, verify successful native restoration or the expected rejection, and
confirm the input was not mutated. For loading-path changes, verify atomic
replacement of the converted envelope before returning the restored battle and
the relevant interrupted-turn behavior. Report unsupported formats or missing
validation explicitly; conversion alone is not a successful load.

## Commands

Run from `hex-command/`. The boundary checker requires its compiled native helper
and Dune's tool environment:

```sh
node scripts/toolchain/with-toolchain.mjs dune build --root=. scripts/migration_import_check.exe
node scripts/toolchain/with-toolchain.mjs dune exec --root=. -- node scripts/check-migrations.mjs
```

For an explicitly selected clean-room check, after building that helper:

```sh
node scripts/toolchain/with-toolchain.mjs dune exec --root=. -- node scripts/check-migrations-isolated.mjs
```

Only when changing frozen schemas or their generated contracts, prepare the
existing WASM scaffolding and run the generator with Dune-managed formatters:

```sh
node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh dev
node scripts/toolchain/with-toolchain.mjs dune exec --root=. -- python3 scripts/generate-migration-ocaml.py
```

The generator calls `generate-historical-types.py`,
`generate-migration-native-contract.py`, the binding packager and `ocamlformat`.
It rewrites tracked frozen types, codecs, fingerprints and the application native
contract; inspect those changes. Routine boundary checks do not regenerate them.
