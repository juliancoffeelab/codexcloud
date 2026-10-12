# Development and publication

The [project README](../README.md) covers setup and common commands.

## Verification

`npm run verify` runs automatic formatting, content validation, WASM bindings,
UI and TypeScript checks, artwork validation, all tests and the production build.
Independent checks continue after failures and report their statuses together.
Select groups with `npm run verify -- types tests`. Legacy `style` selects
formatting; focused `rust` selects native tests. The combined test group already
includes Rust, so verification does not run it twice.

Prepare ignored bindings with `npm run wasm:build` before a fresh test run.
`npm test` runs Rust, Python artwork/analyzer tests and Vitest under one shared
20-second budget. Browser automation is separate. `art:check` validates generated
artwork; its unit tests run in the combined test command.

## Project layout

- `rust/data/` contains authored game content and simulation rosters/schedules.
  The [content reference](../rust/data/README.md) describes schemas and inheritance.
- `rust/src/data/`, `logic/` and `ai/` own the catalog, battles and planning.
  `simulation/` owns scheduling and recording; `protocol.rs` and `wasm.rs` expose
  the browser contract. See the [Rust guide](../rust/README.md).
- `src/game/` adapts the WASM factory, catalog and battle handles. Browser code
  receives resolved metadata rather than importing live authored JSON.
- `src/save/` owns versioned envelopes, storage and sequential compatibility
  migrations. See [save formats](save-formats.md).
- `src/controller/` owns actions, autosaves, undo, turn sequencing and cancellation.
  `src/ui/` renders published views; `src/presentation/` owns artwork, motion,
  sound and icons. See [artwork rendering](../src/presentation/README.md).

Generated bindings live in ignored `rust/pkg/`, sprites in ignored
`public/assets/sprites/`, and production bundles in ignored `dist/`. Edit source
files. Temporary experiments and screenshots belong in ignored `tmp/`.

## Sites publication

`scripts/publish-sites-call.js` coordinates the configured Sites deployment.
It invokes `npm run publish`, which commits project source, synchronizes a Sites
source checkout, builds that checkout and packages its generated output with
`scripts/package-site.py`. The adapter saves/deploys the archive, polls deployment
status and pushes the deployed source commit to GitHub after success.

The Sites source repository receives its commit before deployment; GitHub is a
separate push. Packages include generated files that ordinary `git archive`
would omit. Packaging verifies references, file coverage and archive bytes.
Credentials are passed to the publication command without being printed or saved.
On failure, retain version/deployment identifiers for recovery.

Publication and push policy is defined in [AGENTS.md](../AGENTS.md).
