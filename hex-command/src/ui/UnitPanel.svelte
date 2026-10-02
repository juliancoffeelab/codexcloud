<script lang="ts">
  import type { BattleController, BattleView } from '../controller/battle';
  import { TYPES, TERRAIN } from '../game/engine';
  import { iconMarkup } from '../presentation/icons';
  import { abilityInfo } from '../presentation/unit-ability';
  import UnitCard from './UnitCard.svelte';
  import Arsenal from './Arsenal.svelte';
  import AttackChoices from './AttackChoices.svelte';
  let { view, controller, onnewbattle }: { view: BattleView; controller: BattleController; onnewbattle: () => void } = $props();
  const game = $derived(view.game);
  const unit = $derived(game.unit(view.selected));
  const victim = $derived(game.unit(view.target));
  const attackAvailable = $derived(unit && victim && game.targets(unit.id).includes(victim) && game.turn === 'player' && !game.winner && unit.side === 'player');
</script>
<section id="unit-panel" class="panel" aria-label="Selected unit" hidden={!game.winner && !unit}>
{#if game.winner}<div class="unit-title"><h2>{game.winner === 'player' ? 'The field is yours.' : 'An army needs a leader.'}</h2></div><p>{game.winner === 'player' ? 'Your army defeated the enemy commander.' : 'Keep your commander protected and use villages to grow your army.'}</p><button id="play-again" class="primary" onclick={onnewbattle}>Play again</button>
{:else if unit}{@const type = TYPES[unit.type]}{@const tile = game.tile(unit.tile)!}{@const ability = abilityInfo(unit)}
  {#if attackAvailable && victim}<div class="duel"><UnitCard {unit} artStyle={view.artStyle}/><span class="duel-divider" aria-hidden="true">⚔</span><UnitCard unit={victim} artStyle={view.artStyle}/></div>{#key `${unit.id}-${victim.id}-${view.renderRevision}`}<AttackChoices {game} attacker={unit} target={victim} onattack={id => controller.attackSelected(id)}/>{/key}
  {:else}<div class="unit-detail"><UnitCard {unit} artStyle={view.artStyle}/><div class="unit-info"><div class="unit-stats"><span class="unit-stat">{@html iconMarkup('move')}<span>Move: <b>{type.move}</b></span></span><p class="unit-state"><span>{TERRAIN[tile.terrain].name}</span><span>{unit.attacked ? 'Turn finished' : unit.moved ? 'Can still attack' : 'Ready'}</span></p></div><p class="ability">{ability.name}</p><p>{ability.description}</p><Arsenal {unit}/></div></div>{/if}
{/if}
</section>
