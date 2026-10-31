<script lang="ts">
  import { onDestroy, tick, untrack } from "svelte";
  import type { TileView } from "../game/view";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { MAPS } from "../game/maps";
  import { battlefieldTiles, type BattlefieldTileView } from "./battlefield-view";
  import { battlefieldSpriteSize, mapDimensions } from "./geometry";
  import {
    TerrainMapCache,
    TerrainViewCache,
    terrainMapKey,
    terrainMapPreview,
    type TerrainMapImage,
  } from "../presentation/terrain-map";
  import { daylight, lightingKey, lightingMatrix } from "../presentation/battlefield-lighting";
  import { paintFog } from "../presentation/battlefield-fog";
  import terrainSprites from "virtual:terrain-prebuilt";
  import type { Artwork } from "../presentation/art";
  import { TerrainTilePrefetcher, terrainInspectionUrls } from "../presentation/terrain-prefetch";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
  }
  let { view, actions }: Props = $props();
  let map: SVGSVGElement | undefined;
  let viewport: HTMLDivElement | undefined;
  const terrainCache = new TerrainMapCache();
  const terrainViewCache = new TerrainViewCache();
  const terrainPrefetch = new TerrainTilePrefetcher();
  let loadedTerrain = $state<string>();
  let bakedTerrain = $state.raw<
    { key: string; artwork: Artwork; image: Readonly<TerrainMapImage> } | undefined
  >();
  let failedSprite = $state<string | null>(null);
  let hoveredTile = $state<string | null>(null);

  const game = $derived(view.game);
  const selected = $derived(view.selected);
  const artwork = $derived(view.artwork);
  const recruitment = $derived(view.recruitment);
  // Track the published turn even when the enemy turn hides the destination.
  const isPlayerTurn = $derived(game.turn === "player" && !game.winner);
  const recruitTileId = $derived(isPlayerTurn && recruitment ? view.inspectedTile : null);
  const terrainScene = $derived(terrainViewCache.prepare(game, artwork));
  const terrainResources: Artwork = $derived({
    frames: Object.fromEntries(
      [...terrainScene.tiles.values()].flatMap((tile) =>
        tile.terrainFrame ? [[tile.frameKey, tile.terrainFrame]] : [],
      ),
    ),
  });
  const tiles = $derived(
    battlefieldTiles(game, selected, artwork, recruitTileId, terrainScene.tiles),
  );
  const occupiedTiles = $derived(tiles.filter((tile) => tile.occupant));
  const size = $derived(mapDimensions(game));
  const mapWidth = $derived(size.width);
  const mapHeight = $derived(size.height);
  const viewBox = $derived(`0 0 ${mapWidth} ${mapHeight}`);
  const lighting = $derived(game.presentation.lighting);
  const fogTiles = $derived(game.presentation.fog);
  const fogSet = $derived(new Set(fogTiles));
  const terrainKey = $derived(
    JSON.stringify([...JSON.parse(terrainScene.key).slice(0, 4), lightingKey(lighting)]),
  );
  const authoredTerrainKey = $derived.by(() => {
    const map = MAPS[game.mapId];
    return (
      map &&
      terrainMapKey(
        {
          mapId: map.id,
          cols: map.cols,
          rows: map.rows,
          tiles: map.terrain as readonly TileView[],
        },
        lighting,
      )
    );
  });
  onDestroy(() => terrainCache.dispose());
  const backdrop = $derived(
    bakedTerrain?.key === terrainKey && bakedTerrain.artwork === terrainResources
      ? bakedTerrain.image
      : undefined,
  );
  const prebuilt = $derived(
    failedSprite === terrainKey
      ? undefined
      : terrainMapPreview(terrainKey, terrainSprites, authoredTerrainKey),
  );
  const hasBackdrop = $derived(!!(backdrop || prebuilt));
  const overlayTiles = $derived(
    hasBackdrop ? tiles.filter((tile) => !backdrop && prebuilt?.changed.has(tile.id)) : tiles,
  );
  const hovered = $derived(tiles.find((tile) => tile.id === hoveredTile));
  const inspectionTiles = $derived(
    [game.unit(selected)?.tile, game.unit(view.target)?.tile, view.inspectedTile]
      .filter(Boolean)
      .join("|"),
  );

  $effect(() => {
    const scroller = viewport;
    const board = map;
    const ids = [...new Set(inspectionTiles.split("|").filter(Boolean))];
    if (!scroller || !board || !ids.length) return;
    let current = true;
    const reveal = () => {
      if (!current) return;
      const bounds = ids.flatMap((id) => {
        const nodes = board.querySelectorAll<SVGGElement>(
          `[data-tile="${id}"], [data-unit-tile="${id}"]`,
        );
        return [...nodes].map((node) => node.getBoundingClientRect());
      });
      if (!bounds.length) return;
      const area = scroller.getBoundingClientRect();
      const left = Math.min(...bounds.map((rect) => rect.left));
      const right = Math.max(...bounds.map((rect) => rect.right));
      const top = Math.min(...bounds.map((rect) => rect.top));
      const bottom = Math.max(...bounds.map((rect) => rect.bottom));
      // Keep both combatants visible when they fit; otherwise prioritize the inspected tile.
      const last = bounds[bounds.length - 1]!;
      const x1 = right - left + 24 <= scroller.clientWidth ? left : last.left;
      const x2 = right - left + 24 <= scroller.clientWidth ? right : last.right;
      const y1 = bottom - top + 24 <= scroller.clientHeight ? top : last.top;
      const y2 = bottom - top + 24 <= scroller.clientHeight ? bottom : last.bottom;
      scroller.scrollLeft +=
        Math.min(0, x1 - area.left - 12) || Math.max(0, x2 - area.left - scroller.clientWidth + 12);
      scroller.scrollTop +=
        Math.min(0, y1 - area.top - 12) || Math.max(0, y2 - area.top - scroller.clientHeight + 12);
    };
    // Panel changes settle first. Later viewport resizes must preserve the same inspection.
    void tick().then(reveal);
    const resize = new ResizeObserver(reveal);
    resize.observe(scroller);
    return () => {
      current = false;
      resize.disconnect();
    };
  });

  $effect(() => {
    if (loadedTerrain !== terrainKey || !prebuilt || prebuilt.changed.size) return;
    return terrainPrefetch.start(
      terrainInspectionUrls(
        artwork,
        untrack(() => game.tiles),
      ),
    );
  });

  $effect(() => {
    const key = terrainKey;
    const resources = terrainResources;
    if ((prebuilt && !prebuilt.changed.size) || tiles.some((tile) => !tile.terrainFrame)) {
      terrainCache.dispose();
      return;
    }
    const plan = {
      width: mapWidth,
      height: mapHeight,
      pixelScale: 64 / battlefieldSpriteSize,
      tiles: untrack(() => tiles),
      lighting,
    };
    let current = true;
    void terrainCache.prepare(key, resources, plan).then((image) => {
      if (current && image) bakedTerrain = { key, artwork: resources, image };
    });
    return () => {
      current = false;
    };
  });

  function mountFog(node: HTMLElement, ids: readonly string[]) {
    const canvas = document.createElement("canvas");
    canvas.style.width = "100%";
    canvas.style.height = "100%";
    node.replaceChildren(canvas);
    const draw = (ids: readonly string[]) => {
      const scale = 64 / battlefieldSpriteSize;
      canvas.width = Math.ceil(mapWidth * scale);
      canvas.height = Math.ceil(mapHeight * scale);
      const context = canvas.getContext("2d")!;
      context.scale(scale, scale);
      paintFog(
        context,
        tiles.filter((tile) => ids.includes(tile.id)),
      );
    };
    draw(ids);
    return {
      update: draw,
      destroy() {
        node.replaceChildren();
      },
    };
  }

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
  <div class="map-scroll" bind:this={viewport}>
    <svg
      bind:this={map}
      id="map"
      {viewBox}
      style={`--map-scale: ${game.cols / 11};`}
      role="group"
      aria-label={`${MAPS[game.mapId].name} battlefield`}
    >
      <defs>
        {#if lighting.time === "night"}
          {#each tiles as tile (tile.id)}
            <filter id={`night-tile-${tile.col}-${tile.row}`} color-interpolation-filters="sRGB">
              <feColorMatrix
                type="matrix"
                values={lightingMatrix(tile.x, tile.y, lighting).join(" ")}
              />
            </filter>
          {/each}
        {/if}
        {#each overlayTiles as tile (tile.id)}
          <clipPath id={tile.clipId}><polygon points={tile.points} /></clipPath>
        {/each}
        {#if hasBackdrop && hovered && !fogSet.has(hovered.id)}
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
        {:else if prebuilt}
          {#key terrainKey}
            {@const spriteKey = terrainKey}
            <g class="pixel-art" pointer-events="none" aria-hidden="true">
              <image
                class="map-terrain"
                href={prebuilt.sprite.url}
                x="0"
                y="0"
                width={prebuilt.sprite.displayWidth}
                height={prebuilt.sprite.displayHeight}
                preserveAspectRatio="none"
                pointer-events="none"
                aria-hidden="true"
                onload={() => {
                  if (terrainKey === spriteKey) loadedTerrain = spriteKey;
                }}
                onerror={() => {
                  failedSprite = spriteKey;
                }}
              />
            </g>
          {/key}
        {/if}
        {#if hasBackdrop}
          {#if hovered?.terrainFrame && lighting.time === "day" && !fogSet.has(hovered.id)}
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
            aria-label={fogSet.has(tile.id) ? "Fog-covered terrain" : tile.label}
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
            {#if hasBackdrop && (backdrop || !prebuilt?.changed.has(tile.id))}
              <polygon points={tile.points} fill="transparent" pointer-events="all" />
            {:else}
              <g
                filter={lighting.time === "night"
                  ? `url(#night-tile-${tile.col}-${tile.row})`
                  : undefined}
              >
                <polygon class="tile-ground" points={tile.points} fill={tile.fill} />
                <g clip-path={`url(#${tile.clipId})`}>
                  {@render terrainSprite(tile)}
                </g>
              </g>
            {/if}
            {#if tile.classes.includes("objective") || tile.classes.includes("recruitable")}
              <polygon class="tile-highlight-outline" points={tile.points} fill="none" />
            {/if}
            <polygon class="tile-outline" points={tile.points} fill="none" />
          </g>
        {/each}
      </g>

      {#if fogTiles.length}
        <g class="fog-layer" pointer-events="none" aria-hidden="true">
          <foreignObject x="0" y="0" width={mapWidth} height={mapHeight}>
            <div style="width:100%;height:100%" use:mountFog={fogTiles}></div>
          </foreignObject>
        </g>
      {/if}

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
                filter={lighting.time === "night"
                  ? `url(#night-tile-${tile.col}-${tile.row})`
                  : undefined}
                class="art-sprite pixel-art unit-art"
                class:enemy-unit={occupant.faction.relation === "hostile"}
                class:spent-unit={occupant.unit.attacked}
                data-faction={occupant.faction.id}
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
