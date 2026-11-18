# Connected terrain

Read for directional sources, crossings, capped roads or explicit connection selection.
For shared walls, paving and towers, also read [castles](castles.md).

Separate **offline artwork authoring**, the **artwork build**, and **runtime
selection**. An authoring script can use Bézier curves for paths, polygonal
masonry, deterministic variation or prop placement to produce candidate native
matrices. Keep reusable authoring scripts with this skill so the process survives
the cloud workspace. Review and save the accepted matrices. The artwork build
reads those finished sources, validates them and composes layouts into PNGs;
it does not rerun the authoring algorithm. Runtime selects a finished frame and
knows nothing about the generator.

- Map data carries explicit `connections`. Direction bits are `e=1`, `se=2`,
  `sw=4`, `w=8`, `nw=16`, `ne=32`. Authoring tools may derive these from a layout;
  the runtime artwork selector uses the supplied connections.
- Follow the [terrain contract](../SKILL.md#terrain-contract) for explicit geometry
  and source filenames. Read supported masks from
  `frontend/engine/Terrain_plan.ml`, `rust/battle/src/data/state.rs`
  and the terrain's variant registry. Persist authoring results in map data.
- An explicitly supplied empty connection array means mask 0 where supported.
  Authoring tools may compute and persist connections before runtime.
  Capped roads retain their explicit
  `<exits>-end-<ends>.txt` variants and registered `dead-ends.json` pairs.
- Runtime selects finished crossing orientations; it does not rotate or draw
  a crossing. Preserve matching exit positions, widths and normal tangents.
- Ford source bases contain ground/water decoration. Stepping-stone props are
  composed through directional `layouts.json` entries. The build bakes those
  arrangements; runtime chooses one deterministically from map ID and tile ID.
- `floor` independently supplies G/g colors. Other material symbols retain their
  own colors. Foreground props and optional layout variation can sit on top of
  the connected source geometry; they do not have to generate the connection.

The reference implementation is `frontend/engine/Terrain_plan.ml` and
`scripts/build-pixel-art.py`. Their contract matters
whether the original authoring script exists or the frames were drawn by hand.
