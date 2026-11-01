# Ground and foreground

Read when choosing a tile’s surfaces, foliage and source formats.

Distinguish the backdrop, foreground and foliage of a terrain tile. These are
visual roles, not a requirement for three separate files:

- **Background:** the ground surface: grass, soil, rock or water. Shared `G`/`g`
  colors come from `pixel/art/terrain/floors.json`. Floor changes should not
  recolor trees, structures or stones.
- **Foreground:** grass tufts, trees, roads, rocks, crossing stones, villages and
  structures. Use separate material symbols and reusable transparent 64×64
  pieces. These are composed over ground, with procedural variation suited to
  their complexity; not every object needs the same amount of randomization.

Within that artwork, distinguish the main foreground feature (buildings,
mountains, crossing stones) from foliage and small surface decoration (grass,
shrubs, wave marks). Sea tiles have the water backdrop and wave decoration, with
no main foreground object. Mountains have the rock backdrop and mountain
foreground, generally without foliage. Most other tiles have both a main feature
and surrounding foliage. Plains have ground decoration without a main structure.
Changing a backdrop palette does not supply missing foliage. A burned building
does not imply that the whole hex has burned ground or no vegetation.

Check the complete tile on an actual map through the shared map renderer:
backdrop joins with neighbors, foreground scale and placement, and foliage density
must all fit the surroundings. An isolated prop or matching ground color alone
is not enough. Use the same lighting as the intended map when judging the result.

The existing implementation is partly separated: forest, hill and plains bases
still include ground decoration; ford bases include water/ripples; props are
placed through `layouts.json`. Roads and villages use finished directional
frames, some with ground baked into their source. Respect these formats while
designing new pieces toward separate layers. Do not claim all foreground already
has a procedural generator.
