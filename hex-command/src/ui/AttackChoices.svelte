<script lang="ts">
  import type { Game } from "../game/engine";
  import type { Unit } from "../game/types";
  import { attackChoiceViews } from "./combat-view";
  import AttackChoice from "./AttackChoice.svelte";

  interface Props {
    game: Game;
    attacker: Unit;
    target: Unit;
    onAttack: (attackId: string) => void;
  }

  let { game, attacker, target, onAttack }: Props = $props();
  const choices = $derived(attackChoiceViews(game, attacker, target));
</script>

<div class="attack-actions" aria-label="Attack actions">
  <h3>Attack</h3>
  {#each choices as choice (choice.id)}
    <AttackChoice {choice} {onAttack} />
  {/each}
</div>
