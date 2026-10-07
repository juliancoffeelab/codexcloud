# Static surface artwork

Redo plains, deep water, shallow water (`ford`) and bridge for Normal and Low-poly.
These are fixed tiles; rendering does not choose shoreline shapes from neighbors.

All four surfaces use the game's overhead view and exact pointy-top hex clipping.
Source textures are full square canvases so the renderer's regular hex reaches
opaque surface color at every edge. Broad background planes have no edge vignette,
border, elevated tile slab or hex-shaped shadow. Detail stays away from tile edges.

- Plains: quiet olive green, short fine grass, a few small earth/vegetation patches.
  Grass clumps are small beside a human foot; the tile is open, level ground.
- Deep water: subdued dark blue, small low-contrast ripples, a few short restrained
  highlights. No large white wave ribbons or central swirl.
- Shallow water: turquoise, a visible pale sandy bottom, a few submerged pebbles
  and delicate ripples. Clearly distinct from deep water at battlefield scale.
- Bridge: one fixed left-to-right timber span over the same dark blue water as
  the deep-water tile. Visible deck planks, low wooden rails and supporting posts.
  Deck ends reach the left/right boundaries. Water remains visible on both sides.

Normal uses softly painted surfaces and materials. Low-poly uses low-contrast
polygon planes and angular wood; broad surface color is more prominent than the
facets. Both styles keep units, ownership and objective markers readable.

Inspect repeated plains, repeated water, a shallows transition and a bridge with
units at the actual 70-pixel terrain / 56-pixel unit sizes. Generated source sheets
use a 2×2 layout: plains/deep water on top; shallows/bridge below. Keep these
references and save each square as its own source texture before atlas packing.
