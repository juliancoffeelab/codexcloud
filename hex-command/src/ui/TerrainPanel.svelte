<script lang="ts">
  import { recruitTypes, type BattleController, type BattleView } from '../controller/battle';
  import { TYPES, TERRAIN } from '../game/engine';
  import type { Tile, TerrainType } from '../game/types';
  import { artSprite } from '../presentation/art';
  let { view, controller }: { view: BattleView; controller: BattleController } = $props();
  const game = $derived(view.game);
  const tile = $derived(game.tile(view.inspectedTile));
  const context = $derived(controller.recruitmentContext(tile));
  const active = $derived(game.turn === 'player' && !game.winner);
  const descriptions: Record<TerrainType, string> = { plains: 'Open grassland with easy footing.', forest: 'Trees shelter units and obstruct ranged shots.', hill: 'Rocky high ground offers cover and obstructs ranged shots.', water: 'Deep water cannot be crossed.', bridge: 'A sturdy crossing over water with easy footing, but no protection.', ford: 'Shallow water can be crossed on foot. Heavy armor slows movement, and the open water leaves units exposed.', village: 'Capture this settlement by moving a unit onto it.', keep: 'A fortified home for its army’s commander.' };
  function facts(tile: Tile) {
    const result: string[] = [], info = TERRAIN[tile.terrain];
    if (Number.isFinite(info.cost)) {
      result.push(tile.terrain === 'ford' ? 'Movement: 3 for soldiers and commanders; 2 for archers and scouts' : `Movement: ${info.cost}${['forest', 'hill'].includes(tile.terrain) ? ' (scouts: 1)' : ''}`);
      result.push(tile.terrain === 'ford' ? 'Exposure: +1 damage received from all attacks' : tile.terrain === 'hill' ? 'Protection: −1 damage from all attacks' : tile.terrain === 'forest' ? 'Protection: −1 damage from ranged attacks' : 'Protection: none');
    } else result.push('Movement: impassable');
    if (['forest', 'hill'].includes(tile.terrain)) result.push('Blocks ranged shots through this hex');
    if (tile.terrain === 'village') result.push('+2 gold at the start of its owner’s turn', 'Heals a friendly occupant by 2 HP at the start of its turn');
    if (tile.terrain === 'keep') result.push('Its owner’s commander can recruit into adjacent empty land hexes', 'Select an empty adjacent hex to recruit', 'Keeps cannot be captured');
    return result;
  }
</script>
<section id="terrain-panel" class="panel" aria-label="Selected terrain" hidden={!tile}>{#if tile}
{#if context}<div class="terrain-recruitment"><h3>Recruit here</h3><div class="recruit-list">{#each recruitTypes as name}{@const type = TYPES[name]}{@const enabled = active && context.stationed && context.commander && game.recruitTiles(context.commander.id, name).some(destination => destination.id === tile.id)}<button class="recruit-button" data-recruit={name} disabled={!enabled} onclick={() => controller.recruitHere(name, tile.id)}><span>{type.name}</span><span>{type.cost} gold</span></button>{/each}</div><p class="recruit-hint">{!context.stationed ? 'Return your commander to this keep to recruit.' : !active ? 'Recruit on your turn.' : 'New recruits act next turn.'}{active && context.stationed && game.gold.player < TYPES.scout.cost ? ' Not enough gold.' : ''}</p></div>{/if}<div class="terrain-detail"><div class="terrain-portrait">{@html artSprite(view.artStyle, 'terrain', tile.terrain, 0, 0, 80, 80, 'terrain-card-art')}<span class="unit-side">Terrain</span></div><div class="terrain-info"><h2>{TERRAIN[tile.terrain].name}</h2>{#if ['village', 'keep'].includes(tile.terrain)}<p class={`terrain-owner ${tile.owner || 'neutral'}`}>{tile.owner === 'player' ? 'Your ' : tile.owner === 'enemy' ? 'Enemy ' : 'Unclaimed '}{tile.terrain}</p>{/if}<p>{descriptions[tile.terrain]}</p><ul class="terrain-facts" role="list">{#each facts(tile) as fact}<li>{fact}</li>{/each}</ul></div></div>
{/if}</section>
