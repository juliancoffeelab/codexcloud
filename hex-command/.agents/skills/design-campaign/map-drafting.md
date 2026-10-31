# Draft map previews

Run the bundled renderer from the project root:

```sh
node --import tsx .agents/skills/design-campaign/scripts/render-map.ts \
  --map tmp/draft/map.json \
  --overlay tmp/draft/overlay.json \
  --output tmp/draft/preview.png
```

`--overlay` is optional. Keep scenario-specific map builders, input files and
outputs in ignored `tmp/`. The renderer reads source data as a devtool, never
registers content in the playable catalog, and does not execute encounters.
Use `npm run wasm:build` after Rust changes and `npm run art:build` if artwork
frames need rebuilding. Conditions are resolved by the same Rust evaluator used
by live battles.

The map input has `mapId`, positive `cols` and `rows`, and a complete `tiles` grid.
Tiles use the existing map schema, including explicit `floor` choices and terrain
`connections`. An optional `name` titles the preview. Optional `lighting` uses the
Rust-authored map format; optional `units` have `type`, `side`, `col` and `row`.
This renderer checks geometry and artwork availability; it does not replace Rust
content validation or establish whether a scenario is playable.

An overlay can supply:

- `title` and `notes`: preview title and footer lines.
- `units`: the complete set of unit placements for this preview, replacing map
  units. Use an empty array for terrain alone. An optional unit `faction` selects
  an available visual palette independently of allegiance (for example white
  passive artwork). These are illustrations, not spawned units in a running game.
- `memory`: previously remembered condition keys, as exported in a checkpoint’s
  `conditionMemory`. `round` optionally sets the current round (default 1).
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

Unit placement follows `Battlefield.svelte`: native sprites use the battlefield
scale, their canvas bottom is at tile center + 20, and units draw in tile order.
Team bases sit at center + 19; health bars draw at center + 27 in a separate layer
above all units. Draft bars depict full health, not simulated combat state.
