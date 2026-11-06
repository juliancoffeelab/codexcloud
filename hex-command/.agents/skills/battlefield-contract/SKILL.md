---
name: battlefield-contract
description: Use when changing unit artwork, terrain authoring, battlefield rendering or runtime game-data ownership.
---

# Battlefield contract

- **Permanent character canvas rule: every character/unit sprite and portrait is
  exactly 64×64 native pixels. This rule must never be changed; do not reintroduce
  alternate sizes, automatic cropping or resizing.** New generation, extraction,
  cleaning and generated frames must use 64×64. Most characters should be below
  40 painted pixels tall; cavalry and giants may go slightly above this. Compare
  mounted artwork with existing mounted units before setting scale. Distinguish
  body/mount height from raised weapons or staffs. Terrain uses a 64×64 logical hex at the same native pixel scale. Explicit
  terrain overflow may add transparent canvas padding with an authored anchor
  and drawing depth; it must never resize the hex or artwork.
  Terrain, troops and commanders share one battlefield pixel scale; do not add
  per-unit scaling to emphasize leaders.
- **Native row 63 is the ground line. Grounded units must stand on it with no
  transparent rows beneath their ground-contacting body.** Preserve every painted
  pixel when aligning existing units: translate within the canvas, never resize
  or crop. Intentionally suspended animal poses must declare `"flying": true` in
  `palette.json`; this artwork-only flag covers hovering and swimming poses.
  Missing or false means grounded.

## Terrain canvas

Native canvas dimensions are a finite, enforced contract, not a convenience
setting in artwork metadata. Ordinary terrain is exactly 64×64. Only
`castle-wall` and `castle-bell` use the approved 80×96 canvas,
anchored at (40,48), without resizing the 64×64 logical hex or its pixels.
Do not enlarge canvases, move anchors, or add another size/family to make artwork
fit. A different profile requires an explicit user request and a coordinated
change to the validator, authoring tools and documented contract. Drawing depth
may be any authored integer; it does not authorize larger artwork.

## Permanent prohibition on map post-processing

Map post-processing is absolutely forbidden except for weather and atmospheric effects such as fog and lighting. Under no circumstances may this rule be changed, weakened, bypassed or given an exception. Runtime must select complete, prebuilt terrain frames from each tile’s explicit data. It must never assemble structures across the map, inspect neighboring geometry to inject artwork, distribute fragments, or paint terrain features onto other tiles. All terrain geometry and composition must be finished offline and baked into the selected tile frames.

## Connected terrain

Connected terrain must declare explicit geometry: castles use `walls`;
other connected terrain uses `connections`. Use numbered
`<mask>.txt` sources for every supported combination. Implicit orientation
defaults, special filename aliases such as `tile.txt` for one connected mask,
and unsuffixed connected frames are explicitly forbidden. Missing or invalid
geometry must fail; do not silently substitute, infer or backfill it in
runtime rendering or validation. This applies to all connected terrain,
including roads, crossings, settlements and castles. Unconnected terrain and
individual prop sources may use `tile.txt`.

## Runtime data ownership

Runtime code must obtain current authored game data, including scenarios and their
settings, through Rust. Do not bypass Rust by importing JSON or injecting game
data through generated frontend modules. Pure rendering details such as terrain
colors and shading may live in the frontend. Devtool scripts may read source data
directly.

For runtime renderer changes or GPU performance analysis, read
[battlefield rendering](references/rendering.md).
