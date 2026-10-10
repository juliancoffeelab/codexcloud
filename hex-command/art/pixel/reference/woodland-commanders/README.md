# Approved woodland commander artwork

The user approved the existing **Wildkeeper L1 → Grovewarden L2 → Grove oracle L3**
artwork as shown in [the enemy lineup](enemy-lineup.png). No pixels or palettes
were changed. `approved-sources.json` records their original byte hashes.

[The lineup](../../lineups/woodland.json) and artwork catalog identify all three
as commanders. The original L1/L2 game unit levels and leader flags remain in
place. Grove oracle is an artwork tier; gameplay registration and stats are
outside this approval. Thorncaller branching and fox-tier differentiation remain
separate TODOs.

The expansion source resolver preserves these chosen names, levels and role.
The historical L3 reuse recipe now uses that resolver for catalog metadata.
Do not rerun it to refresh previews: it writes matrices.

Refresh native faction PNGs with `python3 scripts/render-pixel.py --previews`, then
render this read-only board from the project root:

```sh
node art/pixel/reference/human-anatomies/preview.cjs \
  art/pixel/reference/woodland-commanders/enemy-lineup.json \
  art/pixel/reference/woodland-commanders/enemy-lineup.png 3 5
```
