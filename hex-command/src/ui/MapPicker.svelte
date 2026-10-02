<script lang="ts">
  import { MAPS } from '../game/maps';
  import { hexPoints, tileCenter, previewColors } from './geometry';
  import type { MapDefinition, TerrainType } from '../game/types';
  let { selected = $bindable('mirror') }: { selected?: string } = $props();
  function tiles(layout: MapDefinition) {
    const features = new Map(layout.terrain.map(tile => [`${tile.col},${tile.row}`, tile.terrain]));
    return Array.from({ length: layout.rows * layout.cols }, (_, index) => {
      const col = index % layout.cols, row = Math.floor(index / layout.cols);
      return { ...tileCenter({ col, row }), terrain: (features.get(`${col},${row}`) || 'plains') as TerrainType };
    });
  }
</script>
<fieldset id="map-picker"><legend>Battlefield</legend>{#each Object.values(MAPS) as layout}<label class="map-choice"><input type="radio" name="battle-map" value={layout.id} bind:group={selected}/><span><svg class="map-preview" viewBox={`0 0 ${85 + 61 * (layout.cols - .5)} ${84 + 53 * (layout.rows - 1)}`} aria-hidden="true">{#each tiles(layout) as tile}<polygon points={hexPoints(tile.x, tile.y)} fill={previewColors[tile.terrain]} stroke="#263322" stroke-width="2"/>{/each}</svg><strong>{layout.name}</strong><span class="map-description">{layout.description}</span></span></label>{/each}</fieldset>
