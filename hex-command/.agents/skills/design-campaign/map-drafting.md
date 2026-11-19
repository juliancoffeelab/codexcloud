# Draft map previews

Run the bundled renderer from the project root:

```sh
node scripts/render-map.mjs \
  --watch \
  --map tmp/draft/map.json \
  --roster tmp/draft/roster.json \
  --scenario tmp/draft/scenario.json \
  --overlay tmp/draft/overlay.json \
  --output tmp/draft/preview.png
```

Watch mode keeps Wasm, decoded artwork and unchanged terrain layers in memory.
Saving the map, roster, scenario, overlay, candidate manifest or used image updates
the PNG. Input errors leave the last successful PNG available; the next edit
retries. Source or built-Wasm changes rebuild/restart the client. Stop with Ctrl+C.
Omit `--watch` for one PNG; Dune reuses unchanged OCaml compilation.

Add `--grid` for a red hex-grid overlay above the terrain and units. The grid
uses the same hex geometry as the map. Use `--grid-color "#ff4040"` to choose
another grid color; omit `--grid` for the normal preview.

`--artwork candidate-frames.json` supplies complete local images. The JSON maps
frame keys to `url`, `width`, `height`, `anchorX`, `anchorY` and integer `z`; image
paths are relative to that JSON. Replacements preserve registered geometry. New
actor keys use native 64×64 images with anchors (32, 32). This does not register
artwork in the playable game. The shared painter draws complete images only.

Use `--draw-order reverse-within-z` or `shuffle-within-z --order-seed 17` to
diagnose order dependence while preserving authored depth. Restrict the test with
`--order-terrain castle-wall,castle-bell` to keep scenery order fixed.
The diagnostic changes the actual paint sequence, rather than just permuting
inputs before the normal sort. `reverse-all` and `shuffle-all` deliberately ignore
depth and are controls, not correctness requirements for a design with explicit z.

`--overlay` is optional. Keep scenario-specific map builders, input files and
outputs in ignored `tmp/`. The renderer reads source data as a devtool, never
registers content in the playable catalog, and does not execute encounters.
The OCaml/Melange client reuses the game's terrain selector, painter, fog and
lighting. The launcher asks Cargo and Dune to update their inputs, then bundles
the renderer; map-only watch updates reuse the running renderer. Conditions are resolved by the same Rust evaluator used
by live battles.

Supply three standalone documents using the current authored formats. They can
live entirely under `tmp/`; the map ID and revision need not exist in the playable
catalogue. The renderer creates a temporary in-memory manifest containing only
this scenario, while retaining the shared unit, terrain, item and ability
definitions. It neither edits nor registers files in `rust/data/`. Reusing an
existing map ID for an experiment does not replace that map on disk.

- `--map`: the authored map, including its ID/revision, name, description,
  dimensions, modifiers, lighting and tiles. `cols` and `rows` bound the layout;
  tiles may form clipped edges or holes. Missing cells stay absent rather than
  being filled with invented terrain. Use explicit floor and connection geometry
  where required by the selected artwork.
- `--roster`: the authored initial factions, vision groups and unit placements.
  Supply their mechanical fields even for a terrain-only preview; the tool does
  not fabricate a roster, sight rules or lighting.
- `--scenario`: the normal scenario document, including `map`, `roster`,
  `characters` and `tags`. Its relative map/roster references must resolve to the
  files passed by the corresponding CLI flags.

For current format examples, inspect the independent
[map](../../../rust/data/game/scenario/mirror-v1/map.json),
[roster](../../../rust/data/game/scenario/mirror-v1/roster.json) and
[scenario](../../../rust/data/game/scenario/mirror-v1/scenario.json) documents.
Author a draft rather than changing those examples. Unsupported or incomplete
mechanical data fails through the normal Rust setup path; no parallel draft
rules engine or historical input adapter is involved.

## Generate a campaign landscape

Use a current-format template of at least 24 columns by 20 rows. A rectangle of
plain tiles is a useful starting point. Choose the profile, a decimal seed
between `0` and `4294967295`, and optionally the route intent and facing.

```sh
node scripts/render-map.mjs \
  --map tmp/draft/map.json \
  --roster tmp/draft/roster.json \
  --scenario tmp/draft/scenario.json \
  --generate 315 --profile river-crossing --intent evacuation --facing west \
  --reveal-terrain --output tmp/draft/evacuation.png \
  --write-map tmp/draft/evacuation.json
```

| Profile          | Supported intents, default first       | Planned geography                                                                        |
| ---------------- | -------------------------------------- | ---------------------------------------------------------------------------------------- |
| `coast`          | `exploration`, `siege`                 | Open sea around islands, mainland port roads, and offshore landmarks or a citadel        |
| `island-city`    | `siege`, `transit`                     | Fortified river island with bridges to both banks. Siege adds an alternate approach      |
| `river-crossing` | `transit`, `evacuation`, `defense`     | Broad ford, single edgeward bridge, or a bridge and ford with a guard post               |
| `floodplain`     | `exploration`, `transit`, `evacuation` | River-fed wetlands and tributaries around planned crossings                              |
| `woodland`       | `defense`, `siege`, `exploration`      | Forest clearings, watch posts, and courtyard approaches. Exploration adds a shrine trail |
| `ridge-pass`     | `transit`, `evacuation`, `exploration` | Narrow pass through a continuous ridge. Exploration adds a second pass                   |
| `badlands`       | `transit`, `evacuation`, `exploration` | Staggered oases and trails across dry country, with an optional side route               |
| `caverns`        | `exploration`, `transit`, `siege`      | Connected chambers, a side chamber before the gate, and a deep objective or fortress     |
| `cave-frontier`  | `transit`, `exploration`               | Cave mouth opening onto dry country and a route to the far edge                          |
| `rift-caverns`   | `exploration`, `transit`               | Underground river with branching rifts and a required bridge                             |

Facing defaults to `east`. Facing `west` changes the direction of travel or the
side of the main terrain feature. Seed and dimensions vary landmark locations,
route geometry, and terrain. Each profile checks its structural constraints on
the template before the renderer attempts to export. Missing cells and authored
features remain in place. The planner preserves occupied tiles, named or owned
tiles, connection and wall metadata, environmental effects, distinct floor
palettes, and special terrain such as existing roads and villages. A template
that interrupts a required route, waterway, or courtyard can produce an error.
Try a larger clear footprint or a different seed or facing.

`--write-map` creates an editable authored map and requires one-shot mode. Use
a new output path. Update the standalone scenario's relative `map` reference
when reopening that file as the input to `--map`. Generated sites have
`draft-…` tile IDs that can serve as location references during authoring.
`--reveal-terrain` shows the whole landscape in the PNG, including with empty
armies. It leaves the map's perception rules and actor visibility intact.

The planner uses existing terrain, movement roles, and artwork. Badlands use
dry palettes with existing ground terrain. Underground rifts use water. Gates,
keys, forts, ports, and sites provide geography and landmark IDs for encounter
scripts. Locking gates, ferry transport, authored fog, lighting, objectives, and
unit placement are separate authoring steps.

## Local definitions and illustrations

For mechanics supported by the engine, add `--content content.json`. Its optional
registry arrays are `units`, `races`, `items`, `abilities` and
`environmentEffects`; entries are definition-file paths relative to that manifest:

```json
{ "units": ["units/elf-archer.json"], "abilities": ["abilities/woodcloak.json"] }
```

Each definition uses the current complete authored format. A local ID replaces
its shared definition only in the temporary catalogue; new IDs are appended.
Duplicate IDs in a local registry fail. Referenced files participate in watch
mode. Rust still parses mechanics: this does not extend closed race, family or
terrain vocabularies. A new unit ID's frame key follows the usual
`<palette>_<unit-id>` convention and can be supplied through `--artwork`.

For an illustration of unsupported creatures, use overlay `figures` instead of
inventing playable definitions. A figure requires a declared roster faction,
an authored tile, a local or existing frame key and explicit visibility:

```json
{
  "observer": "player",
  "figures": [
    { "col": 2, "row": 3, "faction": "player", "frame": "draft_elf", "visibility": "shown" },
    { "col": 8, "row": 3, "faction": "enemy", "frame": "draft_dragon", "visibility": "shown" }
  ]
}
```

Use `shown` for the designer's placement view, `hidden` to omit a figure, or
`terrain-visible` to show it only where Rust leaves terrain unfogged. Figures
contribute no sight, concealment or combat, and have no health bars. Unit
detection requires mechanical units; figure visibility is an explicit design
choice. Thus elves and a new dragon can be illustrated in authored fog with
local artwork and empty faction armies, without shared Catalogue edits.

## Overlays

An overlay can supply:

- `title` and `notes`: preview title and footer lines.
- `figures`: illustrations described above.
- `units`: complete current native unit snapshots, replacing the roster's
  prepared units. Use an empty array for terrain alone. A unit's `faction` must
  name a roster faction; its palette comes from that faction. These are static
  preview placements, not units spawned by scenario events.
- `observer`: the roster faction whose perception to show (`player` if omitted).
- `memory`: explored hex coordinates for that observer's vision group. These
  reveal fogged terrain only on tiles with the `remember-terrain` effect;
  other tiles still require current observation.
  `round` optionally overrides the roster's initial round.
- `marks`: completed event markers, in the engine's `event:<id>` format, for
  resolving event-controlled lights. Ownership comes from the supplied map tiles.
- `annotations`: objects with `col`, `row`, optional `text`, `color`, `dy` (label
  offset in battlefield coordinates), and `ring` (boolean).
- `arrows`: objects with `from` and `to` coordinates and optional `color`.

Use separate overlay files for different encounter stages rather than hardcoding
scenario stages into the renderer. Use separate map snapshots when terrain or
ownership changes. Arrows and labels explain the plan; they do not simulate
movement, AI, triggers or objectives. The preview is marked as a static draft.

Ground, terrain frames, clipping, unit artwork and night transforms come from the
existing presentation modules. Do not add terrain-looking overlay drawings or
custom artwork selectors to fake unsupported map geometry.

Unit placement follows `frontend/renderer/Phaser_battlefield.ml`: native sprites use the battlefield
scale, their canvas bottom is at tile center + 20, and units draw in tile order.
Team bases sit at center + 19; health bars draw at center + 27 in a separate layer
above all units. Draft bars are illustrative full-health markers, not simulated combat state.

## Draft scope and completion

Draft work may add artwork to the shared library and extend this renderer.
Artwork alone does not authorize gameplay/runtime changes. Keep broader tooling
improvements separate from the preview; missing gameplay support must not block
an illustration. Check artwork through its build, consistency checks and visual
inspection; artwork-only changes do not require a broader code audit.

Use separate preview states to examine routes, detour costs and encounter pressure.
A static illustration does not establish playability or balance. A draft completes
with the requested map-data preview and supported/unsupported mechanics stated.
Promote or publish only within the user's authorized scope.
