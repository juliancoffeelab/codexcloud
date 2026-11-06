# Browser persistence

Read when changing loading integration, normal save writes or interrupted-turn
recovery. Read `src/save/schema.ts` for current envelope fields and
`src/save/` for the storage implementation.

Opening side, AI selection and configured skirmish armies are durable user setup,
not planner workspaces. Preserve those choices across missions, retries and
restoration. Campaigns retain their authored recruitment pools; read the schema
for current setup and army field shapes.

## Atomic enemy turns

An enemy-first skirmish saves its untouched initial enemy state before planning;
reloading resumes that opening phase from the same checkpoint. Thereafter the
entire enemy phase is atomic for browser persistence: save the player state
with `pendingEnemy: true` before starting it, retain that checkpoint throughout
all enemy actions, then replace it with the resolved state after the phase ends
or the battle finishes. Reloading an interrupted phase replays the whole phase
from its pre-turn checkpoint. A single IndexedDB transaction replaces the complete
envelope; failed writes leave the previous checkpoint intact. Capture action state
before queuing writes, and await the boundary write attempt before enemy work. Storage is optional,
so persistence cannot be guaranteed when the browser refuses writes.

## Loading integration

Loading is the sole migration entry point: parse, convert, validate through Rust,
then atomically replace the stored envelope before returning the restored battle.
This write is independent of gameplay saving and enemy-turn completion. Invalid
saves are discarded; play and saving resume with a fresh battle. Disabled or full
browser storage does not stop play.

## Storage transition

Storage v1→v2 converts localStorage records to IndexedDB. Pure conversion lives in
`src/save/migrations/storage-v1-to-v2.ts`; `src/save/import.ts` owns source I/O.
Validate all converted retained saves natively, then commit records and the
initialization marker together. Remove the source only after commit. An
initialized database is authoritative even when empty: discard subsequent
localStorage saves without reimporting them. Failed import retains the source;
cleanup tools may operate on canonical migration output and retry import, while
normal gameplay repository operations require initialized IndexedDB. Source import
and source cleanup share a short-lived Web Lock across tabs; failed-import tabs
recheck initialization before touching the source.
