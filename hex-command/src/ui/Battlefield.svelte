<script lang="ts">
  import { tick } from 'svelte';
  import type { BattleController, BattleView } from '../controller/battle';
  import { TYPES, TERRAIN } from '../game/engine';
  import { MAPS } from '../game/maps';
  import type { Tile, Unit } from '../game/types';
  import { artSprite } from '../presentation/art';
  import { hexPoints, tileCenter, colors } from './geometry';
  let { view, controller }: { view: BattleView; controller: BattleController } = $props();
  let map: SVGSVGElement;
  const game = $derived(view.game);
  const unit = $derived(game.unit(view.selected));
  const canAct = $derived(game.turn === 'player' && !game.winner && unit?.side === 'player');
  const moves = $derived(canAct ? game.reachable(view.selected) : new Map<string, number>());
  const targets = $derived(canAct ? game.targets(view.selected) : []);
  const recruitTile = $derived(controller.active() && controller.recruitmentContext(game.tile(view.inspectedTile)) ? view.inspectedTile : null);
  function tileClass(tile: Tile) { return ['hex', `terrain-${tile.terrain}`, unit?.tile === tile.id ? 'selected' : '', moves.has(tile.id) ? 'reachable' : '', targets.some(u => u.tile === tile.id) ? 'target' : '', tile.id === recruitTile ? 'recruitable' : ''].filter(Boolean).join(' '); }
  function label(tile: Tile, occupant: Unit | undefined) { return `${TERRAIN[tile.terrain].name}, column ${tile.col + 1}, row ${tile.row + 1}${tile.owner ? `, ${tile.owner === 'player' ? 'your' : 'enemy'} territory` : ''}${occupant ? `, ${occupant.side === 'player' ? 'your' : 'enemy'} ${TYPES[occupant.type].name}, ${occupant.hp} health${occupant.attacked ? ', turn finished' : ''}` : ''}${moves.has(tile.id) ? ', move here' : ''}${targets.some(u => u.tile === tile.id) ? ', attack available' : ''}`; }
  function keydown(event: KeyboardEvent, tile: Tile) {
    if (['Enter', ' '].includes(event.key)) { event.preventDefault(); controller.choose(tile.id); return; }
    let { col, row } = tile;
    if (event.key === 'ArrowLeft') col--; else if (event.key === 'ArrowRight') col++;
    else if (event.key === 'ArrowUp') row--; else if (event.key === 'ArrowDown') row++; else return;
    event.preventDefault(); const next = game.tile(`${col},${row}`);
    if (next) { controller.focusTile(next.id); void tick().then(() => map.querySelector<SVGGElement>(`[data-tile="${next.id}"]`)?.focus()); }
  }
</script>
<section class="map-section" aria-label="Battlefield"><div class="map-scroll"><svg bind:this={map} id="map" viewBox={`0 0 ${758 + 61 * (game.cols - 11)} ${452 + 53 * (game.rows - 7)}`} style={`--map-scale: ${game.cols / 11};`} role="group" aria-label={`${MAPS[game.mapId].name} battlefield`}>
  <!-- Match the original tile lifetime, including SVG clip painting and focus restoration. -->
  {#key view.renderRevision}
  <defs>{#each game.tiles as tile (tile.id)}{@const center = tileCenter(tile)}<clipPath id={`hex-clip-${tile.col}-${tile.row}`}><polygon points={hexPoints(center.x, center.y)}/></clipPath>{/each}</defs>
  {#each game.tiles as tile (tile.id)}
    {@const occupant = game.at(tile.id)}{@const center = tileCenter(tile)}{@const x = center.x}{@const y = center.y}
    <g class={tileClass(tile)} role="button" tabindex={tile.id === view.focusedTile ? 0 : -1} aria-label={label(tile, occupant)} data-tile={tile.id} onclick={() => controller.choose(tile.id)} onkeydown={event => keydown(event, tile)}>
      <polygon class="tile-ground" points={hexPoints(x, y)} fill={colors[tile.terrain]}/>
      <g clip-path={`url(#hex-clip-${tile.col}-${tile.row})`}>{@html artSprite(view.artStyle, 'terrain', tile.terrain, x - 35, y - 35, 70, 70, 'terrain-art')}</g>
      <polygon class="tile-outline" points={hexPoints(x, y)} fill="none"/>
      {#if occupant}{@const teamColor = occupant.side === 'player' ? '#6fdec0' : '#ff9590'}{@const spriteSize = occupant.type === 'commander' ? 62 : 56}
        <ellipse class="team-base" cx={x} cy={y + 19} rx="19" ry="7" fill={teamColor} stroke={teamColor}/>{@html artSprite(view.artStyle, 'units', occupant.type, x - spriteSize / 2, y + 20 - spriteSize, spriteSize, spriteSize, `unit-art ${occupant.side === 'enemy' ? 'enemy-unit' : ''} ${occupant.attacked ? 'spent-unit' : ''}`, occupant.side)}<rect class="hp-bg" x={x - 19} y={y + 27} width="38" height="4" rx="2"/><rect class="hp" x={x - 19} y={y + 27} width={(38 * occupant.hp) / TYPES[occupant.type].hp} height="4" rx="2"/>{#if occupant.attacked}<text class="acted" x={x + 21} y={y - 17}>✓</text>{/if}
      {/if}
      {#if tile.owner}<circle class="territory-badge" cx={x + 23} cy={y + 17} r="5" fill={tile.owner === 'player' ? '#6fdec0' : '#ff9590'}/>{/if}
    </g>
  {/each}
  {/key}
</svg></div></section>
