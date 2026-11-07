<script lang="ts">
  import { iconMarkup } from "../presentation/icons";
  import type { AttackChoiceView } from "./combat-view";
  import HealthForecast from "./HealthForecast.svelte";

  interface Props {
    choice: AttackChoiceView;
    onAttack: (attackId: string) => void;
    busy?: boolean;
  }

  let { choice, onAttack, busy = false }: Props = $props();
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
      disabled={busy || !choice.available}
      data-activation="release"
      onclick={() => onAttack(choice.id)}
    >
      <span class="attack-heading" class:paired={choice.available}>
        {#each choice.available ? [choice.name, choice.responseName] : [choice.name] as name}
          <strong>
            {#each name.split(/(\([^)]*\))/) as part}
              {#if part.startsWith("(")}
                <span class="keep-together">{part}</span>
              {:else}{part}{/if}
            {/each}
          </strong>
        {/each}
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
