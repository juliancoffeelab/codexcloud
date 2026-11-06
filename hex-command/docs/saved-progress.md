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

A failed write leaves the previous committed flash save available. A dialog explains
whether storage is full, access is blocked, or an internal error occurred. Capacity
failures offer save management; cleanup automatically saves pending progress when
possible. The player may acknowledge a failure and continue without repeated
interruptions for the same condition. There is no manual retry control. Other
failures provide appropriate browser guidance or access to the debug report.

## Flash save and history

The IndexedDB `state.current` record is the atomic flash save envelope. Completed player
actions write the complete native checkpoint together with compact backward
changes for the available move undo. Unchanged terrain and rosters are stored
once. Combat, recruitment, promotion, item drops and turn changes retain
their existing undo barriers. UI selection and AI planner workspaces are not
saved. An enemy turn retains the pre-turn flash save with `pendingEnemy: true`
throughout its actions, then writes the fully resolved result. Enemy-first games
retain the untouched opening enemy state. Reload resumes from these safe points;
updates may migrate the underlying state and discard undo.

Automatic historical payloads are immutable turn-start checkpoints, including opening
mission dialogue and campaign progress. Completed battles remain recoverable as
the current continuation; they do not create a mid-turn historical checkpoint.
There is one exact-state current flash save, including undo, alongside any
explicitly requested manual checkpoints. Starting another run or
loading earlier progress replaces it; it must not archive the outgoing flash save.
Previously recorded turn starts remain accessible when play continues differently.

Old saved data is handled exclusively by explicit version-to-version migrations.
Runtime initialization must not adopt an old save into historical records or
invent an unavailable turn start.

## Storage and retention

`src/save/storage.ts` owns the asynchronous IndexedDB boundary; repository
operations live in `src/save/progress.ts`. The `hex-command-saves` database has
separate `state`, `entries` and `turns` stores. Metadata indexes support game and
save-time queries without deserializing battlefields. The dialog renders thirty
entries at a time. Flash save writes do not scan or rewrite historical payloads.

History insertion, replacement and deletion update metadata and payloads in one
transaction. Whole-campaign deletion checks flash save ownership and removes the
matching flash save in that same transaction. Flash save writes clear the idle choice
atomically. Transactions request strict durability; callers report success after
commit. Action checkpoints are captured before queuing, so a delayed write cannot
serialize a later action or another session. Enemy work waits for its boundary
write attempt before proceeding.

Tabs share database transactions without owning gameplay. The last committed
flash save write wins. History transactions operate on current database records,
so competing additions cannot overwrite an entire stale index. A locally deleted
active game cancels that session's pending work. Another active tab may still save
its own game afterward; there is no tab-control or takeover protocol.

Automatic thinning uses save time across branches within each game. The latest
five minutes remain dense, followed by one save per five minutes up to an hour,
one per hour up to a day, one per day up to a week, and one per week up to thirty
days. Older ordinary turn saves expire. Chapter starts, manual saves, the latest
automatic turn in each game, and the sole current flash save remain untouched by
automatic cleanup. Retention maintenance runs before and after turn insertion so
eligible old history can free space before a new payload is written. No-op
maintenance does not rewrite unchanged records.

Individual historical saves can be deleted manually. Delete campaign saves
removes the selected playthrough's entire history and any stored current flash save
belonging to it, with no protected checkpoints. If it is the locally played game,
a prominent red warning explains that deletion ends play and opens the existing
New game UI. Confirming cancels pending work and prevents autosaves from recreating
the deleted session. A small browser marker retains the choice of New game after
reload until another game is saved. Deleting an inactive game leaves current play
running and automatically attempts to save pending progress. The storage-full
warning clears only when saving succeeds; no success banner is added.

History writers accept opaque turn-boundary or explicitly requested manual
turn snapshots, all using the same snapshot-only format without undo. They reject
current flash saves and arbitrary serialized bytes. Current flash save
has a separate metadata query and is never a historical turn entry.

Each record carries stable run, continuation, checkpoint and parent identities.
These identities and self-contained payloads leave room for a future player-private
sync transport with divergent continuations. No sync, account credentials, planner
state or network-dependent code is part of local persistence today.

## Save migrations

The current battle envelope is version 37. The imported localStorage history
index reaches version 4 before being split into IndexedDB records.
The explicit 36→37 and index 3→4 steps collapse the earlier manual variant into
the shared turn-snapshot format, discard historical undo, and record manual status
only as retention/presentation metadata. Current flash save retains its undo.
The explicit 34→35 conversion resolves missing identities, setup, experience and
combat RNG from frozen defaults. It removes the old imported-position variant and
discards old undo. Native restoration validates the converted state without
repairing it or consulting historical formats.

The explicit history-index 1→2 conversion retains genuine turn starts and removes
archived flash saves and imported positions. Retired payloads are excluded from the IndexedDB import. Automatic history
cleanup leaves the sole current flash save untouched. Conversion never reconstructs
unavailable turn starts.

Storage version 1→2 imports localStorage through an explicit pure conversion in
`src/save/migrations/storage-v1-to-v2.ts`. Existing save-envelope and index
migrations produce current envelopes. The storage conversion renames the former
current-save discriminator to `"flash"`, without changing the envelope version.
Native validation checks every retained checkpoint and
flash save undo before writing. All imported records and the initialization marker
commit in a single transaction. No source record is removed before that commit.
An empty database is also initialized explicitly: its lack of current play is
still authoritative. A short-lived Web Lock serializes source import and source
cleanup across new-code tabs; gameplay needs no ownership lock. A failed-import
tab rechecks authority before offering source data or deleting it.

Once initialization commits, IndexedDB is authoritative. Startup discards any
remaining or subsequently written game-save localStorage records without reading
them as saves. Old tabs can continue running, but their later localStorage writes
are discarded on reload. Other preferences, such as sound, remain independent.
There is no forced reload, reconciliation or localStorage gameplay backend.

A failed import leaves the source intact and the database uninitialized. It opens
**Saved data needs repair**, independently of ordinary save-write errors. Normal
Saved games only reads the initialized target repository; it never becomes an
adapter for unmigrated source data.

Repair inventories each source record independently. Readable metadata can identify
a campaign even when its snapshot is malformed. Unreadable records remain visible
by storage key. Individual deletion does not decode a battlefield. Campaign deletion
removes only records whose ownership is established, including its flash save when
known. An unidentified flash save remains untouched, with a warning strongly
recommending explicit deletion. Conflicting identities are treated as unknown.

A damaged or missing history list can be rebuilt from identifiable snapshots.
Unlisted and unidentified payloads are retained for explicit cleanup rather than
silently lost on import. Delete all source saves provides a confirmed reset path.
All destructive actions require confirmation. Cleanup does not import implicitly;
Import remaining saves confirms replacement of any current unsaved play, retries
conversion and native validation, then installs the
retained flash save or opens the existing New game UI when none remains. A tab
whose import completed elsewhere must reopen the authoritative data before repair
can delete anything.

Choose a new game without saving keeps source records untouched and permits normal
in-memory gameplay while repair is pending. Source repair and normal gameplay
remain separate: automatic writes stay suspended until import completes. If storage
is unavailable with no source saves, the ordinary storage error explains the lack
of saving, and campaigns, scenarios and player actions remain usable. Pending
history writes are best-effort when switching games; they cannot veto fresh play.

Subsequent stored-envelope conversions still use the loading boundary and commit
before installing restored play. Their failed writes can use the separate repair
interface, including atomic target-store cleanup. Debug reports retain raw source
and target records. No historical shape is interpreted by gameplay or normal save
management. Data-only repair decoding stays inside the explicit storage migration;
`src/save/repair.ts` owns its I/O and exposes record identities and diagnostics,
never playable historical snapshots.

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
