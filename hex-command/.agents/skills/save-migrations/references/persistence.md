# Browser persistence

Read when changing loading, import, deletion of blocked saves, normal writes or
interrupted-turn flash saves. Read `packages/save-migrations/ocaml/Migration_envelope.ml` for current envelope fields and
`frontend/persistence/Save.ml` and `frontend/persistence/Save_storage.ml` for storage and loading.

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

Loading a game parses and converts its save, validates it through Rust, then
atomically replaces the stored envelope before returning the restored battle.
This write is independent of gameplay saving and enemy-turn completion. Invalid
stored data is retained for explicit deletion; fresh in-memory play remains
available. Disabled or full browser storage does not stop play.

## Storage transition

Storage v1→v2 converts localStorage records to IndexedDB. Pure conversion lives in
`packages/save-migrations/ocaml/Storage_migrations.ml`; `frontend/persistence/Save.ml` owns source I/O.
`Save_migrations` owns full payload conversion for loading and import. Validate
all converted retained saves natively, then commit records and the
initialization marker together. Remove the source only after commit. An
initialized database is authoritative even when empty: discard subsequent
localStorage saves without reimporting them. Failed import retains the source and
opens a deletion-only interface. It retains rejected saves unchanged and offers
explicit record, campaign or complete-data deletion. Only a save accepted by the
ordinary typed migration chain can establish campaign ownership. Literal index
and payload claims may expose conflicts or uncertainty; they never authorize
campaign deletion. Unknown or disputed records require individual deletion or a
complete reset. Do not reconstruct rejected saves or rebuild indexes from them.
Deleting a record removes its list references without converting the remaining
rows. Normal save management accepts initialized, migrated data only.

Source import and deletion share a short-lived Web Lock across tabs and recheck
initialization before touching source records. Fresh in-memory play remains
available while blocked source data awaits deletion; its automatic writes stay
suspended. Storage unavailability without source saves is an ordinary save error
and cannot block starting or playing a campaign.
