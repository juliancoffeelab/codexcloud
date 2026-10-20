<script lang="ts">
  import type { GameView, UnitView } from "../game/view";
  import { attackChoiceViews } from "./combat-view";
  import AttackChoice from "./AttackChoice.svelte";

  interface Props {
    game: GameView;
    attacker: UnitView;
    target: UnitView;
    onAttack: (attackId: string) => void;
    busy?: boolean;
  }

  let { game, attacker, target, onAttack, busy = false }: Props = $props();
  const choices = $derived(attackChoiceViews(game, attacker, target));
</script>

<div class="attack-actions" aria-label="Attack actions">
  {#each choices as choice (choice.id)}
    <AttackChoice {choice} {onAttack} {busy} />
  {/each}
</div>
