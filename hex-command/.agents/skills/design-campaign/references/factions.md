# Faction and identity authoring

Read when changing rosters, diplomacy, recruitment or named participants.
Work from the game root. Use `rust/catalog/src/data/catalog_documents.rs` for
roster loading, `factions.rs` in that directory for faction rules, and current
campaign `roster.json` files for examples.

## Armies and diplomacy

Define starting armies in each stage's `roster.json`, under `factions`.
Placements inherit faction and phase from their containing entry. Map documents
own terrain and presentation. Each faction owns its controller, palette,
relations, treasury and recruitment pool.

Phase, control, palette and diplomacy describe different things. An enemy-phase
faction need not be hostile; a white palette does not make a unit passive.
Author hostility explicitly. Different factions default to neutral, and relation
entries declared on both sides must agree. Allied and neutral settlements are
retained when entered; recruitment spends the recruiting faction's own treasury.

Use a passive controller for nonacting civilians. When a rescue should preserve
a unit, use the existing allegiance mechanism rather than replacing it with a
new spawn. Transfer preserves wounds, equipment and action flags; it grants no
extra actions. Current event actions, including `changeAllegiance`,
`unlockRecruits` and `recruits`, are defined in `events.rs` in the same directory.

Assign local hold orders only to the intended group. A guard duty must not
silently become an army-wide defense or survival objective. Inspect current
`enemyGroups` examples and the `EnemyGroup` model in `scenario.rs` before editing
activation or hold conditions.

## Stable identities

Use campaign-scoped character IDs for placements, dialogue and selectors. Display
names and promoted unit types are not identity. Leadership is an instance's
`role: "leader"`, independent of its unit type. Preserve character identity,
progression and equipment across promotion and chapter transitions.

Use map-scoped tile IDs for placements and references. Coordinates describe
geometry: moving or changing a tile retains its ID. A placement references
`tile`, not copied `col`/`row` coordinates. Keep character and tile references
consistent across scripts, roster, objectives and companion arrivals; validate
through `npm run content:check`.
