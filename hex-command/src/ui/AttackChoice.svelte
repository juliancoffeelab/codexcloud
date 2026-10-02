<script lang="ts">
  import { iconMarkup } from "../presentation/icons";
  import type { AttackChoiceView } from "./combat-view";
  import HealthForecast from "./HealthForecast.svelte";

  interface Props {
    choice: AttackChoiceView;
    onAttack: (attackId: string) => void;
  }

  let { choice, onAttack }: Props = $props();
  let expanded = $state(false);

  function toggleExplanation() {
    expanded = !expanded;
  }
</script>

<div class="attack-action-row">
  <div class="attack-box">
    <button
      class="attack-action"
      data-attack={choice.id}
      aria-label={choice.buttonLabel}
      disabled={!choice.available}
      onclick={() => onAttack(choice.id)}
    >
      <span class="attack-heading">
        <strong>{choice.name}</strong>
        {#if !choice.available}
          <span>Out of reach or blocked</span>
        {/if}
      </span>
      {#if choice.available}
        <span class="attack-forecasts">
          <HealthForecast forecast={choice.playerForecast} />
          <HealthForecast forecast={choice.enemyForecast} />
        </span>
      {/if}
    </button>
    {#if choice.available}
      <button
        class="attack-inspect"
        data-inspect={choice.id}
        aria-label={choice.inspectLabel}
        aria-expanded={expanded}
        aria-controls={choice.detailsId}
        onclick={toggleExplanation}
      >
        {@html iconMarkup("search")}
      </button>
    {/if}
  </div>
  {#if choice.available}
    <div id={choice.detailsId} class="combat-explanation" hidden={!expanded}>
      <div class="attack-calculation">
        <p>{choice.attackCalculation.description}</p>
        <p class="damage-equation">{choice.attackCalculation.equation}</p>
      </div>
      <div class="attack-calculation">
        <p>{choice.retaliationCalculation.description}</p>
        <p class="damage-equation">{choice.retaliationCalculation.equation}</p>
      </div>
    </div>
  {/if}
</div>
