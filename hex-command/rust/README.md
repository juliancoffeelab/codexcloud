# Rust game core

[Authored JSON](data/README.md) supplies rules and scenario content directly.
The `hex-command-catalog` mini crate owns definitions, snapshots and catalog validation;
`logic` owns battle commands and campaign
execution, and `ai` owns recruitment, campaign orders and planning. `simulation` owns scheduling,
workers and records; `protocol` defines the browser transfer contract.

A native host owns `Catalog` and lends it to operations. Native workers share an immutable
catalog through scoped threads, while each match owns state and planning workspace.
Immutable geometry, effective missions and distance fields use local `Rc` inside
one match; the core needs no shared mutable catalog or synchronization locks.

## Core execution

`Catalog::load_from` and `Catalog::from_json_bundle` validate content and resolve
terrain/race/unit inheritance once. Native hosts use disk content, including
`--data-dir` overrides; explicit browser bundles use the same loader. `Game` executes typed commands and owns battle
state. Occupancy and unit indexes avoid roster scans; terrain revisions invalidate
cached route fields.

AI tactics compose a recruiter, campaigner and planner. Each receives a restricted
view and contributes to a shared turn draft. Native matches execute that draft;
the browser consumes its command queue through revision-checked battle handles.

## Baked catalog

With the `wasm` feature, `build.rs` calls the catalog crate's host-only compiler.
It validates authored JSON and emits resolved Rust statics into Cargo's `OUT_DIR`.
The default WASM factory borrows that catalog directly: it performs no JSON parsing,
inheritance resolution, fingerprint hashing or catalog collection construction.
Content edits trigger regeneration through Cargo's directory dependency tracking.
Generated files remain build artifacts; JSON remains the authoring source.

Catalog strings and slices use `Cow`; baked values borrow literals, while native
loads own their data. Ordered lookup tables borrow sorted entries and use binary
search. Derived movement classes and content fingerprints are computed by the
shared loader during compilation. Initial map snapshots and map-picker metadata
use static templates: they become owned only when a match starts, a save is
validated or metadata is requested. Live battle state and save formats remain owned.
Road-cap validation also uses a generated static table. Unicode lowercase log labels
are computed by the catalog compiler, so browser actions do not need Unicode casing
code or allocate transformed labels. Native-only arena layouts and workload defaults
are excluded from the browser catalog.

The JSON loader, inheritance resolver, validator and fingerprint hasher are excluded
from default WASM builds. Custom JSON bundles require the opt-in `runtime-catalog`
feature; passing a bundle to a default factory returns an `invalidRequest` error.
Native disk loading and `--data-dir` remain available without that feature. To build
browser bindings with custom importing enabled:

```sh
node --import tsx .agents/skills/rust-toolchain/scripts/with-toolchain.ts wasm-pack build rust --target web --dev --out-dir pkg --out-name hex_command --no-pack --no-opt --locked --features wasm,runtime-catalog
```

The catalog crate's `bake` feature and `databake` dependency are build-host tools;
they are not needed by the WASM runtime. The crate is internal to this project.

## Browser contract

The optional `wasm` feature exposes `NativeEngineFactory`: catalog, create and
restore; battle handles expose views, precomputed action inspection, dispatch, AI
commands, save and disposal. Revision checks reject stale decisions. Saved games
contain only game state and seed. Browser saves bracket the whole enemy turn;
commands and planning workspaces are never persisted.

Each browser handle retains its live `Game`, numeric indexes and mechanical caches.
Choosing commands and inspecting actions borrow that runtime; dispatch mutates it
only after command validation. Protocol snapshots are materialized at publication
and save boundaries, without reconstructing a game for each command. Catalogs
borrow baked statics or share an owned custom catalog through `Rc`; native matches
keep their borrowed catalog path. The live adapter publishes the snapshot returned
by dispatch instead of requesting the same snapshot again.

Planning forks share terrain through `Rc<[usize]>`, with copy-on-write terrain
updates. Cache validation compares shared terrain identities without copying or
scanning the grid.
They skip log copying, narration formatting and combat-animation publication;
resolved live commands retain their presentation and recording behavior.
Each planning workspace retains integer search buffers; destination filtering
operates in place and bounded origin selection uses four stack entries. WASM's
shared sorting kernel recycles an index buffer per thread. Taking that buffer
before comparison also permits nested sorts without a mutable-borrow conflict.

The live TypeScript adapter is `src/game/native-engine.ts`. Engine handles use
`wasm-bindgen`; commands, snapshots and responses derive `Tsify`. Typed `Ts<T>`
arguments validate inside the facade, where conversion failures return ordinary
engine errors. Concrete transparent response types retain the shared
`EngineResult<T>` envelope without losing generic arguments at the WASM boundary.
Serialization uses JavaScript objects through Tsify's `js` feature, with maps as
objects and explicit nulls; omitted optional save fields remain omitted.

The Svelte UI, map picker, artwork, sound, pacing and browser tools remain in
JavaScript. `src/game/live-game.ts` adapts native state/actions to the existing
controller. Its published views contain detached state and precomputed Rust
inspection results; rendering never calls the mutable engine. Moves, combat,
recruitment, promotion, enemy planning and campaign transitions dispatch through
WASM. Browser checkpoints retain native state and seed; previous browser saves
are imported once, including pending enemy-turn recovery. Undo restores a native
checkpoint. JavaScript continues to own campaign and scenario selection.

`wasm-pack` generates the JavaScript, `.wasm` and `.d.ts` together in Git-ignored
`rust/pkg/`. The adapter imports those generated types directly and derives its
bridge signatures from the generated class. There is no separate type exporter
or handwritten bridge contract.

```sh
npm run wasm:build
npm run wasm:build:release
npm run check:wasm
```

`wasm-pack` is pinned as an npm development dependency and installs the matching
`wasm-bindgen` tool. Development and type-check commands build the bindings before
using them; the production build uses the `wasm-release` profile and removes diagnostic
function names and DWARF sections from the final WASM. Development builds retain
names for debugging. Executable sections, exports and target-feature metadata are
preserved. The WASM profile uses size optimization, full LTO and one codegen unit
to limit inlining and repeated generic code. Native release and simulation profiles
keep their separate speed-oriented settings. WASM sorting uses one shared index
kernel with original positions as tie breakers, instead of instantiating full sort
implementations for every record and comparator. Native sorting remains specialized.
Small ordered publication maps use incremental insertion to avoid bulk-sort code.
Simulation recording and profiling are compiled only for native targets; browser
combat retains its animation events without duplicate telemetry buffers. When npm runs
behind an environment proxy, use `NODE_USE_ENV_PROXY=1 npm ci` with Node 24.5 or
newer so its binary installer uses that proxy.

Browser persistence has one versioned envelope owned by `src/save/`. The generated
`SavedGame` checkpoint contains only seed and resolved game state;
it has no separate format version. Historical conversion runs before native
restore. Rust validates current battle invariants and never migrates browser saves.
See [save migrations](../docs/save-migrations.md).

## Tests and simulations

`npm test` includes native rules, commands, catalog validation, saves, scheduling
and recording tests together with Python and browser-adapter tests under one
shared 20-second budget. `npm run check:rust` is available for a focused native run.
Its WASM feature also compares every baked catalog field and deferred snapshot
with the authored disk catalog.

Use the [simulation guide](../.agents/skills/combat-simulation/SKILL.md) for
commands, comparisons and interpreting evidence, and the
[development guide](../docs/development.md) for verification and publication.
