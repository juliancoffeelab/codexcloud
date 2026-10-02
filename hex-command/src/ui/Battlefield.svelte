<script lang="ts">
  import type { TileView } from "../game/view";
  import { tick } from "svelte";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { MAPS } from "../game/maps";
  import { battlefieldTiles } from "./battlefield-view";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
  }
  let { view, actions }: Props = $props();
  let map: SVGSVGElement | undefined;

  const game = $derived(view.game);
  const recruitment = $derived(view.recruitment);
  // Track the published turn even when the enemy turn hides the destination.
  const isPlayerTurn = $derived(game.turn === "player" && !game.winner);
  const recruitTileId = $derived(isPlayerTurn && recruitment ? view.inspectedTile : null);
  const tiles = $derived(battlefieldTiles(view, recruitTileId));
  const viewBox = $derived(`0 0 ${758 + 61 * (game.cols - 11)} ${452 + 53 * (game.rows - 7)}`);

  async function handleTileKey(event: KeyboardEvent, tile: TileView): Promise<void> {
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

    const renderRevision = view.renderRevision;
    actions.focusTile(nextTile.id);
    await tick();
    if (
      !map?.isConnected ||
      view.renderRevision !== renderRevision ||
      view.focusedTile !== nextTile.id
    )
      return;
    map.querySelector<SVGGElement>(`[data-tile="${nextTile.id}"]`)?.focus();
  }
</script>

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
      <!-- Redraw tile children together to preserve clip painting, cues, and restored focus. -->
      {#key view.renderRevision}
        <defs>
          {#each tiles as tile (tile.id)}
            <clipPath id={tile.clipId}>
              <polygon points={tile.points} />
            </clipPath>
          {/each}
        </defs>

        {#each tiles as tile (tile.id)}
          <g
            class={tile.classes}
            role="button"
            tabindex={tile.id === view.focusedTile ? 0 : -1}
            aria-label={tile.label}
            data-tile={tile.id}
            onclick={() => actions.choose(tile.id)}
            onkeydown={(event) => handleTileKey(event, tile)}
          >
            <polygon class="tile-ground" points={tile.points} fill={tile.fill} />
            <g clip-path={`url(#${tile.clipId})`}>
              {@html tile.terrainArtwork}
            </g>
            <polygon class="tile-outline" points={tile.points} fill="none" />

            {#if tile.occupant}
              {@const occupant = tile.occupant}
              <ellipse
                class="team-base"
                cx={tile.x}
                cy={tile.y + 19}
                rx="19"
                ry="7"
                fill={occupant.teamColor}
                stroke={occupant.teamColor}
              />
              {@html occupant.artwork}
              <rect class="hp-bg" x={tile.x - 19} y={tile.y + 27} width="38" height="4" rx="2" />
              <rect
                class="hp"
                x={tile.x - 19}
                y={tile.y + 27}
                width={occupant.healthWidth}
                height="4"
                rx="2"
              />
              {#if occupant.unit.attacked}
                <text class="acted" x={tile.x + 21} y={tile.y - 17}>✓</text>
              {/if}
            {/if}

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
      {/key}
    </svg>
  </div>
</section>
