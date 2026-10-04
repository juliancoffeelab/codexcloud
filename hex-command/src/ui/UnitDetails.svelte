<script lang="ts">
  import type { ArtStyle } from "../presentation/art";
  import { iconMarkup } from "../presentation/icons";
  import type { UnitDetailsView } from "./unit-panel-view";
  import UnitCard from "./UnitCard.svelte";
  import Arsenal from "./Arsenal.svelte";

  interface Props {
    details: UnitDetailsView;
    artStyle: ArtStyle;
  }

  let { details, artStyle }: Props = $props();
</script>

<div class="unit-detail">
  <UnitCard unit={details.unit} {artStyle} showName />
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
