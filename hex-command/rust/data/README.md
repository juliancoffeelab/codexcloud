# Authored catalog

These JSON files are the native game's source of rules and scenario content. They
are edited directly; TypeScript exports and generated rules are not inputs.

`manifest.json` declares schema version 2 and all document paths. Array order is
intentional: terrain and unit arrays determine runtime indices, campaign `stages`
arrays determine mission order, and simulation arrays determine schedule order.
Paths are relative to this directory and cannot contain traversal components.

- `game/terrain.json` defines terrain IDs, labels, movement, passability and
  environmental damage modifiers.
- `game/races/<race-id>.json` defines one race's inherited resistance and terrain modifiers.
- `game/units/<unit-id>.json` defines one unit's deviations from its race, weapons and
  progression. Every unit explicitly declares `role`: `warrior`, `flank`, `ranger`
  or `healer`; AI does not infer it from speed, weapons or healing. Damage and range come from weapons. Movement costs and
  environmental modifiers are resolved once when Rust loads the catalog. Never
  author `costs` or `environment` arrays here.
- `game/scenario/<id>-v<revision>/scenario.json` references `map.json` (geometry)
  and `roster.json` (starting gold and placements).
- `game/campaigns/<id>/campaign.json` contains campaign metadata, ending dialogue
  and ordered stage document paths. Each `stages/<stage-id>/stage.json` references
  `map.json`, `roster.json`, `objectives.json`, `story.json` and `scripts.json`.
  Maps contain geometry and metadata; rosters contain placements, recruitment and
  budgets; objectives contain goals, required companions and defense settings;
  stories contain briefings, hints and dialogue; scripts contain triggered events,
  reinforcement waves and conditional story overrides. Components are assembled
  into typed mission data during loading.
- `simulation/rosters.json`, `arena.json` and `campaign.json` contain roster and
  simulation defaults. They do not run simulations during loading.

The unit/scenario/campaign paths in the root manifest are catalog-relative.
Stage paths are relative to their campaign document, and component paths are
relative to their scenario/stage descriptor. Traversal is rejected at every level.
Standalone scenarios currently have no separate authored objective definitions;
their existing commander-defeat rule remains in the core.

Timed `hold` goals require ownership of their tile for the authored `turns`,
after `startEvent` when present, with completion no earlier than `earliestRound`.
An optional `companion` names a player unit that must also occupy the tile.
Leaving the tile or losing ownership resets the counter. Without `companion`,
ownership alone is sufficient. `vigil` always requires its named companion.

Initial placements declare `type`, `side`, `col` and `row`, with optional story
`name` and `role`. A new troop's HP comes from its unit rule. Author `hp` only for
an intentionally wounded troop; it must be positive and at most the unit's max
HP. IDs follow placement order (`u1`, `u2`, ...). Rust creates fresh action flags,
zero experience, round 1, player turn, next ID and the initial log message.
Only the current document schema is accepted. A stage roster's `initial` contains
the base scenario's gold and placements, while its top-level `units`, `recruits`,
`enemyRecruits`, `gold` and `enemyGold` define campaign deployment and recruitment.
These preserve the distinct scenario and campaign starting definitions.

A terrain modifier's `cost` adds to the base terrain cost; minimum resolved cost
is 1. Unit `passable` overrides race `passable`, which overrides terrain
passability. Damage modifiers add terrain, race and unit values; category and
damage-type modifiers both apply. Unit `damageModifiers` are deviations, so race
resistances must not be repeated in a unit specification.

Native `Rules::load()` reads this directory using the crate's build-time location,
or `HEX_COMMAND_DATA_ROOT` if configured. `Rules::load_from(path)` selects an
explicit root. `Rules::from_json_bundle` uses the same typed loader without filesystem access,
including on WebAssembly. `Rules::embedded()` and its embedded JSON documents are
available only for tests or the `wasm` feature. Default native builds contain no
embedded JSON, so editing authored data takes effect at the next load without
recompiling Rust.

The WASM factory exposes typed catalog metadata through `catalog()`. Unit movement
and terrain damage tables are resolved by Rust, including race and unit deviations.
The browser formats those values; it does not import JSON or reimplement inheritance.
Historical layouts, numeric wave markers and old dialogue live exclusively under
`src/save/migration/legacy/`, outside the live catalog and Rust game data.

Invalid fields, identifiers, references, placements and duplicate IDs produce a
`ContentError` with its source document path. Unsupported manifest versions are
rejected.

`Rules.content_fingerprint` is the SHA-256 digest of the exact loaded JSON bundle.
Documents are hashed in lexicographic relative-path order as path bytes, one NUL
byte, then the complete document bytes. Formatting changes alter this digest. It
is separate from the engine build fingerprint; metadata and simulation defaults
are retained on the loaded catalog and shared by each match.
