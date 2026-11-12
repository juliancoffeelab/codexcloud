# Rust game core

[Authored JSON](data/README.md) supplies rules and scenario content directly.
The internal `catalog` crate owns definitions, snapshots and authoring validation;
`data` re-exports the model vocabulary. The `battle` crate owns mechanical state,
commands and campaign execution; `ai` owns recruitment, campaign orders and
planning. `simulation` owns scheduling,
workers and records; `protocol` defines the browser transfer contract.

A host owns `Catalog` and lends it to operations. Native workers share an immutable
catalog through scoped threads, while each match owns state and planning workspace.
Immutable geometry, effective missions and distance fields use local `Rc` inside
one match; the core needs no shared mutable catalog or synchronization locks.

The internal `freeze-derive` crate derives static mirrors of the current model
fields and exhaustive conversions back to owned values. The catalog compiler
validates the authoring documents and emits these mirrors as borrowed constants.
The browser factory constructs its owned catalog from them once, retaining the
existing gameplay types. Simulation-only tables are omitted from browser builds;
the authoring loader is available there only with `runtime-catalog`. Static mirror
types are not part of the saved-game or browser transfer format.

## Core execution

Native `Catalog::load_from` and `Catalog::from_json_bundle` validate content and resolve
terrain/race/unit inheritance once. `Game` executes typed commands and owns battle
state. Occupancy and unit indexes avoid roster scans; terrain revisions invalidate
cached route fields.

AI tactics compose a recruiter, campaigner and planner. Each receives a restricted
view and contributes to a shared turn draft. Native matches execute that draft;
the browser consumes its command queue through revision-checked battle handles.

Condition evaluation borrows current battlefield facts through the catalog's shared
query interface. Remembered fog has a derived tile index; saved games retain the
original durable condition keys. Traversal updates remember new facts without
constructing discarded presentation snapshots. Immutable tile fog conditions use
`Arc` so snapshots share their trees while the catalog remains safe to share
across native simulation threads. Their serialized shape is unchanged.

## Browser contract

The optional `wasm` feature exposes `NativeEngineFactory`: catalog, create and
restore; battle handles expose views, precomputed action inspection, dispatch, AI
commands, save and disposal. Revision checks reject stale decisions. Saved games
contain only game state and seed. Browser saves bracket the whole enemy turn;
commands and planning workspaces are never persisted.

Rust exports in `src/wasm.rs` use `ocamljs_bindgen` and `ocamljsify` to generate
native OCaml records, variants, results and opaque resource modules.
`ocamljs_pack` emits the Wasm binary and matching bindings/runtime into Git-ignored
`rust/pkg/`. The build copies `Bindings.ml/.mli` to `frontend/Domain.ml/.mli`;
`frontend/Native.ml` owns the generated factory and battle handles.

`Native.ml` applies revision-checked updates and builds detached OCaml
publications from state and precomputed Rust inspection. Unchanged terrain and
records remain shared across publications. Native collections retain ordinary
OCaml semantics. Rendering reads the publications rather than the mutable engine.
The Rust buffered encoder borrows current state during serialization; the
`BorrowFields` derive checks incremental projections against the owned model.

The frontend uses OCaml/Melange and ReasonReact with Preact compatibility imports.
`frontend/Session.ml` owns commands, undo, saves, pacing and cancellable enemy
phases. Moves, combat, recruitment, promotion, enemy planning and campaign
transitions execute in Rust. Undo restores a native checkpoint. Enemy decisions
are prepared during pacing windows and applied only while the session and
revision still match.

Development and verification commands prepare matching generated artifacts;
production builds also run the pinned Binaryen optimization. See [verification](../.agents/skills/verification/SKILL.md) for checks and
`scripts/build-wasm.sh` for WASM preparation.

Browser persistence has one versioned envelope defined by `src/save/schema.ts`,
with browser I/O and lifecycle owned by `frontend/Save.ml` and `Session.ml`. The generated
`SavedGame` checkpoint contains only seed and resolved game state;
it has no separate format version. Historical conversion runs before native
restore. Rust validates current battle invariants and never migrates browser saves.
See [save migrations](../.agents/skills/save-migrations/SKILL.md).

## Tests and simulations

Use the [simulation guide](../.agents/skills/combat-simulation/SKILL.md) for
commands, comparisons and interpreting evidence, and the
[development guide](../docs/development.md) for verification and publication.
