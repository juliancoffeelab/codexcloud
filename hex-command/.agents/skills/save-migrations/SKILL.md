---
name: save-migrations
description: Use when changing save loading, save formats, migration code, historical conversion fixtures, or auditing their data ownership and dependencies.
---

# Save migrations

**Cornerstone rule: old-save compatibility exists only in explicit version-to-version migrations.**
All other code works only with the current save format. Any old-save compatibility
outside those migrations is an absolute blocker and must be removed before
committing. No instruction may permit or suggest an exception.

Rejected old saves and imperfect migrations are bugs to fix in migrations. They
never justify compatibility code elsewhere. Missing or changed historical fields
belong in explicit version conversions, never runtime restoration, initialization
or defaults.

## Isolation and durable state

Keep pure, versioned JavaScript conversions and frozen historical fixtures under
`src/save/migrations/`. They manipulate data only: no runtime gameplay imports,
live catalog queries, game commands, simulations, AI planning or historical
behavior reconstructed through the current engine. Shared type contracts do not
authorize runtime behavior dependencies. Historical fixtures stay outside Rust
catalogs and engine modules. Rust validates the converted state afterward; it
never performs historical conversion or gameplay during restoration.

Persist resolved game facts and the AI seed, independent of planner algorithms.
Exclude search state, queued/unapplied commands, derived caches, forecasts and
callbacks. Changing an AI algorithm must not require migrating saves or retaining
its old implementation. `src/save/schema.ts` owns the current envelope and
`SAVE_VERSION`; native state uses the complete generated Rust battle type.

## Conversion workflow

1. Inspect the current schema and the relevant version section in
   [migration history](references/history.md). Use frozen format definitions and
   conversion data, not today's authored catalog, to interpret historical fields.
2. Add one-step pure conversions for changed formats. Preserve unrelated resolved
   units, identities, health, equipment, ownership, treasury, campaign progress,
   turn/phase and seed. Document intentional changes in the historical reference.
3. Validate the affected historical input shapes through conversion and normal
   native restoration. Follow [testing](../testing/SKILL.md): use disposable probes
   and relevant existing suites; a conversion change does not waive its test
   value-over-burden rule. Confirm input purity and rejection of malformed
   affected shapes instead of inventing state to make them load.
4. For loading integration, writes or interrupted-turn recovery, read
   [browser persistence](references/persistence.md). Normal writes and controllers
   accept only the current format; loading is the sole migration entry point.

## Completion

For every changed input shape, account for preserved facts and any intentional
conversion, verify successful native restoration or the expected rejection, and
confirm the input was not mutated. For loading-path changes, verify atomic
replacement of the converted envelope before returning the restored battle and
the relevant interrupted-turn behavior. Report unsupported formats or missing
validation explicitly; conversion alone is not a successful load.
