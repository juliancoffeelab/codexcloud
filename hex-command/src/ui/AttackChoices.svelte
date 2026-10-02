<script lang="ts">
  import { Game, TYPES, TERRAIN, DAMAGE_TYPES } from '../game/engine';
  import type { Unit, Weapon, DamageBreakdown, Tile } from '../game/types';
  import { iconMarkup } from '../presentation/icons';
  import { damageExplanation } from '../presentation/combat';
  import HealthForecast from './HealthForecast.svelte';
  let { game, attacker, target, onattack }: { game: Game; attacker: Unit; target: Unit; onattack: (id: string) => void } = $props();
  let expanded = $state<Record<string, boolean>>({});
  function attackDescription(label: string, attack: Weapon, detail: DamageBreakdown, tile: Tile) {
    const terrain = detail.protection || detail.exposure ? `, against terrain: ${TERRAIN[tile.terrain].name.toLowerCase()}` : '';
    return `${label}: ${attack.damage} ${DAMAGE_TYPES[attack.damageType].toLowerCase()}, ${attack.name.toLowerCase()}, ${attack.category}${terrain}`;
  }
</script>
<div class="attack-actions" aria-label="Attack actions"><h3>Attack</h3>{#each TYPES[attacker.type].attacks as attack}
  {@const available = game.canAttackFrom(attacker, attacker.tile, target, attack.id)}{@const result = available ? game.previewDetails(attacker, target, attack.id) : null}{@const warning = result && (result.retaliation >= attacker.hp ? ', you fall' : result.damage >= target.hp ? ', enemy falls' : '')}{@const detailsId = `attack-details-${attack.id}`}
  <div class="attack-action-row"><div class="attack-box"><button class="attack-action" data-attack={attack.id} aria-label={`Attack with ${attack.name}${result ? `: deal ${result.damage} damage, take ${result.retaliation} damage${warning}` : ' — out of reach or blocked'}`} disabled={!available} onclick={() => onattack(attack.id)}><span class="attack-heading"><strong>{attack.name}</strong>{#if !result}<span>Out of reach or blocked</span>{/if}</span>{#if result}<span class="attack-forecasts"><HealthForecast unit={attacker} damage={result.retaliation} label="You" impact="Take"/><HealthForecast unit={target} damage={result.damage} label="Enemy" impact="Deal"/></span>{/if}</button>{#if result}<button class="attack-inspect" data-inspect={attack.id} aria-label={`Inspect ${attack.name.toLowerCase()} damage calculation`} aria-expanded={expanded[attack.id] ?? false} aria-controls={detailsId} onclick={() => expanded[attack.id] = !expanded[attack.id]}>{@html iconMarkup('search')}</button>{/if}</div>{#if result}<div id={detailsId} class="combat-explanation" hidden={!expanded[attack.id]}><div class="attack-calculation"><p>{attackDescription('Your attack', attack, result.damageDetail, game.tile(target.tile)!)}</p><p class="damage-equation">{damageExplanation(result.damageDetail)}</p></div><div class="attack-calculation"><p>{result.response && result.retaliationDetail ? attackDescription('Their attack', result.response, result.retaliationDetail, game.tile(attacker.tile)!) : 'Their attack: none'}</p><p class="damage-equation">{result.response && result.retaliationDetail ? damageExplanation(result.retaliationDetail) : result.damage >= target.hp ? '0 (enemy defeated)' : '0 (out of range)'}</p></div></div>{/if}</div>
{/each}</div>
