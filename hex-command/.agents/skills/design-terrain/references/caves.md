# Caves and underground areas

Read when authoring cave passages, solid rock or underground ambience. Follow the
[terrain contract](../SKILL.md#terrain-contract) and
[connections](connections.md) for complete native frames and explicit openings.

## Passages and solid rock

`cave-passage` is walkable floor with rock rims on closed sides. Its explicit
`connections` open sides into neighboring floor; keep adjoining passage openings
reciprocal. Use `cave-wall` solid rock beyond closed sides so the artwork agrees
with movement. Passage rims select artwork; solid wall cells supply collision.

`cave-wall` has its own impassable gameplay role. Reusing a surface role can let
inherited flight or swimming modifiers make underground rock traversable. Check
native movement costs when adding units or changing terrain roles.

Passage frames use the castle canvas while solid walls use ordinary hex frames.
Read dimensions, anchors and depth from family metadata and the builder's
defaults for families without canvas metadata. Preview passages
beside solid rock and castle structures: raised masonry must remain above the
surrounding floors, and wall tops must join their faces across hex boundaries.
Verify reversed and shuffled draw order within each depth.

The offline [authoring script](../scripts/generate-cave.py) reads the explicit
passage registry and can also produce a solid wall candidate:

```sh
python3 .agents/skills/design-terrain/scripts/generate-cave.py \
  tmp/cave-candidate --catalog pixel/art/terrain/cave-passage/variants.json \
  --walls-output tmp/cave-wall-candidate
```

Review candidates through the shared map drafter with rooms, bends, junctions,
inward corners and troops. Installation follows [integration](integration.md).

## Underground ambience and exploration

Map lighting supplies explicit underground cells independently of terrain IDs:

```json
{
  "lighting": {
    "time": "day",
    "underground": [
      { "col": 4, "row": 5 },
      { "col": 5, "row": 5 }
    ]
  }
}
```

The map time controls outdoor ambience; underground cells use cool darkness and
local tile lights. Include chamber floors, structures and covered wall cells so
raised artwork shares the intended ambience. Check terrain and moving actors at
entrances, and confirm that local cave lights leave outdoor daylight intact.

Author fog separately on cells that should remain unexplored. Cover rock between
unknown chambers as well as their floors so the shroud conceals the layout.
Remembered proximity reveal retains explored cells and does not block vision at
walls. Keep outdoor fog rules consistent with the intended surface visibility.

A cave layout is ready when its openings match actual traversability, troops can
reach the intended rooms, layered artwork joins correctly, and fog and lighting
agree with the authored exploration state.
