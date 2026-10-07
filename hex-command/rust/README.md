# Rust game core and simulation harness

The JSON files in [`data/`](data/README.md) are the source of rules and scenario
content. Rust reads them directly. There is no TypeScript export step.

The core is divided into `data` (definitions and owned snapshots), `logic`
(commands, movement, combat, turns and campaign execution), and `ai` (complete
classic/rush/mixed policies). `simulation` owns scheduling, scoped workers,
journals and reports. `protocol` defines the browser transfer contract.

A host owns `Catalog` and passes `&Catalog` to operations. Native workers borrow
one catalog through scoped threads; each match owns its state and planning
workspace. The engine uses no `Arc` or mutex. Immutable geometry, effective
missions and distance fields use local `Rc` within a single-threaded match.

## Native commands

Run from the project directory:

```sh
npm run ai:rust:build
npm run ai:schedule -- --mode full --seeds 10
npm run ai:simulate -- --mode arena --seeds 2 --out artifacts/arena.json
npm run ai:simulate -- --mode campaign --seeds 2 --out artifacts/campaign.json
npm run ai:full -- --out artifacts/full.json
npm run ai:simulate -- --replay artifacts/full.json
```

Simulation batches require an explicit user request. `--schedule-only` enumerates
configurations without running matches. The ten-seed default full schedule has
11,520 arena matches and 720 campaign attempts.

npm builds and invokes the binary; it does not schedule or aggregate simulations.
Rust respects the CPU quota, limits workers to eight, and writes complete
byte-indexed, SHA-256 checked JSONL journals plus compact JSON/Markdown reports.
Records contain every action, combat, roster, objective and endpoint state. A
bounded channel limits in-flight results; workers never share mutable battles.

Progress uses an append-only `.outcomes.jsonl` index and a constant-size
`.partial.json` snapshot. The final report retains its existing schema. See
[recording and recovery](src/simulation/README.md) for journal formats and
interrupted-run recovery.

`--replay` requires the recorded engine/content fingerprint and reruns the recorded
matches. Cross-engine comparison instead uses saved detailed journals through the
Arrow workflow below; it does not execute matches.

```sh
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/baseline.arrow artifacts/baseline.details.jsonl
uv run --with pyarrow scripts/convert-simulation-dump.py artifacts/current.arrow artifacts/current.details.jsonl
uv run --with pyarrow --with polars==2.0.0 scripts/compare-simulation-dumps.py artifacts/baseline.arrow artifacts/current.arrow
```

The converter reads JSONL with an explicit schema and preserves record order.
Supply all detail journals for a sharded report. The comparator matches identities
with a one-to-one Polars join, gathers Arrow rows and compares gameplay values
without sorting. Missing or duplicate identities fail comparison; arrays within
matches retain their order. It excludes only `stateHash`, `elapsedMs`,
`elapsedSeconds` and `detail` at the match and mission level, and records per-stage
wall time, CPU time and peak memory. Convert each saved dump once and reuse its
Arrow file for subsequent comparisons.

`HEX_COMMAND_DATA_ROOT` selects another catalog directory. Otherwise native loading
uses the crate's data directory, independent of the working directory. JSON edits
are read at the next load without recompiling. `--fingerprint` reports the loaded
content fingerprint separately from the compiled engine fingerprint.

`--stdio` retains the one-request/one-result JSON-line interface for development
samples. It is not the harness transport.

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

## Verification and bounded benchmarks

`npm run verify` batches style, the WASM package build, UI/types, Rust tests,
artwork, Vitest and the frontend build. `npm run format`/`style` include Rust and
authored JSON. Focused Rust tests cover loading, rules, commands, saves, scheduling,
recording and error paths, including the facade's saved AI continuation; they do
not launch campaign/tournament batches.

`npm run ai:benchmark -- --limit 12 --out artifacts/sample.json` runs a bounded
native sample. `--compare artifacts/sample.json` checks complete result hashes for
identical configurations; `--alternate` changes seed, initiative and campaign
policy/diplomacy. The Rust scheduler supplies the sample configurations.
