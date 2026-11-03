# Authored events

Lantern Road uses the general event schema in
`rust/catalog/src/data/events.rs`. Events are authored in each stage's
`scripts.json`. The other campaigns retain their existing scripts.

## Conditions and execution

`when` composes predicates with `all`, `any` and `not`. Supported predicates are
`stand`, `proximity`, `killed`, `left`, `turn`, `turnAtLeast`, `chapterStarted`,
`flag`, `hasUnits`, `noUnits`, `visibleUnits`, `owner` and consecutive `held`.
Character and tile references are stable authored IDs, independent of names and
coordinates. Unit selectors can narrow by character, role, faction, side, type,
a list of types or AI group.

Rust evaluates after completed movement, combat, recruitment, advancement and
equipment actions, at chapter entry and at turn boundaries. `turn` means the
start of that player turn; `turnAtLeast` is a persistent condition. `killed`
receives actual deaths from the completed action. `left` receives the moving
unit's departure. `stand` checks final occupancy unless `through: true` is
specified. Proximity includes the completed movement route by default; use
`through: false` to inspect only final occupancy.

Events and their actions execute in authored order. Higher optional `priority`
values execute first, preserving authored order among ties. Effects are visible
to later events immediately. Further scans allow a later event to enable an
earlier event. Within one evaluation an event can fire only once, or once per
matching death for a repeating death event. This bounds cascades.

By default a completed event is removed from the active event list. Setting
`remembers: false` leaves it eligible for subsequent evaluations; it does not
require leaving/re-entering a tile. Events that need previous outcomes read
explicit flags written with `setFlag`; an event ID alone is not a flag.

`held` counts completed intervals at player-turn starts. Its per-event,
per-predicate counter resets whenever no eligible unit occupies its tile. Another
eligible unit may continue the occupation if there is no intervening evaluation
with the tile unoccupied. The counter and last counted round survive reloads.

## Actions and durable state

Actions include `spawn`, `give`, `placeItem`, `changeAllegiance`, `deleteUnit`,
`setFlag`, `say`, `sound`, `log`, `createMark`, `removeMark`, `forEachUnit`,
`forEachMark`, `setTerrain`, `gold`, `unlockRecruits`, `victory` and `defeat`.
Rescue and sacrifice share `deleteUnit`: deletion emits no `killed` occurrence.
An explicit flag records what the deletion meant.

A spawn uses a faction and either an ordered placement area or an origin tile
with a radius. Occupied/impassable sites leave remaining members pending; the
request survives event completion and save/load. It is retried on later
evaluations without replaying dialogue or recreating already placed members.

Map marks are structured instances with a unique numeric ID, shared type and
tile ID. Several instances may occupy one tile. Optional mark lights use the
same light definition and condition rules as tile lights, including remembered
activation. Loop bindings expose `$unit`, `$unit.tile`, `$mark`, `$mark.tile`
and `$event.deathTile`; references are validated against their binding scopes.

## Dialogue and objectives

`say` contains ordered lines and `with` passages. A passage is selected when its
character is alive and present; optional `alternative` supplies the fallback.
A passage may contain several speakers. Selection happens when the action runs;
the selected dialogue queue and its cursor survive reloads. Names come from the
campaign character registry. Narrators may provide a speaker label. The
ferryman's letter is dialogue, not an item.

Objective metadata lives alongside its event. The intent is explicit: `goTo`,
`approach`, `escape`, `escort`, `capture`, `defeat` or `hold`. Hold includes its
selector, destination and duration. Campaigner translates intent rather than
reverse-engineering arbitrary boolean conditions. Owned capture sites are
removed from active targets. A hold can request eligible recruits. Visibility is
an explicit condition; optional objectives are separate from required ones.
Event completion determines objective completion, and authored victory/defeat
events determine chapter outcomes. AI intent does not execute gameplay.

## Items

Shared definitions live in `rust/data/game/items/*.json` and the manifest lists
them. Roster inventories and event rewards reference item IDs. Rust resolves
stats and supplies the catalogue to the frontend. Presentation-only assets are
keyed by item ID in `src/presentation/item-assets.ts` and preloaded for the
current map, including possible rewards and carried equipment.

## Bell

Bell uses three ordinary events, with no special ritual runtime or periodic
scheduler:

| Arrival | First toll | Second toll | Third toll |
| --- | --- | --- | --- |
| Normal | Turn 6 | Turn 12 | Turn 18 |
| Late | Chapter start | Turn 6 | Never |

Each toll processes its designated original ritual tile and all pending
`captive-death` mark instances. It deletes any occupying captive, spawns three
skeleton troops per consumed captive/death mark, spends the original terrain
mark and removes consumed death marks. A moved captive creates its death mark
where it dies. Two deaths on one tile create two marks and two groups. Sacrifice
does not create a death mark. Blocked placement remains pending after the mark
is consumed. The late branch never gets a third toll on turn 12.

The illustrative JSONs in [event-examples](event-examples/) preserve the agreed
examples; descriptive location IDs and prose there are illustrative. The
playable stage scripts contain the actual stable tile and faction IDs.

## Persistence

Save version 28 introduced storage for active event IDs, held counters, structured marks,
pending spawn members and unlocked recruitment pools in chapter event state.
Explicit flags store a history of `{state, stage, turn}` changes, where stage is
zero-based. The final entry supplies the current value; a missing history is
false. `setFlag: "name"` sets true; `setFlag: {"id": "name", "state": false}`
clears it. Setting the same value again adds no redundant change.

Retry trims changes from the current stage, restoring values from earlier
stages, and restarts chapter event state. Turn-zero entries represent incoming
facts with unavailable historical timestamps; they survive the retry. Advancing
to the next stage retains the full history. This also governs legacy story
choices, replacing the refuge-specific retry reset.

Save version 29 replaces the flat choices list with this history. Frozen stage
provenance preserves old values, but original write turns and prior values cannot
be recovered from old flat lists.
The isolated version-27 conversion uses frozen historical recipes and never
queries the current catalogue or runs game actions. Legacy campaigns keep their
existing progress representation.
