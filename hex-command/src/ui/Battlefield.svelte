<script lang="ts">
  import type { TileView } from "../game/view";
  import type { BattleCommands, BattleView } from "../controller/view";
  import { MAPS } from "../game/maps";
  import { battlefieldTiles } from "./battlefield-view";
  import { hexColumnSpacing, hexRowSpacing } from "./geometry";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
  }
  let { view, actions }: Props = $props();
  let map: SVGSVGElement | undefined;

  const game = $derived(view.game);
  const selected = $derived(view.selected);
  const artwork = $derived(view.artwork);
  const recruitment = $derived(view.recruitment);
  // Track the published turn even when the enemy turn hides the destination.
  const isPlayerTurn = $derived(game.turn === "player" && !game.winner);
  const recruitTileId = $derived(isPlayerTurn && recruitment ? view.inspectedTile : null);
  const tiles = $derived(battlefieldTiles(game, selected, artwork, recruitTileId));
  const viewBox = $derived(
    `0 0 ${758 + hexColumnSpacing * (game.cols - 11)} ${452 + hexRowSpacing * (game.rows - 7)}`,
  );

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
        {#each tiles as tile (tile.id)}
          <clipPath id={tile.clipId}>
            <polygon points={tile.points} />
          </clipPath>
        {/each}
      </defs>

      <g class="terrain-layer">
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
            {#if tile.classes.includes("objective") || tile.classes.includes("recruitable")}
              <polygon class="tile-highlight-outline" points={tile.points} fill="none" />
            {/if}
            <polygon class="tile-outline" points={tile.points} fill="none" />
          </g>
        {/each}
      </g>

      <!-- Paint every terrain first; adjacent hexes must not hide units or bars. -->
      <g class="unit-layer" pointer-events="none" aria-hidden="true">
        {#each tiles as tile (tile.id)}
          <g data-unit-tile={tile.id}>
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
            {/if}
          </g>
        {/each}
      </g>

      <!-- Health and state indicators remain above every terrain and sprite. -->
      <g class="indicator-layer" pointer-events="none" aria-hidden="true">
        {#each tiles as tile (tile.id)}
          <g data-indicator-tile={tile.id}>
            {#if tile.occupant}
              {@const occupant = tile.occupant}
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
              {#if occupant.unit.retreating}
                <text class="retreat-marker" x={tile.x - 28} y={tile.y - 14}>↶</text>
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
      </g>
    </svg>
  </div>
</section>
