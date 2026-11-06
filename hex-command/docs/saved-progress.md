# Local saved progress

Saved progress is local to a browser profile. Saving and loading use browser
storage directly and do not require a network service. Clearing site data removes
saved progress.

Saved games are opened from Settings. Each playthrough has an inline disclosure:
its heading unfolds and folds the history in place. Games sort by their latest
save and remain separate even when they use the same map. The heading shows the
opening save time, or the earliest retained save if the opening was removed.

Turns form a flat newest-first list. Campaign rows retain chapter numbering and
names (for example, `II: The Toll Road, 5`); standalone scenarios show only the
turn number. Load and magnifying-glass buttons stay visible and do not move when
details open. Each detail section shows saved gold, player-controlled units by
level, captured villages and visible completed objectives. Summaries are read on
demand from the selected checkpoint through normal migration/native validation;
they neither advance nor replace current play and add no persisted format fields.
Objective counts use current `progress.done`, including objectives that can become
incomplete again. Permanent completion history remains internal to gameplay; the
inspection API exposes no separate historical completion flag.

Load, individual deletion and Delete all campaign saves require a confirmation
that identifies their target. Confirmation initially rejects rapid follow-up taps.
Individual deletion is secondary inside the summary; whole-campaign deletion is
a quiet action after that playthrough's turns. Settings also exposes Save game,
which records a turn snapshot without move undo. Manual saves are labelled and
are exempt from automatic pruning.

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

Automatic historical payloads are immutable turn-start checkpoints, including opening
mission dialogue and campaign progress. Completed battles remain recoverable as
the current continuation; they do not create a mid-turn historical checkpoint.
There is one exact-state current recovery, including undo, alongside any
explicitly requested manual checkpoints. Starting another run or
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

Automatic thinning uses save time across branches within each game. The latest
five minutes remain dense, followed by one save per five minutes up to an hour,
one per hour up to a day, one per day up to a week, and one per week up to thirty
days. Older ordinary turn saves expire. Chapter starts, manual saves, the latest
automatic turn in each game, and the sole current recovery remain untouched by
automatic cleanup. Retention maintenance runs before and after turn insertion so
eligible old history can free space before a new payload is written. No-op
maintenance does not rewrite the index.

Individual historical saves can be deleted manually. Delete campaign saves
removes the selected playthrough's entire history and any stored current recovery
belonging to it, with no protected checkpoints. If it is the locally played game,
a prominent red warning explains that deletion ends play and opens the existing
New game UI. Confirming cancels pending work and prevents autosaves from recreating
the deleted session. A small browser marker retains the choice of New game after
reload until another game is saved. Deleting an inactive game leaves current play
running and automatically attempts to save pending progress. The storage-full
warning clears only when saving succeeds; no success banner is added.

History writers accept opaque turn-boundary or explicitly requested manual
turn snapshots, all using the same snapshot-only format without undo. They reject
current recoveries and arbitrary serialized bytes. Current recovery
has a separate metadata query and is never an entry in the history index.

Each record carries stable run, continuation, checkpoint and parent identities.
These identities and self-contained payloads leave room for a future player-private
sync transport with divergent continuations. No sync, account credentials, planner
state or network-dependent code is part of local persistence today.

## Save migrations

The current battle envelope is version 37 and the history index is version 4.
The explicit 36→37 and index 3→4 steps collapse the earlier manual variant into
the shared turn-snapshot format, discard historical undo, and record manual status
only as retention/presentation metadata. Current recovery retains its undo.
The explicit 34→35 conversion resolves missing identities, setup, experience and
combat RNG from frozen defaults. It removes the old imported-position variant and
discards old undo. Native restoration validates the converted state without
repairing it or consulting historical formats.

The explicit history-index 1→2 conversion retains genuine turn starts and removes
archived recoveries and imported positions. Their payloads are removed only after
the converted index is committed. Automatic history cleanup leaves the sole current recovery untouched. This conversion runs under the same short-lived history lock as normal
history transactions; it never reconstructs unavailable turn starts.

Older turn payloads pass through the versioned migration chain when selected,
then their converted current-format payload is committed before restoration.
If committing a converted recovery fails, the loading boundary can still return
its canonical ownership metadata through the same migration chain without a write.
Cleanup resolves ownership before deleting any history. Deleting another campaign
retries the retained recovery; deleting its own campaign ends the session and opens
New game. Opening save management dismisses any pending Load confirmation.
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
