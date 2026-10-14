# Browser saves and migrations

`src/save/schema.ts` defines the current version-5 envelope:

```json
{
  "version": 5,
  "pendingEnemy": false,
  "checkpoint": { "seed": 17, "state": {} }
}
```

`state` is the complete generated Rust battle type. Native checkpoints contain
only resolved game state and the AI seed. They never contain planner workspaces,
queued actions or selected but unapplied commands.

The entire enemy phase is atomic for browser persistence: save the player state
with `pendingEnemy: true` before starting it, retain that checkpoint throughout
all enemy actions, then replace it with the resolved state after the phase ends
or the battle finishes. Reloading an interrupted phase replays the whole phase
from its pre-turn checkpoint. A single `localStorage.setItem` replaces the complete
envelope; failed writes leave the previous checkpoint intact. Storage is optional,
so persistence cannot be guaranteed when the browser refuses writes.

Loading parses JSON, runs consecutive migrations and validates through Rust before
writing the current envelope. Migration steps are pure data conversions; new
versions require another one-step module under `src/save/migration/`. Normal
writes and the controller accept only the current format and native engine.

Rejected saves retain their original bytes in `hex-command-battle-unrestored` and
leave a playable fresh battle. If the backup cannot be stored, that recovery
battle cannot overwrite the original; starting a new battle enables saving again.
Disabled or full browser storage does not stop play.

## Version-one compatibility

Both previous formats were version one. TS snapshots have a top-level `version`
and store ownership without terrain. Native envelopes have `engine: "rust-wasm"`
and a version inside `saved`. The v1-to-v2 step distinguishes these formats and
removes their format markers. Historical layouts, defaults and campaign
conversions live exclusively under `src/save/migration/legacy/`.

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
Revision-one river and refuge checkpoints retain their own tiles, ownership, units
and planner continuation, while their revision tag is changed to the current
catalog entry with the same geometry. Normal Rust restoration validates the
result. The catalog accepts only current authored documents; old layouts, marker
indices and dialogue are stored under `src/save/migration/legacy/data/`.

## Version-three migration

The v3-to-v4 step grows the developer map's published 6×20 and 13×23 layouts to
the current 32×32 revision. Existing tiles, ownership, units, battle progress,
seed and planner continuation stay intact; only missing tiles are added from
the current map. It never resets units or respawns defeated showcase troops.
Malformed or incomplete historical boards still fail validation. Other maps
receive only the envelope-version update.

## Version-four migration

The v4-to-v5 step drops the entire historical `planner` field, including all
queued actions, pending commands and continuation bookkeeping. It preserves
resolved battle state and the pre-turn replay marker. Migration never executes
AI actions or deserializes the historical planner. Normal gameplay creates a
fresh plan when needed.

Older native saves may contain resolved state halfway through an enemy phase.
They have no pre-turn state to roll back to: completed actions remain, while
unexecuted actions are discarded. Such saves keep their original storage bytes
until normal gameplay finishes the phase and writes a version-five boundary.
New saves never persist partial enemy phases.
