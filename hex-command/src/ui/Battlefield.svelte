<script lang="ts">
  import { pixelSprites } from "../presentation/pixel-art";
  import { tick, untrack } from "svelte";
  import type { TileView } from "../game/view";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { TERRAIN } from "../game/catalog";
  import { MAPS } from "../game/maps";
  import { BattlefieldViewCache } from "./battlefield-view";
  import { battlefieldSpriteSize, mapDimensions } from "./geometry";
  import { TerrainViewCache, terrainAssetKey } from "../presentation/terrain-map";
  import { lightingMatrix } from "../presentation/battlefield-lighting";
  import {
    MapRenderer,
    acquireMapRenderer,
    type MapFrameRequest,
  } from "../presentation/map-renderer";
  import terrainSprites from "virtual:terrain-prebuilt";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
  }
  let { view, actions }: Props = $props();
  let map: SVGSVGElement | undefined;
  let viewport: HTMLDivElement | undefined;
  const terrainViewCache = new TerrainViewCache();
  const battlefieldViewCache = new BattlefieldViewCache();
  let renderError = $state<string>();
  let renderAttempt = $state(0);

  const game = $derived(view.game);
  const selected = $derived(view.selected);
  const focusedTile = $derived(view.focusedTile);
  const artwork = $derived(view.artwork);
  const recruitment = $derived(view.recruitment);
  // Track the published turn even when the enemy turn hides the destination.
  const isPlayerTurn = $derived(game.turn === "player" && !game.winner);
  const recruitTileId = $derived(isPlayerTurn && recruitment ? view.inspectedTile : null);
  const terrainScene = $derived(terrainViewCache.prepare(game, artwork));
  const tiles = $derived(
    battlefieldViewCache.prepare(game, selected, artwork, recruitTileId, terrainScene.tiles),
  );
  const occupiedTiles = $derived(tiles.filter((tile) => tile.occupant));
  const size = $derived(mapDimensions(game));
  const mapWidth = $derived(size.width);
  const mapHeight = $derived(size.height);
  const viewBox = $derived(`0 0 ${mapWidth} ${mapHeight}`);
  const lighting = $derived(game.presentation.lighting);
  const fogTiles = $derived(game.presentation.fog);
  const fogSet = $derived(new Set(fogTiles));
  const terrainKey = $derived(terrainScene.key);
  // Artwork metadata is part of scene identity, including anchors and drawing depth.
  const renderTerrainKey = $derived(
    JSON.stringify([terrainKey, [...terrainScene.tiles.values()].map((tile) => tile.terrainFrame)]),
  );
  const mapFrame: MapFrameRequest = $derived({
    terrainKey: renderTerrainKey,
    plan: {
      width: mapWidth,
      height: mapHeight,
      pixelScale: 64 / battlefieldSpriteSize,
      tiles: [...terrainScene.tiles].map(([id, tile]) => ({
        ...tile,
        role: TERRAIN[game.tile(id)!.terrain].role,
      })),
      lighting,
    },
    prebuilt: terrainSprites[terrainAssetKey(terrainKey)],
    fog: fogTiles,
  });
  // Map lighting is baked by the worker. These filters affect live unit/item overlays only.
  const nightFilters = $derived.by(() => {
    if (lighting.time !== "night") return [];
    const liveTiles = new Set([
      ...game.units.map((unit) => unit.tile),
      ...game.groundItems.map((item) => item.tile),
    ]);
    return [...terrainScene.tiles]
      .filter(([id]) => liveTiles.has(id))
      .map(([, tile]) => ({
        id: `night-tile-${tile.col}-${tile.row}`,
        values: lightingMatrix(tile.x, tile.y, lighting).join(" "),
      }));
  });
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

  function mountMap(node: HTMLDivElement, request: { frame: MapFrameRequest; attempt: number }) {
    let renderer: MapRenderer | undefined;
    let retired: MapRenderer[] = [];
    let attempt = -1;
    const update = (next: typeof request) => {
      try {
        if (attempt !== next.attempt) {
          if (renderer?.hasFrame) retired.push(renderer);
          else renderer?.destroy();
          renderer = undefined;
          attempt = next.attempt;
          const currentAttempt = attempt;
          renderer = acquireMapRenderer(
            node,
            (message) => {
              if (attempt === currentAttempt) renderError = message;
            },
            () => {
              if (attempt !== currentAttempt) return;
              for (const previous of retired) previous.destroy();
              retired = [];
            },
          );
        }
        renderer!.update(next.frame);
      } catch (error) {
        renderError = String(error);
      }
    };
    update(request);
    return {
      update,
      destroy() {
        renderer?.destroy();
        for (const previous of retired) previous.destroy();
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
    const nextTile = game.tiles.find((tile) => tile.col === col && tile.row === row);
    if (!nextTile) return;

    actions.focusTile(nextTile.id);
    // Keyed tile nodes already exist; focus does not require a render or a flush.
    map?.querySelector<SVGGElement>(`[data-tile="${nextTile.id}"]`)?.focus();
  }
</script>

<section class="map-section" aria-label="Battlefield">
  {#if renderError}
    <div role="alert" class="map-render-error">
      <p>{renderError}</p>
      <button
        onclick={() => {
          renderError = undefined;
          renderAttempt++;
        }}
      >
        Retry map rendering
      </button>
    </div>
  {/if}
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
        {#each nightFilters as filter (filter.id)}
          <filter id={filter.id} color-interpolation-filters="sRGB">
            <feColorMatrix type="matrix" values={filter.values} />
          </filter>
        {/each}
      </defs>

      <g class="terrain-layer">
        <foreignObject
          x="0"
          y="0"
          width={mapWidth}
          height={mapHeight}
          pointer-events="none"
          aria-hidden="true"
        >
          <div class="terrain-bitmap">
            <div
              class="map-terrain"
              use:mountMap={{ frame: mapFrame, attempt: renderAttempt }}
            ></div>
          </div>
        </foreignObject>
        {#each tiles as tile (tile.id)}
          <g
            class={tile.classes}
            role="button"
            tabindex={tile.id === focusedTile ? 0 : -1}
            aria-label={fogSet.has(tile.id) ? "Fog-covered terrain" : tile.label}
            data-tile={tile.id}
            onclick={() => actions.choose(tile.id)}
            onkeydown={(event) => handleTileKey(event, tile)}
          >
            <polygon points={tile.points} fill="transparent" pointer-events="all" />
            {#if tile.classes.includes("objective") || tile.classes.includes("recruitable")}
              <polygon class="tile-highlight-outline" points={tile.points} fill="none" />
            {/if}
            <polygon class="tile-outline" points={tile.points} fill="none" />
          </g>
        {/each}
      </g>

      <g class="ground-items pixel-art" pointer-events="none" aria-hidden="true">
        {#each game.groundItems.filter((item) => !fogSet.has(item.tile)) as ground, index (`${ground.tile}-${index}`)}
          {@const site = tiles.find((tile) => tile.id === ground.tile)!}
          {@const pile = game.groundItems.filter((item) => item.tile === ground.tile)}
          {@const offset = (pile.indexOf(ground) - (pile.length - 1) / 2) * 12}
          <g
            transform={`translate(${site.x + offset} ${site.y})`}
            filter={lighting.time === "night"
              ? `url(#night-tile-${site.col}-${site.row})`
              : undefined}
          >
            <ellipse cx="0" cy="14" rx="12" ry="5" fill="#d5bc68" opacity="0.4" />
            {#if pixelSprites[`terrain_prop_items_${ground.item.id}`]}
              <image
                href={pixelSprites[`terrain_prop_items_${ground.item.id}`]}
                x="-32"
                y="-32"
                width="64"
                height="64"
              />
            {:else}
              <path d="M0,-6 8,2 0,10 -8,2Z" fill="#e2c778" stroke="#302c28" stroke-width="2" />
            {/if}
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
            {#if tile.owner && !fogSet.has(tile.id)}
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
  .map-terrain {
    position: relative;
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
