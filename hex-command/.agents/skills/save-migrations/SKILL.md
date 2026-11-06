---
name: save-migrations
description: Use when changing save loading, save formats, migration code, historical conversion fixtures, or auditing their data ownership and dependencies.
---

# Save migrations

Follow the [old-save cornerstone rule](../../../AGENTS.md): explicit
version-to-version migrations are the only permitted old-save handling.

Read this skill before changing save loading, save formats, migration code or
historical conversion fixtures, including changes proposed during data-ownership
audits.

Missing or changed fields in old saves are handled by those version conversions,
never by restoration, initialization or runtime defaults.

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

`src/save/schema.ts` defines the current envelope and `SAVE_VERSION`. Read that
contract for field shapes and the current version.

`state` is the complete generated Rust battle type. Native checkpoints contain
only resolved game state and the AI seed. They never contain planner workspaces,
queued actions or selected but unapplied commands.

The envelope `setup` records the opening side and enemy AI selection
(policy and optional planner/recruiter overrides). These preferences persist across
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

Add one-step pure conversions under `src/save/migrations/` when the format changes.

Migrations run in JavaScript and may only manipulate data. They must not execute
game commands, run simulations, invoke AI planners or reconstruct historical
behavior through the current engine. Use explicit format versions and documented
historical data to convert fields. Rust restoration validates the converted game
state afterward; it is not a migration step. AI planning starts through normal
gameplay after restoration.

Loading is the sole migration entry point: parse, convert, validate through Rust,
then atomically replace the stored envelope before returning the restored battle.
This write is independent of gameplay saving and enemy-turn completion. Invalid
saves and obsolete rejected-save backups are discarded; play and saving resume
with a fresh battle. Disabled or full browser storage does not stop play.

## Historical reference

See [save migration history](references/history.md) for version-to-version
conversion details.
