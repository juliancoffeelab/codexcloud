# Local saved progress

Saved progress is local to a browser profile. Saving and loading use browser
storage directly and do not require a network service. Clearing site data removes
saved progress.

Saved games are opened from Settings. They use authored campaign and map names,
chapter groups and compact rows whose load/delete actions appear locally when
selected. Current recovery is not listed as a second historical checkpoint.
Deletes require confirmation.

A failed write leaves the previous committed recovery available. A dialog explains
whether storage is full, access is blocked, or an internal error occurred. Capacity
failures offer save management; cleanup automatically saves pending progress when
possible. The player may acknowledge a failure and continue without repeated
interruptions for the same condition. There is no manual retry control. Other
failures provide appropriate browser guidance or access to the debug report.

## Recovery and history

`hex-command-battle` is the atomic current recovery envelope. Completed player
actions write the complete native checkpoint together with compact backward
changes for the available move undo. Unchanged terrain and rosters are stored
once. Combat, recruitment, promotion, item drops and turn changes retain
their existing undo barriers. UI selection and AI planner workspaces are not
saved. An enemy turn retains the pre-turn recovery with `pendingEnemy: true`
throughout its actions, then writes the fully resolved result. Enemy-first games
retain the untouched opening enemy state. Reload resumes from these safe points;
updates may migrate the underlying state and discard undo.

Historical payloads are immutable turn-start checkpoints, including opening
mission dialogue and campaign progress. Completed battles remain recoverable as
the current continuation; they do not create a mid-turn historical checkpoint.
There is one exact-state current save, including undo. Starting another run or
loading earlier progress replaces it; it must not archive the outgoing recovery.
Previously recorded turn starts remain accessible when play continues differently.

Old saved data is handled exclusively by explicit version-to-version migrations.
Runtime initialization must not adopt an old save into historical records or
invent an unavailable turn start.

## Storage and retention

`src/save/progress.ts` owns the local repository operations. Its versioned
metadata index is separate from checkpoint payloads, so browsing reads labels and
timestamps without deserializing every battlefield. The dialog renders thirty
entries at a time. Recovery writes do not scan or rewrite historical payloads.
A history insert writes its payload before publishing its index entry; deletions
remove index entries before removing payloads. Interrupted operations can leave
unreferenced bytes but never publish an absent payload. Each payload replacement
and current recovery replacement is an atomic browser storage write.

Automatic thinning runs at turn boundaries. It is optional maintenance: a failure
does not invalidate an already committed checkpoint, and its error is included in
the debug report. It retains every one of the latest 24 turn checkpoints in each continuation. Older checkpoints are selected by a
stable turn ordinal at intervals of 2, 4, 8 and so on in progressively older age
bands. Mission starts are always exempt from automatic thinning. Whole-run cleanup
protects only the current recovery. Historical turn starts, including the current
turn’s start and mission starts, can be deleted manually. Retained payloads are
complete native snapshots, so a missing ancestor does not prevent loading them.

History accepts only immutable snapshots captured at real turn boundaries; its
writer cannot accept a current save or arbitrary serialized bytes. Current recovery
has a separate metadata query and is never an entry in the history index.

Each record carries stable run, continuation, checkpoint and parent identities.
These identities and self-contained payloads leave room for a future player-private
sync transport with divergent continuations. No sync, account credentials, planner
state or network-dependent code is part of local persistence today.

## Save migrations

The current battle envelope is version 35 and the history index is version 2.
The explicit 34→35 conversion resolves missing identities, setup, experience and
combat RNG from frozen defaults. It removes the old imported-position variant and
discards old undo. Native restoration validates the converted state without
repairing it or consulting historical formats.

The explicit history-index 1→2 conversion retains genuine turn starts and removes
archived recoveries and imported positions. Their payloads are removed only after
the converted index is committed. The sole current recovery is untouched by history
cleanup. This conversion runs under the same short-lived history lock as normal
history transactions; it never reconstructs unavailable turn starts.

Older turn payloads pass through the versioned migration chain when selected,
then their converted current-format payload is committed before restoration.
No historical shape is interpreted by gameplay, browsing or repository operations.
Conversion code and frozen historical data remain under `src/save/migrations/`.

Undo validation transfers the current state and compact changes to Rust once.
Rust reconstructs and validates every earlier state against the native rules
without creating battle handles, AI workspaces, geometry publications, combat
forecasts or presentation views. JavaScript reconstructs immutable undo states
with structural sharing, retaining unchanged terrain/roster records. Saving caches
the changes between older undo entries and refreshes only the latest backward
changes against current play, including any intervening dialogue changes. Undo
uses native restoration before applying a prior state, preserving validation at
both loading and execution.

Version 34 integrates objective history from the independently developed objective
branch. It preserves that branch’s already-recorded completed/revealed arrays,
and converts saved-progress versions 31–33 with the frozen historical objective
recipes. A partial objective-history shape is rejected. Conversion preserves
progress identities and native facts and discards older undo; it never executes
historical gameplay. This also applies when selecting an older historical save.
