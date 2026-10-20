<script lang="ts">
  import type { MapDefinition, TerrainType } from "../game/types";
  import type { Immutable } from "../game/immutable";
  import {
    hexPoints,
    tileCenter,
    previewColors,
    hexColumnSpacing,
    hexRowSpacing,
  } from "./geometry";
  let { map }: { map: Immutable<MapDefinition> } = $props();
  const tiles = $derived.by(() => {
    const terrain = new Map(map.terrain.map((tile) => [`${tile.col},${tile.row}`, tile.terrain]));
    return Array.from({ length: map.rows * map.cols }, (_, i) => {
      const col = i % map.cols,
        row = Math.floor(i / map.cols);
      const { x, y } = tileCenter({ col, row });
      return {
        points: hexPoints(x, y),
        fill: previewColors[(terrain.get(`${col},${row}`) || "plains") as TerrainType],
      };
    });
  });
</script>

<svg
  class="setup-map-preview"
  viewBox={`0 0 ${85 + hexColumnSpacing * (map.cols - 0.5)} ${84 + hexRowSpacing * (map.rows - 1)}`}
  aria-hidden="true"
>
  {#each tiles as tile}<polygon
      points={tile.points}
      fill={tile.fill}
      stroke="#263322"
      stroke-width="2"
    />{/each}
</svg>
