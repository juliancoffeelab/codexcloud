<script lang="ts">
  import type { UnitView } from "../game/view";
  import { TYPES } from "../game/engine";
  import { artSprite, type ArtStyle } from "../presentation/art";

  interface Props {
    unit: UnitView;
    artStyle: ArtStyle;
    showName?: boolean;
  }
  let { unit, artStyle, showName = false }: Props = $props();

  const definition = $derived(TYPES[unit.type]);
  const teamLabel = $derived(unit.side === "player" ? "Your unit" : "Enemy unit");
  const healthPercent = $derived((unit.hp / definition.hp) * 100);
  const portrait = $derived(
    artSprite(
      artStyle,
      "units",
      unit.type,
      0,
      0,
      72,
      90,
      `portrait ${unit.side === "enemy" ? "enemy-unit" : ""}`,
      unit.side,
    ),
  );
</script>

<div class={`combatant ${unit.side}`}>
  <span class="unit-side">{teamLabel}</span>
  {@html portrait}
  <h2>{showName && unit.name ? unit.name : definition.name}</h2>
  {#if unit.retreating}<span class="retreat-label">Retreating</span>{/if}
  {#if showName && unit.name}<span class="named-unit-type">{definition.name}</span>{/if}
  <div class="health-numbers">
    <b>{unit.hp}</b>
    <span>{" "}/ {definition.hp} HP</span>
  </div>
  <div class="health-track" role="img" aria-label={`${unit.hp} of ${definition.hp} current health`}>
    <span class="health-left" style={`width:${healthPercent}%`}></span>
  </div>
</div>
