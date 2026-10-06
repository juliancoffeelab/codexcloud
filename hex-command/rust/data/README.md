# Authored catalog

These JSON files are the native game's source of rules and scenario content. They
are edited directly; TypeScript exports and generated rules are not inputs.

`manifest.json` declares schema version 1 and all document paths. Array order is
intentional: terrain and unit arrays determine runtime indices, campaign `stages`
arrays determine mission order, and simulation arrays determine schedule order.
Paths are relative to this directory and cannot contain traversal components.

- `game/terrain.json` defines terrain IDs, labels, movement, passability and
  environmental damage modifiers.
- `game/races.json` defines inherited resistance and terrain modifiers.
- `game/units.json` defines unit deviations from their race, weapons and
  progression. Damage and range come from weapons. Movement costs and
  environmental modifiers are resolved once when Rust loads the catalog. Never
  author `costs` or `environment` arrays here.
- `game/scenario/<id>-v<revision>.json` contains standalone scenarios and retained
  legacy map revisions with geometry and starting placements.
- `game/campaigns/<id>/campaign.json` contains campaign metadata, ending dialogue
  and ordered stage document paths. Each stage embeds initial map geometry and
  placements under `state`, plus mission rules, story, goals, events, recruitment
  and reinforcements. These are authored starting definitions rather than frozen
  runtime snapshots. Its
  `map.units` retains starting-unit definitions used for campaign setup.
- `simulation/rosters.json`, `arena.json` and `campaign.json` contain roster and
  simulation defaults. They do not run simulations during loading.

Initial placements declare `type`, `side`, `col` and `row`, with optional story
`name` and `role`. A new troop's HP comes from its unit rule. Author `hp` only for
an intentionally wounded troop; it must be positive and at most the unit's max
HP. IDs follow placement order (`u1`, `u2`, ...). Rust creates fresh action flags,
zero experience, round 1, player turn, next ID and the initial log message. Legacy
full snapshots remain accepted for frozen reference catalogs; their explicit
fields preserve their recorded initial values. The story roster in `map.units`
and the base map placements in `state.units` remain separate definitions.

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

Invalid fields, identifiers, references, placements and duplicate IDs produce a
`ContentError` with its source document path. Unsupported manifest versions are
rejected.

`Rules.content_fingerprint` is the SHA-256 digest of the exact loaded JSON bundle.
Documents are hashed in lexicographic relative-path order as path bytes, one NUL
byte, then the complete document bytes. Formatting changes alter this digest. It
is separate from the engine build fingerprint; metadata and simulation defaults
are retained on the loaded catalog and shared by each match.
