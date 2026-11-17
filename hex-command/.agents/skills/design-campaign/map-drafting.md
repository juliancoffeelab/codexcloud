# Draft map previews

Run the bundled renderer from the project root:

```sh
node scripts/render-map.mjs \
  --watch \
  --map tmp/draft/map.json \
  --overlay tmp/draft/overlay.json \
  --output tmp/draft/preview.png
```

Watch mode keeps Wasm, decoded artwork and unchanged terrain layers in memory.
Saving the map, overlay, candidate manifest or used image automatically updates
the PNG. Input errors leave the last successful PNG available; the next edit
retries. Source or built-Wasm changes rebuild/restart the client. Stop with Ctrl+C.
Omit `--watch` for one PNG; Dune reuses unchanged OCaml compilation.

Add `--grid` for a red hex-grid overlay above the terrain and units. The grid
uses the same hex geometry as the map. Use `--grid-color "#ff4040"` to choose
another grid color; omit `--grid` for the normal preview.

For an offline artwork experiment, `--artwork candidate-frames.json` replaces
existing frame keys with complete local images. The JSON maps frame keys to
`url`, `width`, `height`, `anchorX`, `anchorY` and integer `z`; image paths are
relative to that JSON. Dimensions and anchors must match the registered frames.
This does not register artwork in the playable game. The shared painter draws complete images only; it never adds a ground polygon.

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

Pass an authored `map.json` with the `mapId` and `mapRevision` of a registered
map. The renderer substitutes this file into the draft catalog and loads it
through Rust with the registered roster. Keep the authored `tiles` array,
including explicit floors and terrain connections. Clipped edges and gaps stay
empty. The dimensions give the coordinate bounds.

An overlay can supply:

- `title` and `notes`: preview title and footer lines.
- `units`: complete native unit snapshots, replacing the roster units in the
  preview. Copy units from a checkpoint or omit this field to use the roster.
  Use an empty array for terrain alone. Each unit's `faction` is an ID declared
  by the roster. The renderer uses that faction's `palette` for its sprites.
- `observer`: faction whose visibility to render. Defaults to `player`.
- `memory`: explored hexes for the observer's vision group. `round` can
  override the prepared preview's round.
- `marks`: completed event markers, in the engine's `event:<id>` format, for
  resolving event-controlled lights. Ownership comes from the prepared preview state.
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
above all units. Draft bars depict full health, not simulated combat state.

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
