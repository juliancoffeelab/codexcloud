# Rust game core

[Authored JSON](data/README.md) supplies rules and scenario content directly.
`data` owns definitions and snapshots, `logic` owns battle commands and campaign
execution, and `ai` owns classic/rush/mixed planning. `simulation` owns scheduling,
workers and records; `protocol` defines the browser transfer contract.

A host owns `Catalog` and lends it to operations. Native workers share an immutable
catalog through scoped threads, while each match owns state and planning workspace.
Immutable geometry, effective missions and distance fields use local `Rc` inside
one match; the core needs no shared mutable catalog or synchronization locks.

## Public core API

`Catalog::load_from` and `Catalog::from_json_bundle` validate content and resolve
terrain/race/unit inheritance once. `Battle::new`, `Battle::campaign` and
`Battle::restore` construct an owned battle. Queries and `Battle::apply` borrow
`&Catalog`; invalid commands leave state unchanged. `snapshot` borrows the coherent
state, and `save` copies it for persistence.

Commands and identifiers are typed. Goal/trigger variants require their necessary
fields. Effective movement and damage tables are internal resolved data. Occupancy
and unit indexes avoid roster scans; terrain revisions invalidate cached heap-based
route fields. AI emits commands through a read-only planning interface. Recruitment
is part of each policy, not a separate planner/recruiter dimension.

Native matches use a compact internal battle directly. The public/browser facade
uses owned snapshots at the boundary, preserving the existing JSON/save format,
including absent versus null fields. See the Rustdoc for operation contracts.

## Browser contract

The optional `wasm` feature exposes `NativeEngineFactory`: catalog, create and
restore; battle handles expose views, movement/combat previews, dispatch, AI
commands, save and disposal. Revision checks reject stale decisions. Saved games
retain pending AI commands and planner continuation, excluding disposable caches.

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
using them; the production build uses the release profile. `bindings:generate`
and `check:bindings` remain aliases for the standard WASM build. When npm runs
behind an environment proxy, use `NODE_USE_ENV_PROXY=1 npm ci` with Node 24.5 or
newer so its binary installer uses that proxy.

Browser persistence has one versioned envelope owned by `src/save/`. The generated
`SavedGame` checkpoint contains seed, state and optional planner continuation;
it has no separate format version. Historical conversion runs before native
restore. Rust validates current battle invariants and never migrates browser saves.
See [save migrations](../docs/save-migrations.md).

## Tests and simulations

`npm test` includes native rules, commands, catalog validation, saves, scheduling
and recording tests together with Python and browser-adapter tests under one
shared 20-second budget. `npm run check:rust` is available for a focused native run.
Its WASM feature also compares embedded content with the authored disk catalog.

Use the [simulation guide](../.agents/skills/combat-simulation/usage.md) for commands and comparisons,
[AI policy reference](src/ai/README.md) for planning behavior,
[recording reference](../.agents/skills/combat-simulation/implementation.md#native-simulation-records-and-recovery) for journals and recovery, and
[development guide](../docs/development.md) for full verification and publication.
