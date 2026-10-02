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
  <UnitCard unit={details.unit} {artStyle} />
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
  </div>
</div>
