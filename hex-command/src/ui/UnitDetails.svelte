<script lang="ts">
  import type { Artwork } from "../presentation/art";
  import { iconMarkup } from "../presentation/icons";
  import type { UnitDetailsView } from "./unit-panel-view";
  import type { UnitType } from "../game/types";
  import UnitCard from "./UnitCard.svelte";
  import Arsenal from "./Arsenal.svelte";

  interface Props {
    details: UnitDetailsView;
    artwork: Artwork;
    onAdvance: (type: UnitType) => void;
  }

  let { details, artwork, onAdvance }: Props = $props();
</script>

<div class="unit-detail">
  <UnitCard unit={details.unit} {artwork} showName />
  <div class="unit-info">
    <div class="unit-stats">
      <span class="unit-stat">
        {@html iconMarkup("move")}
        <span>
          Move: <b>{details.definition.move}</b>
        </span>
      </span>
      <p class="unit-state">
        <span>{details.terrainName}</span>
        <span>{details.actionStatus}</span>
      </p>
    </div>
    <p class="ability">{details.ability.name}</p>
    <p>{details.ability.description}</p>
    <p>
      Level {details.definition.level ?? 1} · {details.definition.experienceNeeded
        ? `${details.unit.experience ?? 0} / ${details.definition.experienceNeeded} XP`
        : "Fully advanced"}
    </p>
    {#if details.definition.experienceNeeded}
      <p>
        Earn experience by dealing damage and defeating enemies. Advance while ready; wounds carry
        over and advancement uses this turn.
      </p>
    {/if}
    {#each details.promotions as promotion (promotion.type)}
      <button class="primary" onclick={() => onAdvance(promotion.type)}>
        Advance to {promotion.name} · {promotion.hp} HP
      </button>
      <p>{promotion.description}</p>
    {/each}
    <Arsenal unit={details.unit} />
    <div class="movement-costs">
      <h3>Movement costs</h3>
      <dl>
        {#each details.movementCosts as movement (movement.terrain)}
          <div class={`movement-cost-row ${movement.className}`}>
            <dt>{movement.name}</dt>
            <dd>{movement.label}</dd>
          </div>
        {/each}
      </dl>
      <p>Movement spent to enter each terrain.</p>
    </div>
  </div>
</div>
