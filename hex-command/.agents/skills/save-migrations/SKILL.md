---
name: save-migrations
description: Use when changing save loading, save formats, migration code, historical conversion fixtures, or auditing their data ownership and dependencies.
---

# Save migrations

Read this skill before changing save loading, save formats, migration code or
historical conversion fixtures, including changes proposed during data-ownership
audits.

Backfilling or defaulting fields to make older saves compatible is migration work,
regardless of whether it is called restoration, initialization or authored-default
resolution.

## Isolation

Migration conversion code and its historical fixtures must stay self-contained
under `src/save/migrations/`. Do not import or call runtime gameplay modules,
query the live game catalog, or fetch current scenario definitions to perform a
historical conversion. Use frozen, versioned conversion data stored alongside the
migration. Shared type contracts do not authorize runtime behavior dependencies.

Historical conversion fixtures are migration data, not current authored game
data. Keep them out of Rust game catalogs and engine modules. Rust validates the
converted state after migration; validation must not run historical gameplay.

## Save contract

`src/save/schema.ts` defines the current version-9 envelope:

```json
{
  "version": 9,
  "pendingEnemy": false,
  "checkpoint": { "seed": 17, "state": {} }
}
```

`state` is the complete generated Rust battle type. Native checkpoints contain
only resolved game state and the AI seed. They never contain planner workspaces,
queued actions or selected but unapplied commands.

The optional envelope `setup` records the opening side and enemy AI selection
(policy and optional planner/recruiter overrides). Saves without it retain Classic
AI. These are user preferences, not planner workspaces; they persist across
campaign missions and retries. Configured skirmish faction IDs live in native
`state.armies` so recruitment survives native state restoration. Campaigns retain
their authored pools.

The save format is a data contract, independent of planner implementation.
Persist durable game facts such as units, health, gold, ownership, completed
events and turn counters. Do not persist search cursors, AI orders, derived
caches, forecasts, callbacks or other objects whose meaning requires running
the logic that created them. Changing an AI algorithm must not require a save
migration or retaining its old implementation.

An enemy-first skirmish saves its untouched initial enemy state before planning;
reloading resumes that opening phase from the same checkpoint. Thereafter the
entire enemy phase is atomic for browser persistence: save the player state
with `pendingEnemy: true` before starting it, retain that checkpoint throughout
all enemy actions, then replace it with the resolved state after the phase ends
or the battle finishes. Reloading an interrupted phase replays the whole phase
from its pre-turn checkpoint. A single `localStorage.setItem` replaces the complete
envelope; failed writes leave the previous checkpoint intact. Storage is optional,
so persistence cannot be guaranteed when the browser refuses writes.

Loading parses JSON, runs consecutive migrations and validates through Rust before
writing the current envelope. Migration steps are pure data conversions; new
versions require another one-step module under `src/save/migrations/`. Normal
writes and the controller accept only the current format and native engine.

Migrations run in JavaScript and may only manipulate data. They must not execute
game commands, run simulations, invoke AI planners or reconstruct historical
behavior through the current engine. Use explicit format versions and documented
historical data to convert fields. Rust restoration validates the converted game
state afterward; it is not a migration step. AI planning starts through normal
gameplay after restoration.

After successful validation, the loader immediately replaces the stored envelope
with the converted data in one atomic write. This conversion must not go through
the gameplay save function or wait for an enemy turn to finish. If validation
fails, discard the save and start a fresh battle.

All migration implementations, dispatch and historical conversion data belong
exclusively in `src/save/migrations/`. Loading is the only runtime migration entry
point. The loader parses, migrates, validates and commits before returning the
restored battle. Unparseable, unsupported or invalid saves are wiped, along with
obsolete rejected-save backups; start a fresh battle without suspending saving.
Disabled or full browser storage does not stop play.

## Version-one compatibility

Both previous formats were version one. TS snapshots have a top-level `version`
and store ownership without terrain. Native envelopes have `engine: "rust-wasm"`
and a version inside `saved`. The v1-to-v2 step distinguishes these formats and
removes their format markers. Historical layouts, defaults and campaign
conversions live exclusively under `src/save/migrations/legacy/`.

TS saves tagged `campaignDeploymentFormat: "named-waves-v1"` already use authored
wave IDs. Untagged published saves from `3956f5c` and `7a30040` translate numeric
markers through each wave's explicit `legacyIndex`. Revision-one Lantern village
and Ashen finale saves already contained subsequently delayed reinforcements;
the importer marks those waves deployed without changing wounded troops, gold
or IDs. Revision-two saves retain pending waves.

The unpublished intermediate draft `1acc1dd` reused a revision with a different
Ashen numeric order. Its format cannot be distinguished reliably from an older
published save, so that ambiguous draft is not supported. Wave identity is never
inferred from surviving troop health.

## Version-two migration

The v2-to-v3 step removes the need for historical layouts in the live catalog.
Revision-one refuge checkpoints retain their own tiles, ownership and
units, while their revision tag is changed to the current catalog entry with the
same geometry. Normal Rust restoration validates the
result. The catalog accepts only current authored documents; old layouts, marker
indices and dialogue are stored under `src/save/migrations/legacy/data/`.

## Version-three migration

The v3-to-v4 step grows the developer map's published 6×20 and 13×23 layouts to
the current 32×32 revision. Existing tiles, ownership, units, battle progress
and seed stay intact; only missing tiles are added from the current map.
It never resets units or respawns defeated showcase troops.
Malformed or incomplete historical boards still fail validation. Other maps
receive only the envelope-version update.

These historical intermediate conversions may carry an opaque `planner` field
through the migration chain. The v4-to-v5 step discards it before native
restoration. Carrying those bytes does not permit interpreting or resuming them.

## Version-four migration

The v4-to-v5 step drops the entire historical `planner` field, including all
queued actions, pending commands and continuation bookkeeping. It preserves
resolved battle state and the pre-turn replay marker. Migration never executes
AI actions or deserializes the historical planner. Normal gameplay creates a
fresh plan when needed.

Older native saves may contain resolved state halfway through an enemy phase.
They have no pre-turn state to roll back to: completed actions remain, while
unexecuted actions are discarded. Their converted data is persisted immediately
after validation, without playing any remaining enemy actions.
This is recovery of a historical format, not permission to save partial phases.
New saves must persist only the boundaries of the entire enemy turn, even when
that turn contains many units and commands.

## Version-five migration

The v5-to-v6 step turns historical implicit type-based leadership into explicit
`role: "leader"` metadata on unit instances. It uses frozen pre-conversion type
IDs stored with the migrations and preserves every existing explicit role.
Current unit definitions have no leadership flag. Rust, AI, UI and current saves
use only the unit's role; ordinary units of the same type do not become leaders.

## Version-six migration

The v6-to-v7 step makes faction data explicit. A frozen conversion fixture supplies
The Silent Crossing's wildlife controller and assigns its existing animals to that
faction, preserving identity, wounds, equipment and progress. Other maps retain
explicit faction data or receive an empty list. Current snapshots always serialize
`factions`, including an empty list, and Rust rejects omitted fields or missing
required authored factions instead of backfilling from the current map.

Older conversion steps read migration-owned metadata frozen from deployed source
`14371387185340e64c5b68706faa409ebe16fdf8`. That fixture is historical conversion
data; do not regenerate it when current maps, unit rules or campaigns change.

## Version-seven migration

The v7-to-v8 step adds the required `groundItems: []` battlefield inventory.
Version-seven checkpoints had carried equipment but no ground items. Conversion
preserves all existing units, equipment and campaign progress. Current snapshots
serialize ground items explicitly, including an empty list; Rust does not invent
missing state during restoration.

## Version-eight migration

The v8-to-v9 step converts Toll Road revision-three checkpoints to revision four
using migration-owned historical encounter identities. It removes uncontacted
preplaced Sella, records an already-completed meeting’s spawn as resolved, and
preserves a joined Sella and every other existing patrol unit. Existing camp
ownership is retained. The old proximity light memory is discarded, swamp entry
memory is translated to the shared encounter marker, and the original mire
caster receives its authored name without changing identity or progression.
Other chapters receive only the envelope update. No runtime restoration backfill
or live catalog lookup performs this conversion.
