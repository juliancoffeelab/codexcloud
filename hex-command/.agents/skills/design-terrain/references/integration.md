# Terrain integration

Read when adding a gameplay terrain ID or integrating approved artwork.
Work from the game root.

Follow existing terrain implementations before adding a new representation.
Bridge and ford demonstrate directional frames; plains, forest, hill and ford
demonstrate foreground layouts. A new terrain uses the existing tile `terrain`
identifier. Do not add a parallel artwork selector to tiles or extend the save
format just to select its sprite.

1. Register its stable ID, explicit typed `role`, name, movement cost, passability,
   description and any protection in `rust/data/game/terrain.json`. The role owns
   gameplay meaning: a bell-tower keep uses `keep` just like another keep. Native
   rules, AI, UI and inherited race/unit modifiers use roles, not artwork IDs or
   per-variant exceptions. New roles must define exhaustive behavior. Sprite
   selection and frozen historical migrations continue using concrete IDs. Add the ID to `Terrain` and
   `as_str()` in `rust/catalog/src/data/ids.rs`; regenerate the typed frontend
   bindings with `npm run wasm:build`.
   Check race/unit `terrainModifiers` if the terrain needs special treatment;
   do not inherit another terrain's behavior merely because its artwork is similar.
2. Add native `<mask>.txt` sources for connected terrain, or `tile.txt` only
   for terrain without connection variants, and material `palette.json` under
   `pixel/art/terrain/<id>/`. Follow [floor ownership](surfaces.md#floor-palettes).
   Keep foreground pieces
   in `pixel/art/terrain/props/<id>/<piece>/` and compose them in `layouts.json`.
   Ordinary props fit inside their hex. Complete terrain frames follow the
   [shared canvas contract](../SKILL.md#terrain-contract),
   enforced by `scripts/build-pixel-art.py`. The build bakes complete frames;
   runtime does not draw props outside them.
3. If directional frames are needed, follow bridge/ford: add source matrices,
   register their allowed stems in `scripts/build-pixel-art.py`, and select them
   in `frontend/engine/Terrain_plan.ml`. Match connection validation in
   `rust/catalog/src/data/state.rs`. Connection edges must have a stated meaning: for
   landing/ferry, the single `e` or `w` connection identifies the landward bank;
   `1.txt` faces east and `8.txt` faces west. Key directional layouts by the
   same source stems. Extend the existing generic directional checks as needed.
   Castle geometry uses explicit `walls`, not road-style
   `connections`. Register a shared inspection portrait in
   `frontend/engine/Art.ml`; it must not vary with map geometry.
4. Keep the terrain's inspection description in `rust/data/game/terrain.json`,
   consumed by `frontend/game/Presentation.ml`, and its preview color in
   `frontend/engine/Geometry.ml`. Existing movement, ownership, healing
   and recruitment behavior must be checked explicitly; a new ID does not
   automatically acquire the special behavior of villages or keeps.
5. Add it to the developer terrain showcase. Keep campaign/map experiments in
   ignored draft data until promotion is requested. Maps select the terrain with
   `terrain`, its ground with `floor`, and supported orientation with `connections`.
6. Run `npm run art:build`, `npm run art:check`, `npm run content:check`,
   `npm run check` and the shared `npm test` as appropriate to the changed code.
   Use the Rust toolchain skill for Rust/WASM checks. Inspect standalone hexes,
   all supported orientations and neighboring tiles through the normal terrain
   renderer. Preserve original evidence; do not substitute map-canvas overlays
   for an implementation or add save/reload checks for artwork-only changes.
