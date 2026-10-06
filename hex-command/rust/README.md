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
# Cross-engine comparison of recorded matches (runs the recorded workload).
npm run ai:simulate -- --compare artifacts/ai-performance/full-final/report.json
# Explicitly bounded comparison against a different catalog, if needed.
npm run ai:simulate -- --compare artifacts/ai-performance/full-final/report.json --compare-limit 12 --data-dir rust/data
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

`--replay` requires the recorded engine/content fingerprint. `--compare` accepts
an older engine's indexed report and checks complete parsed records against native
results. It ignores object-key order, exact integral number spelling (`24` versus
`24.0`) and derived summary hashes/timings, preserving arrays, actual numeric
differences, and absent versus null values. The first mismatch reports its match
configuration and JSON Pointer. `--compare-limit` bounds the number of matches;
an optional `--out` saves the verification summary. Subset reports are supported.

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

The opt-in TypeScript adapter is `src/game/native-engine.ts`. Transfer declarations
in `native-types.ts` are generated from Rust, with no independently authored type
schema. The current live controller and renderer remain on the previous engine
until the next integration stage; the adapter does not alter artwork or UI.

```sh
npm run bindings:generate
npm run check:bindings
npm run check:wasm
```

## Verification and bounded benchmarks

`npm run verify` batches style, UI/types, Rust tests, generated bindings, WASM,
artwork, Vitest and the frontend build. `npm run format`/`style` include Rust and
authored JSON. Focused Rust tests cover loading, rules, commands, saves, scheduling,
recording and error paths; they do not launch campaign/tournament batches.

`npm run ai:benchmark -- --limit 12 --out artifacts/sample.json` runs a bounded
native sample. `--compare artifacts/sample.json` checks complete result hashes for
identical configurations; `--alternate` changes seed, initiative and campaign
policy/diplomacy. The Rust scheduler supplies the sample configurations.

Historical complete TS dumps and their frozen sources remain in local artifacts.
The latest 11-minute run is `artifacts/ai-performance/full-final/report.json`, with
12,240 indexed detail records. Its matching JSON reference is under
`full-final/rust-reference-data`. On 2026-10-06, the refactored native engine
matched all 12,240 complete records in 110.63 seconds with four workers, using the
semantic comparison rules above. The [verification summary](../docs/simulations/rust-refactor-parity-20261006.json)
records the exact content and engine fingerprints.
