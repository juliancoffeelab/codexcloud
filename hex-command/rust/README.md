# Rust game core

[Authored JSON](data/README.md) supplies rules and scenario content directly.
`data` owns definitions and snapshots, `logic` owns battle commands and campaign
execution, and `ai` owns recruitment, campaign orders and planning. `simulation` owns scheduling,
workers and records; `protocol` defines the browser transfer contract.

A host owns `Catalog` and lends it to operations. Native workers share an immutable
catalog through scoped threads, while each match owns state and planning workspace.
Immutable geometry, effective missions and distance fields use local `Rc` inside
one match; the core needs no shared mutable catalog or synchronization locks.

## Core execution

`Catalog::load_from` and `Catalog::from_json_bundle` validate content and resolve
terrain/race/unit inheritance once. `Game` executes typed commands and owns battle
state. Occupancy and unit indexes avoid roster scans; terrain revisions invalidate
cached route fields.

AI tactics compose a recruiter, campaigner and planner. Each receives a restricted
view and contributes to a shared turn draft. Native matches execute that draft;
the browser consumes its command queue through revision-checked battle handles.

## Browser contract

The optional `wasm` feature exposes `NativeEngineFactory`: catalog, create and
restore; battle handles expose views, precomputed action inspection, dispatch, AI
commands, save and disposal. Revision checks reject stale decisions. Saved games
contain only game state and seed. Browser saves bracket the whole enemy turn;
commands and planning workspaces are never persisted.

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
using them; the production build uses the release profile. When npm runs
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
Its WASM feature also compares embedded content with the authored disk catalog.

Use the [simulation guide](../.agents/skills/combat-simulation/SKILL.md) for
commands, comparisons and interpreting evidence, and the
[development guide](../docs/development.md) for verification and publication.
