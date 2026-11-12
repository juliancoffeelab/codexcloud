# Install or register terrain

Read when installing artwork or adding a terrain ID. Existing artwork revisions
reuse their terrain's registration; apply the new-ID and gameplay branches only
when those contracts change. Follow the [terrain contract](../SKILL.md#terrain-contract)
for geometry fields and native frame profiles.

## New IDs and gameplay changes

For a new terrain, register its stable ID, typed role, name, movement cost,
passability, description and protection in `rust/data/game/terrain.json`. Add the
ID to `Terrain` and `as_str()` in `rust/catalog/src/data/ids.rs`.

The role owns gameplay meaning. Check inherited race/unit `terrainModifiers`
before reusing a role; visual similarity does not establish compatible behavior.
A new role needs exhaustive native behavior. Verify movement, ownership, healing
and recruitment for the requested role or gameplay change, and update inspection
descriptions when that behavior changes. Regenerate typed frontend bindings
with `node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh dev` after Rust API/type changes.

## Artwork and selection

Install reviewed matrices and material palettes under `pixel/art/terrain/<id>/`.
Use [surfaces](surfaces.md) for floor ownership and foreground composition.
Existing terrain revisions retain their registered geometry and native scale.

For a new ID or changed connection set, register selection in
`frontend/engine/Terrain_plan.ml` and keep native acceptance in
`rust/catalog/src/data/state.rs` aligned with the artwork builder. Use
[connections](connections.md) for directional registries and supported stems.
Register a stable inspection portrait in `frontend/engine/Art.ml` and a preview
color in `frontend/engine/Geometry.ml` for new IDs. Artwork-only revisions reuse
those entries unless their presentation changes.

Add new terrains to the Developer showcase with valid authored geometry and
traversable approaches. Reuse existing samples for artwork revisions. Promote
campaigns or other maps only within the requested scope.

## Validate the installed result

Rebuild and check artwork with `dune build --root=. @artwork` and `python3 -m unittest discover -s scripts -p 'test_pixel_*.py'`.
Inspect standalone frames and assembled neighbors through the shared renderer,
including every supported orientation, floor selection and density affected by
the change. Confirm that native-accepted selections all have finished images.

Run the applicable content, frontend and native checks following
[verification](../../verification/SKILL.md). An installation is ready when its sources pass the artwork
checks, native selection and validation agree, and assembled views show the
requested result at native scale. Include gameplay validation for changed rules.
