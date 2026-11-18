# Engine-owned scenarios and workloads

`manifest.json` is the single registry read by Rust for native and browser
engines. Scenario geometry and rosters live in `game/scenario/`; campaign stages
keep their maps, rosters, scripts and story together in `game/campaigns/`.
Frontend code receives resolved metadata through the engine API.

Every `scenario.json` and `campaign.json` declares a nonempty `tags` array, such
as `["human"]` or `["simulation", "concealment"]`. Tags are explicit, unique
lowercase identifiers; there is no default `human` tag or fixed category list.
The engine exposes complete metadata for rendering and loading all registered
content, including historical map revisions. The frontend filters new-game
selection lists to entries tagged `human` unless “Allow non-human maps” is
enabled. Human-only browsing shows neither tags nor a filter. When enabled,
the filter offers the authored tags and matches any selected tag; an empty
selection shows all permitted entries. This does not restrict loading or debug
launchers. Campaign stage metadata inherits its campaign's tags.
Standalone scenarios can declare `characters` when their roster contains named
actors. Map identifiers and revisions remain the engine's loading keys.

Workload manifests reference these registered definitions:

- `simulation/arena.json` defines named arena groups, their fixed map variants,
  capture territory, holding duration and default schedule. Each variant either
  selects match armies with an explicit total budget or retains the scenario's
  forces and economies. Positive match seeds cycle through variants in manifest order; seed zero
  selects the first variant. The seed still independently controls combat and AI decisions. Terrain is loaded from
  the scenario, with no runtime generation or fallback template.
- `simulation/campaign.json` defines default campaign selection, diplomacy modes
  and simulation-only objective routes. These settings do not change the
  human-playable campaign definitions.
- `simulation/rl.json` selects existing arena groups, roster pairs, objective
  modes, campaign routes, opponents and recruiters for mixed learning. Python
  obtains this validated definition from Rust and adds sampling/seeding budgets.

The four frozen cases per original arena group preserve the old generator's
seeds 1–4, including their budgets. Identical boards share one map file. Other
seeds select from those cases; they do not reproduce the old procedural layouts.
The event-free Villages and Sluice combat scenarios are registered alongside the
other maps rather than rebuilt by a separate training loader.

Catalog schema changes are independent of saved-battle formats. Immutable old
experiment catalogs remain paired with their recorded engine; new training runs
use the current catalog and a policy warm start when provenance changes.

## Complete authored data

The current schema version is declared by `schemaVersion` in
[`manifest.json`](manifest.json). It requires complete mechanical contributions,
including zeroes, empty lists and nullable values. Missing required fields and
unknown catalog references fail parsing. Semantic effect and stacking checks belong in the
catalog tests. Runtime code assumes these invariants and panics if they are
violated. Saves contain resolved typed effects;
there is no recursive JSON fragment expansion.

Terrain definitions own capture eligibility, shot blocking, base movement, damage
adjustments, income and named environmental effects. Race and unit
`terrainModifiers` add to the terrain base. `passable: null` contributes no change.

Roster files declare round, turn, RNG seed, opening log and complete placements.
A placement authors `hp: "full"` to inherit its unit's maximum, or an explicit
wounded HP number. IDs and action/experience status are explicit; faction
ownership is inherited from the containing army.

Campaigns declare `carryover` limits, deployment radius and veteran ranking weights.
Survival defeats are ordinary scripted events using `survives`, which selects a
company, allied eligibility, character/role and whether evacuated units count.
Their order is authored through event priorities. Strategic protection metadata
continues to guide Campaigner independently of event execution.

## Illumination and sight stacking

Every map authors `lighting.time` (`day` or `night`) and complete `modifiers`:
`sight`, `damageModifiers`, and `terrainModifiers`. Day supplies `lit` ambient
light; night supplies `unlit`. A tile's `environmentEffects` list can override its
ambient level, including authored `darkness`. Ordinary cells use an empty list.
Mechanical queries and rendered ambient shading use the same resolved levels.
Light emissions illuminate their reach; visual-only glows explicitly author
`emission: null`. Darkness by itself does not hide terrain or units.

Race, unit and item definitions each author a complete sight contribution:

```json
{
  "sight": {
    "range": 0,
    "unlimited": false,
    "sightModifiers": { "lit": 0, "unlit": 0, "darkness": 0 },
    "penetrates": []
  },
  "abilities": []
}
```

The vision group supplies the scenario's base sight. Range contributions and
lighting adjustments add across race, unit, map/weather, items and scoped effects;
unlimited grants and penetration capabilities union. A zero adds nothing; it does
not replace earlier contributions. Target lighting adjusts finite effective sight,
which is bounded to 0–65535. `setMapModifiers` replaces the map's contribution;
`setTimeOfDay` changes its ambient light. Both are ordinary authored event actions.

## Reusable typed abilities

`abilities/` contains unit and equipment capabilities. Each definition authors
`id`, `name`, `description`, and a complete typed `effects` array. Races, units
and items refer to definitions through `abilities: ["darkvision"]`; their sources
stack. There are no `includes`, section references, merge rules or override switches.

For example, Darkvision contributes +3 sight in unlit and dark places while
leaving the existing range and lit adjustment intact:

```json
{
  "scope": { "radius": 0, "includeSelf": true, "targets": "allies" },
  "stacking": { "group": "sight", "combine": "add" },
  "effect": {
    "kind": "sight",
    "range": 0,
    "unlimited": false,
    "sightModifiers": { "lit": 0, "unlit": 3, "darkness": 3 },
    "penetrates": ["darkness"]
  }
}
```

Effect kinds are `healing`, `thorns`, `sight`, `concealment`, `light` and
`stat-modifiers`. Every effect declares its scope and stacking group. Numeric
healing and thorns support `add` or `max`; sight and stat adjustments add;
capabilities union. Scope targets are `own-faction`, `allies`, `enemies` or
`everyone`. Recipients never re-emit an aura they receive.

Thornshield and thorn aura use the same typed reflection effect. Separate
personal and external groups preserve their existing combination: strongest
personal shield plus strongest friendly aura. Healing units and bottles use the
same healing group, preserving strongest-only unit support. Settlement healing
is a distinct environmental group and remains a separate turn-start phase.
Drain remains a weapon effect. Timing and XP awards remain engine rules.

## Environmental effects

`environment-effects/` is a separate catalog for placed ambience, terrain
concealment, remembering terrain, fixed light sources and scoped effects
on occupants. Map tiles reference it through `environmentEffects`; terrain types
reference scoped occupant effects through the same list. It is not a catalog of
unit abilities. Maps can author weather through their numeric `modifiers`.

Concealment requirements name terrains and illumination levels. Darkcloak uses
`illumination: ["darkness"]`; forest camouflage names its terrain. Sight,
concealment and illumination remain distinct physical queries.

Saves retain complete resolved facts. Perception effects have no expiration
field. The v50-to-v51 web migration converts old carried profiles and undo states;
older schemas remain frozen inside the save migration package.
