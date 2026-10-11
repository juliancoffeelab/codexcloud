<script lang="ts">
  import { preloadTerrainMap } from "../presentation/terrain-preload";
  import { BATTLE_MAPS } from "../game/maps";
  import {
    hexPoints,
    tileCenter,
    previewColors,
    hexColumnSpacing,
    hexRowSpacing,
  } from "./geometry";
  import type { MapDefinition, TerrainType } from "../game/types";
  import type { Immutable } from "../game/view";

  let { selected = $bindable("mirror") }: { selected?: string } = $props();

  $effect(() => {
    preloadTerrainMap(selected);
  });

  function previewTiles(layout: Immutable<MapDefinition>) {
    const terrainByTile = new Map<string, TerrainType>(
      layout.terrain.map((tile) => [`${tile.col},${tile.row}`, tile.terrain]),
    );
    return Array.from({ length: layout.rows * layout.cols }, (_, index) => {
      const col = index % layout.cols;
      const row = Math.floor(index / layout.cols);
      const { x, y } = tileCenter({ col, row });
      const terrain = terrainByTile.get(`${col},${row}`) || "plains";
      return { points: hexPoints(x, y), fill: previewColors[terrain] };
    });
  }

  const maps = BATTLE_MAPS.map((layout) => ({
    id: layout.id,
    name: layout.name,
    description: layout.description,
    viewBox: `0 0 ${85 + hexColumnSpacing * (layout.cols - 0.5)} ${84 + hexRowSpacing * (layout.rows - 1)}`,
    tiles: previewTiles(layout),
  }));
</script>

<fieldset id="map-picker">
  <legend>Battlefield</legend>
  {#each maps as map}
    <label class="map-choice">
      <input type="radio" name="battle-map" value={map.id} bind:group={selected} />
      <span>
        <svg class="map-preview" viewBox={map.viewBox} aria-hidden="true">
          {#each map.tiles as tile}
            <polygon points={tile.points} fill={tile.fill} stroke="#263322" stroke-width="2" />
          {/each}
        </svg>
        <strong>{map.name}</strong>
        <span class="map-description">{map.description}</span>
      </span>
    </label>
  {/each}
</fieldset>
