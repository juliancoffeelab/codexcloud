# Browser saves and migrations

`src/save/schema.ts` defines the current version-2 envelope:

```json
{
  "version": 2,
  "pendingEnemy": false,
  "checkpoint": { "seed": 17, "state": {}, "planner": {} }
}
```

`state` is the complete generated Rust battle type. `planner` is optional and
preserves exact enemy AI continuation, including a selected but unapplied command.
The checkpoint itself has no format version. `pendingEnemy` instead records an
interrupted enemy phase that must replay from a player-turn checkpoint.

Loading parses JSON, runs consecutive migrations, validates and restores through
Rust, then writes the current envelope. Migration steps are pure data conversions;
new versions require another one-step module under `src/save/migration/`. Normal
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
