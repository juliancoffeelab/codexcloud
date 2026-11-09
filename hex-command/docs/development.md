# Development

Standing instructions and skill invocation conditions are in
[AGENTS.md](../AGENTS.md). Current npm commands are defined in
[package.json](../package.json).

## Setup

Use Node.js 24.5+, Rust stable with `rustfmt`, the `wasm32-unknown-unknown`
target, Python 3 with Pillow, and Dune 3.24.2. Dune's lock pins OCaml, Melange
and ReasonReact; no separate application Opam switch is needed. With a bootstrap
OCaml compiler on PATH, `scripts/install-dune.sh` can install Dune. The first
locked compiler build can take several minutes.

Keep the sibling SDK checkout alongside Hex. Build commands compile its packager
from source as needed; a globally installed generator is not used:

```sh
rustup target add wasm32-unknown-unknown
npm ci
npm run dev
```

The dev command prepares WASM, generated bindings, artwork and OCaml output,
then starts Dune's watcher and Vite. Vite options pass through. Restart preparation
after Rust/catalog or artwork edits. Stop the watcher before formatting or other
Dune builds. Project commands use the Node helpers in `scripts/toolchain/`;
direct Make and npm commands discover the supplied cloud toolchains without shell
setup. For direct Cargo or Dune use in Bash, run `source scripts/env.sh`; this
exports discovered paths in the current shell without installing tools or editing
profiles. Only run `npm run toolchain:setup` when persistent shell setup is requested.

For Rust/WASM tool discovery and proxy setup, use
[rust-toolchain](../.agents/skills/rust-toolchain/SKILL.md).

## Verification

```sh
npm run format
npm run wasm:build            # Prepare bindings before a fresh test run
npm test
npm run verify               # All default checks and production build
npm run verify -- types tests
npm run build
npm run preview
```

Verification continues independent checks after failures and reports their
statuses together, rebuilding stale artifacts within the shared advisory clock.
Browser automation is separate from the default checks.
Use [testing](../.agents/skills/testing/SKILL.md) for validation scope and test policy.

## Task references

- [Rust core](../rust/README.md) for engine architecture and the browser bridge.
- [Authored content](../rust/data/README.md) for game data ownership.
- [Campaign design](../.agents/skills/design-campaign/SKILL.md) for scenarios,
  objectives, events and story.
- [Unit design](../.agents/skills/design-unit/SKILL.md) and
  [terrain design](../.agents/skills/design-terrain/SKILL.md) for artwork.
- [Combat simulation](../.agents/skills/combat-simulation/SKILL.md) for AI,
  balance experiments and reports.
- [Save migrations](../.agents/skills/save-migrations/SKILL.md) for persistence.
- [Deploy](../.agents/skills/deploy/SKILL.md) for live publication.
- [Live play](../.agents/skills/live-play/SKILL.md) for explicitly authorized
  browser gameplay.

## Performance diagnostics

Menu → Advanced → Performance trace provides recording and JSON download.
Starting a recording closes Menu; reopen it to stop or download. Downloading a
recording stops and flushes it. Recordings stay in memory until reload and are
never uploaded automatically. The controls describe the recorded scopes and
limits. Use the [benchmark guide](benchmarkjs.md) for operation benchmarks and
their measurement boundaries.

Browser callback intervals are not physical display frame rates; API submission
time is not GPU execution time. Software GPU results do not establish hardware
frame rates. Normal draw readiness, completed GPU work, an animation-frame
opportunity and physical display scanout are distinct.

## OCaml port boundaries

The Dune libraries under `frontend/` define the application boundaries:

- `hex_platform` owns browser bindings, native identity collections and shared runtime utilities.
- `phaser_bindings` exposes the Phaser/WebGL API and its required subclass adapters.
- `hex_engine` owns generated Rust bindings, opaque native handles, detached publications,
  authored data and terrain geometry.
- `hex_audio` owns audio lifetimes; consumers receive read-only readiness or explicit commands.
- `hex_persistence` owns storage, migration, history identity and durable restoration.
- `hex_game` owns sessions, commands, undo and cancellable presentation phases.
- `hex_renderer` exposes boot ownership and view data; scene, resource and camera modules are private.
- `hex_devtools` contains the offline map renderer and its private JSON reader; it is absent from the playable bundle.

Dune requires direct dependencies. Private modules cannot be imported by another
OCaml library; interfaces hide mutable owners and distinguish readers from publishers.
Binding declarations operate on native objects without runtime proxy objects.
The root UI continues to use ReasonReact with Preact compatibility imports.
Phaser ownership is loaded lazily after the catalog-free startup shell.

Generate Rust/WASM, glue and `frontend/engine/Domain.ml/.mli` together. Fixture
codecs belong to tests and probes, not application JSON loading. The terrain
worker and offline builder share OCaml rendering code and load no engine WASM or
live catalog. The [save-migrations skill](../.agents/skills/save-migrations/SKILL.md)
owns the persistence workflow and its independent frozen-format package.

Publication uses `scripts/site-operation.mjs`: committed Hex and sibling SDK
sources are pinned to one repository revision, built in an isolated workspace,
and packaged before hosting. `--preview <PR-number>` selects a separate Site and
release cache. See [deployment](../.agents/skills/deploy/SKILL.md).
