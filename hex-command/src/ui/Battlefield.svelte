<script lang="ts">
  import { untrack } from "svelte";
  import type { TileView } from "../game/view";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { MAPS } from "../game/maps";
  import { battlefieldTiles, type BattlefieldTileView } from "./battlefield-view";
  import { battlefieldSpriteSize, hexColumnSpacing, hexRowSpacing } from "./geometry";
  import {
    TerrainMapCache,
    terrainMapKey,
    type TerrainMapImage,
  } from "../presentation/terrain-map";
  import type { Artwork } from "../presentation/art";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
  }
  let { view, actions }: Props = $props();
  let map: SVGSVGElement | undefined;
  const terrainCache = new TerrainMapCache();
  let bakedTerrain = $state.raw<
    { key: string; artwork: Artwork; image: Readonly<TerrainMapImage> } | undefined
  >();
  let hoveredTile = $state<string | null>(null);

  const game = $derived(view.game);
  const selected = $derived(view.selected);
  const artwork = $derived(view.artwork);
  const recruitment = $derived(view.recruitment);
  // Track the published turn even when the enemy turn hides the destination.
  const isPlayerTurn = $derived(game.turn === "player" && !game.winner);
  const recruitTileId = $derived(isPlayerTurn && recruitment ? view.inspectedTile : null);
  const tiles = $derived(battlefieldTiles(game, selected, artwork, recruitTileId));
  const occupiedTiles = $derived(tiles.filter((tile) => tile.occupant));
  const mapWidth = $derived(758 + hexColumnSpacing * (game.cols - 11));
  const mapHeight = $derived(452 + hexRowSpacing * (game.rows - 7));
  const viewBox = $derived(`0 0 ${mapWidth} ${mapHeight}`);
  const terrainKey = $derived(terrainMapKey(game));
  const backdrop = $derived(
    bakedTerrain?.key === terrainKey && bakedTerrain.artwork === artwork
      ? bakedTerrain.image
      : undefined,
  );
  const hovered = $derived(tiles.find((tile) => tile.id === hoveredTile));

  $effect(() => {
    const key = terrainKey;
    const resources = artwork;
    if (!Object.keys(resources.frames).length) return;
    const plan = {
      width: mapWidth,
      height: mapHeight,
      pixelScale: 64 / battlefieldSpriteSize,
      tiles: untrack(() => tiles),
    };
    let current = true;
    void terrainCache.prepare(key, resources, plan).then((image) => {
      if (current && image) bakedTerrain = { key, artwork: resources, image };
    });
    return () => {
      current = false;
    };
  });

  function mountTerrain(node: HTMLElement, image: Readonly<TerrainMapImage>) {
    image.canvas.className = "map-terrain";
    node.replaceChildren(image.canvas);
    return {
      update(next: Readonly<TerrainMapImage>) {
        next.canvas.className = "map-terrain";
        node.replaceChildren(next.canvas);
      },
      destroy() {
        node.replaceChildren();
      },
    };
  }

  function handleTileKey(event: KeyboardEvent, tile: TileView): void {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      actions.choose(tile.id);
      return;
    }

    let { col, row } = tile;
    switch (event.key) {
      case "ArrowLeft":
        col--;
        break;
      case "ArrowRight":
        col++;
        break;
      case "ArrowUp":
        row--;
        break;
      case "ArrowDown":
        row++;
        break;
      default:
        return;
    }

    event.preventDefault();
    const nextTile = game.tile(`${col},${row}`);
    if (!nextTile) return;

    actions.focusTile(nextTile.id);
    // Keyed tile nodes already exist; focus does not require a render or a flush.
    map?.querySelector<SVGGElement>(`[data-tile="${nextTile.id}"]`)?.focus();
  }
</script>

{#snippet terrainSprite(tile: BattlefieldTileView)}
  {#if tile.terrainFrame}
    <svg
      class="art-sprite pixel-art terrain-art"
      x={tile.x - tile.spriteSize / 2}
      y={tile.y - tile.spriteSize / 2}
      width={tile.spriteSize}
      height={tile.spriteSize}
      viewBox={`0 0 ${tile.terrainFrame.width} ${tile.terrainFrame.height}`}
      aria-hidden="true"
      focusable="false"
    >
      <image
        href={tile.terrainFrame.url}
        x="0"
        y="0"
        width={tile.terrainFrame.width}
        height={tile.terrainFrame.height}
        preserveAspectRatio="none"
      />
    </svg>
  {/if}
{/snippet}

<section class="map-section" aria-label="Battlefield">
  <div class="map-scroll">
    <svg
      bind:this={map}
      id="map"
      {viewBox}
      style={`--map-scale: ${game.cols / 11};`}
      role="group"
      aria-label={`${MAPS[game.mapId].name} battlefield`}
    >
      <defs>
        {#if !backdrop}
          {#each tiles as tile (tile.id)}
            <clipPath id={tile.clipId}>
              <polygon points={tile.points} />
            </clipPath>
          {/each}
        {/if}
        {#if backdrop && hovered}
          <clipPath id="terrain-hover-clip"><polygon points={hovered.points} /></clipPath>
        {/if}
      </defs>

      <g class="terrain-layer">
        {#if backdrop}
          <g class="pixel-art" pointer-events="none" aria-hidden="true">
            <foreignObject
              x="0"
              y="0"
              width={backdrop.displayWidth}
              height={backdrop.displayHeight}
            >
              <div class="terrain-bitmap" use:mountTerrain={backdrop}></div>
            </foreignObject>
          </g>
          {#if hovered}
            <g
              class="terrain-hover"
              clip-path="url(#terrain-hover-clip)"
              pointer-events="none"
              aria-hidden="true"
            >
              <polygon class="tile-ground" points={hovered.points} fill={hovered.fill} />
              {@render terrainSprite(hovered)}
            </g>
          {/if}
        {/if}
        {#each tiles as tile (tile.id)}
          <g
            class={tile.classes}
            role="button"
            tabindex={tile.id === view.focusedTile ? 0 : -1}
            aria-label={tile.label}
            data-tile={tile.id}
            onclick={() => actions.choose(tile.id)}
            onkeydown={(event) => handleTileKey(event, tile)}
            onpointerenter={(event) => {
              if (event.pointerType === "mouse") hoveredTile = tile.id;
            }}
            onpointerleave={() => {
              if (hoveredTile === tile.id) hoveredTile = null;
            }}
          >
            {#if backdrop}
              <polygon points={tile.points} fill="transparent" pointer-events="all" />
            {:else}
              <polygon class="tile-ground" points={tile.points} fill={tile.fill} />
              <g clip-path={`url(#${tile.clipId})`}>
                {@render terrainSprite(tile)}
              </g>
            {/if}
            {#if tile.classes.includes("objective") || tile.classes.includes("recruitable")}
              <polygon class="tile-highlight-outline" points={tile.points} fill="none" />
            {/if}
            <polygon class="tile-outline" points={tile.points} fill="none" />
          </g>
        {/each}
      </g>

      <!-- Paint every terrain first; adjacent hexes must not hide units or bars. -->
      <g class="unit-layer" pointer-events="none" aria-hidden="true">
        {#each occupiedTiles as tile (tile.occupant!.unit.id)}
          {@const occupant = tile.occupant!}
          <g
            data-unit-tile={tile.id}
            data-unit={occupant.unit.id}
            transform={`translate(${tile.x} ${tile.y})`}
          >
            <ellipse
              class="team-base"
              cx="0"
              cy="19"
              rx="19"
              ry="7"
              fill={occupant.teamColor}
              stroke={occupant.teamColor}
            />
            {#if occupant.frame}
              <svg
                class="art-sprite pixel-art unit-art"
                class:enemy-unit={occupant.unit.side === "enemy"}
                class:spent-unit={occupant.unit.attacked}
                data-faction={occupant.unit.side}
                x={-occupant.spriteSize / 2}
                y={20 - occupant.spriteSize}
                width={occupant.spriteSize}
                height={occupant.spriteSize}
                viewBox={`0 0 ${occupant.frame.width} ${occupant.frame.height}`}
                aria-hidden="true"
                focusable="false"
              >
                <image
                  href={occupant.frame.url}
                  x="0"
                  y="0"
                  width={occupant.frame.width}
                  height={occupant.frame.height}
                  preserveAspectRatio="none"
                />
              </svg>
            {/if}
          </g>
        {/each}
      </g>

      <!-- Health and state indicators remain above every terrain and sprite. -->
      <g class="indicator-layer" pointer-events="none" aria-hidden="true">
        {#each occupiedTiles as tile (tile.occupant!.unit.id)}
          {@const occupant = tile.occupant!}
          <g data-health-unit={occupant.unit.id} transform={`translate(${tile.x} ${tile.y})`}>
            <rect class="hp-bg" x="-19" y="27" width="38" height="4" rx="2" />
            <rect class="hp" x="-19" y="27" width={occupant.healthWidth} height="4" rx="2" />
            {#if occupant.unit.attacked}
              <text class="acted" x="21" y="-17">✓</text>
            {/if}
            {#if occupant.unit.retreating}
              <text class="retreat-marker" x="-28" y="-14">↶</text>
            {/if}
          </g>
        {/each}
        {#each tiles as tile (tile.id)}
          <g data-indicator-tile={tile.id}>
            {#if tile.owner}
              <circle
                class="territory-badge"
                cx={tile.x + 23}
                cy={tile.y + 17}
                r="5"
                fill={tile.ownerColor}
              />
            {/if}
          </g>
        {/each}
      </g>
    </svg>
  </div>
</section>

<style>
  .terrain-bitmap {
    width: 100%;
    height: 100%;
  }
  .terrain-bitmap :global(canvas) {
    display: block;
    width: 100%;
    height: 100%;
    image-rendering: pixelated;
  }
</style>
