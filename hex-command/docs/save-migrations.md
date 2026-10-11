# Browser saves and migrations

`frontend/Save.ml` defines the current version-4 envelope:

```json
{
  "version": 4,
  "pendingEnemy": false,
  "checkpoint": { "seed": 17, "state": {}, "planner": {} }
}
```

`state` is the complete generated Rust battle type. `planner` is optional and
preserves exact enemy AI continuation, including a selected but unapplied command.
The checkpoint itself has no format version. `pendingEnemy` instead records an
interrupted enemy phase that must replay from a player-turn checkpoint.

Loading parses JSON, runs consecutive migrations, validates and restores through
Rust, then persists the current envelope if migration was needed. Migration steps
are pure data conversions in `frontend/Save.ml`; historical snapshot conversion
is isolated in `frontend/Save_legacy.ml`. New versions extend that consecutive
migration chain. Normal writes and the session accept only the current format
and native engine.

Rejected saves are backed up with their exact original bytes in
`hex-command-battle-unrestored` when storage permits and leave a playable fresh battle. If the backup cannot be stored, that recovery
battle cannot overwrite the original; starting a new battle enables saving again.
Disabled or full browser storage does not stop play.

An accepted native command whose publication fails is terminal for that live
battle handle. The session retains its last coherent view, cancels deferred work
and presents reload; further commands reject, preserving the last saved bytes
for recovery. Ordinary native command rejection leaves the live battle usable.

## Version-one compatibility

Both previous formats were version one. TS snapshots have a top-level `version`
and store ownership without terrain. Native envelopes have `engine: "rust-wasm"`
and a version inside `saved`. The v1-to-v2 step distinguishes these formats and
removes their format markers. `Save_legacy.ml` owns historical defaults and
campaign conversion; historical layout, roster and stage JSON lives under
`src/save/migration/legacy/data/`.

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
