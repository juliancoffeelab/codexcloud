# Saved progress requirements

Requirements for [issue #46](https://github.com/juliancoffeelab/codexcloud/issues/46).

## Functional requirements

1. **Recover current play.** After a completed action at a safe save point, refreshing or reopening restores the current battle, campaign state, and available move-undo history. Existing undo barriers remain intact. Migrations may discard undo history.
2. **Keep enemy turns atomic.** An enemy turn is either saved in full or not saved at all. Partial enemy-turn execution must not be persisted.
3. **Keep independent runs.** Starting another campaign or skirmish must preserve existing runs.
4. **Revisit previous turns.** Automatic historical checkpoints restore the start of a turn, including the campaign state reached at that point. Explicit saves record a turn snapshot without undo. Only the current flash save retains available move undo.
5. **Preserve alternative continuations.** Returning to an earlier checkpoint and playing differently must leave previously reached later progress accessible.
6. **Browse saved progress.** Show saves ordered by save time, with enough context—run, mission, and turn—to identify them.
7. **Manage storage.** Allow deleting individual historical checkpoints or every save in a selected run. Deleting the currently played run warns the player, ends that session and opens New game. Storage failures must provide an error and access to cleanup tools.
8. **Retain history selectively.** Automatic history may keep recent turns densely and older turns more sparsely. Age is measured by save time across branches within each run. Preserve chapter starts, explicit manual saves and the latest automatic turn per run during automatic pruning; explicit deletion has no exemptions.
9. **Support offline play.** Local saving and loading must remain usable without a network connection or cloud service.
10. **Expose local persistence status.** Distinguish successfully saved progress from local-save failure.
11. **Migrate saves.** Current flash save data and historical checkpoints follow the supported migration path after updates. Undo history may be discarded during migration.

Backup and export tools are optional future features.

## Non-functional requirements

- **Responsiveness:** Saving, browsing, restoring, and syncing must keep gameplay responsive as runs grow. Cloud latency must not block local play. Browsing should not require restoring every checkpoint.
- **Durability and consistency:** A successful local-save indication means the flash save data was committed. Battle state and undo history must remain consistent; interrupted writes or syncs must not silently destroy previously saved progress.
- **Resilience:** Storage exhaustion, unavailable storage, network failures, and cloud outages produce understandable states and recoverable workflows.
- **Usability:** Players can identify saves, understand persistence failures, choose conflicting continuations, and reclaim storage without needing implementation knowledge.

References to syncing, cloud failures, and conflicting device continuations apply when cloud support is implemented.

## Cloud saves — postponed

The current implementation must leave a viable path to these capabilities:

1. **Sync across devices.** Automatically make a player’s saved runs and checkpoints available on their other devices.
2. **Sync after offline play.** Pending changes sync when connectivity returns.
3. **Preserve conflicting progress.** Divergent device progress must remain recoverable, with a way to choose which continuation to play.
4. **Expose persistence status.** Distinguish locally saved, pending sync, synced, local-save failure, and sync failure.
5. **Security and privacy:** Cloud saves are associated with the correct player and accessible only to that player.
