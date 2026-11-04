<script lang="ts">
  import { renderErrorReport } from "../presentation/render-error";
  import { battlefieldCamera } from "../presentation/battlefield-camera";
  import { battlefieldScene } from "../presentation/battlefield-scene";
  import type { MotionKind } from "../presentation/motion";
  import { tick, untrack } from "svelte";
  import type { TileView } from "../game/view";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { TERRAIN } from "../game/catalog";
  import { MAPS } from "../game/maps";
  import { BattlefieldViewCache } from "./battlefield-view";
  import { battlefieldSpriteSize, mapDimensions } from "./geometry";
  import { TerrainViewCache, terrainAssetKey } from "../presentation/terrain-map";
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
  let activeRenderer = $state.raw<MapRenderer>();
  let keyboardTile = $state<string | null>(null);
  let reducedMotion = $state(false);
  $effect(() => {
    const preference = matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => {
      reducedMotion = preference.matches;
    };
    update();
    preference.addEventListener("change", update);
    return () => preference.removeEventListener("change", update);
  });
  $effect(() => {
    activeRenderer?.inspect(null, keyboardTile);
  });

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
  const sessionVersion = $derived(view.sessionVersion);
  const liveScene = $derived(
    battlefieldScene(game, tiles, reducedMotion, `${sessionVersion}:${game.mapId}`, artwork),
  );
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
  const staticPlan = $derived.by(() => {
    const current = untrack(() => game);
    return {
      width: mapWidth,
      height: mapHeight,
      pixelScale: 64 / battlefieldSpriteSize,
      tiles: [...terrainScene.tiles].map(([id, tile]) => ({
        ...tile,
        role: TERRAIN[current.tile(id)!.terrain].role,
      })),
    };
  });
  const mapFrame: MapFrameRequest = $derived({
    terrainKey: renderTerrainKey,
    plan: { ...staticPlan, lighting },
    prebuilt: terrainSprites[terrainAssetKey(terrainKey)],
    fog: fogTiles,
    battlefield: liveScene,
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
        const node = board.querySelector<SVGGElement>(`[data-tile="${id}"]`);
        if (!node) return [];
        const rect = node.getBoundingClientRect();
        // GPU sprites extend above the transparent hex; reveal their heads too.
        const pixelScale = board.getBoundingClientRect().width / mapWidth;
        const headroom = game.at(id) ? (battlefieldSpriteSize / 2 - 20) * pixelScale : 0;
        return [
          { left: rect.left, right: rect.right, top: rect.top - headroom, bottom: rect.bottom },
        ];
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

  let errorDialog: HTMLDialogElement | undefined;
  function showError(node: HTMLDialogElement) {
    errorDialog = node;
    node.showModal();
    return {
      destroy() {
        if (errorDialog === node) errorDialog = undefined;
      },
    };
  }
  let reportStatus = $state("");
  async function copyReport() {
    try {
      await navigator.clipboard.writeText(renderError ?? "");
      reportStatus = "Copied";
    } catch {
      reportStatus = "Copy unavailable; download the report instead.";
    }
  }
  function downloadReport() {
    const url = URL.createObjectURL(new Blob([renderError ?? ""], { type: "text/plain" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "hex-render-error.txt";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  function listenMotion(node: SVGSVGElement) {
    const handle = (event: Event) => {
      const { tile, kind } = (event as CustomEvent<{ tile: string; kind: MotionKind }>).detail;
      activeRenderer?.pulse(tile, kind);
    };
    node.addEventListener("battlefieldmotion", handle);
    return { destroy: () => node.removeEventListener("battlefieldmotion", handle) };
  }

  function mountMap(node: HTMLDivElement, request: { frame: MapFrameRequest; attempt: number }) {
    let renderer: MapRenderer | undefined;
    let attempt = -1;
    const update = (next: typeof request) => {
      try {
        if (attempt !== next.attempt) {
          // Release the failed GPU context before retrying. Keeping its buffers
          // alive can make an out-of-memory failure repeat in the replacement.
          renderer?.destroy();
          renderer = undefined;
          attempt = next.attempt;
          const currentAttempt = attempt;
          renderer = acquireMapRenderer(node, (message) => {
            if (attempt === currentAttempt) renderError = message;
          });
        }
        activeRenderer = renderer;
        renderer!.update(next.frame);
      } catch (error) {
        renderError = renderErrorReport("Map setup failed", error);
      }
    };
    update(request);
    return {
      update,
      destroy() {
        if (activeRenderer === renderer) activeRenderer = undefined;
        renderer?.destroy();
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
    <button class="render-error-reopen" onclick={() => errorDialog?.showModal()}>
      Map unavailable · Show error
    </button>
    <dialog
      use:showError
      class="render-error-dialog"
      aria-labelledby="render-error-title"
      aria-describedby="render-error-description"
    >
      <div class="render-error-heading">
        <h2 id="render-error-title">The map couldn’t render</h2>
        <button aria-label="Dismiss rendering error" onclick={() => errorDialog?.close()}>×</button>
      </div>
      <p id="render-error-description">
        Try restarting the map renderer. If it fails again, download the error report and share it
        with us.
      </p>
      <p class="render-error-message">{renderError.split("\n")[0]}</p>
      <details>
        <summary>Error details</summary>
        <pre
          style="white-space: pre-wrap; overflow-wrap: anywhere; max-height: 24rem; overflow: auto;">{renderError}</pre>
        <button onclick={copyReport}>Copy report</button>
        <button onclick={downloadReport}>Download report</button>
        <span aria-live="polite">{reportStatus}</span>
      </details>
      <button
        onclick={() => {
          renderError = undefined;
          renderAttempt++;
        }}
      >
        Retry map rendering
      </button>
    </dialog>
  {/if}
  <div class="map-scroll" bind:this={viewport} use:battlefieldCamera>
    <svg
      bind:this={map}
      id="map"
      data-gpu-battlefield
      use:listenMotion
      {viewBox}
      style={`--map-scale: ${game.cols / 11};`}
      role="group"
      aria-label={`${MAPS[game.mapId].name} battlefield`}
      aria-describedby="battlefield-controls"
    >
      <desc id="battlefield-controls">
        Drag to pan. Control or Command plus wheel to zoom. Keyboard: arrows to navigate, Enter to
        choose, plus and minus to zoom, zero to reset zoom.
      </desc>
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
            onfocus={(event) => {
              keyboardTile = event.currentTarget.matches(":focus-visible") ? tile.id : null;
            }}
            onblur={() => {
              keyboardTile = null;
            }}
          >
            <polygon points={tile.points} fill="transparent" pointer-events="all" />
          </g>
        {/each}
      </g>
    </svg>
  </div>
</section>

<style>
  .render-error-dialog {
    width: min(34rem, calc(100vw - 2rem));
    max-height: calc(100dvh - 2rem);
    overflow: auto;
    box-sizing: border-box;
    padding: 1.25rem;
    color: #e9e4d3;
    background: #18251c;
    border: 1px solid #82734d;
    border-radius: 0.75rem;
    box-shadow: 0 1rem 4rem #0009;
  }
  .render-error-dialog::backdrop {
    background: #060c08b8;
  }
  .render-error-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }
  .render-error-heading h2 {
    margin: 0;
    font-size: 1.3rem;
  }
  .render-error-heading button {
    font-size: 1.5rem;
    padding: 0.1rem 0.65rem;
  }
  .render-error-message {
    overflow-wrap: anywhere;
    color: #f3c5a0;
  }
  .render-error-dialog details {
    margin-block: 1rem;
  }
  .render-error-dialog summary {
    cursor: pointer;
  }
  .render-error-dialog pre {
    font-size: 0.75rem;
    background: #0d1711;
    padding: 0.75rem;
  }
  .render-error-reopen {
    position: absolute;
    z-index: 3;
    margin: 1rem;
  }

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
