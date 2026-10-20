<script lang="ts">
  import type { UnitView } from "../game/view";
  import { TYPES } from "../game/catalog";
  import { artSprite, type Artwork } from "../presentation/art";

  interface Props {
    unit: UnitView;
    artwork: Artwork;
    showName?: boolean;
    showProgress?: boolean;
    hp?: number;
  }
  let { unit, artwork, showName = false, showProgress = false, hp = unit.hp }: Props = $props();

  const definition = $derived(TYPES[unit.type]);
  const healthPercent = $derived((hp / definition.hp) * 100);
  const portrait = $derived(
    artSprite(
      artwork,
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

<div
  class={`combatant ${unit.side}`}
  role="group"
  aria-label={unit.side === "player" ? "Your unit" : "Enemy unit"}
>
  {@html portrait}
  <div class="unit-heading">
    <h2>{showName && unit.name ? unit.name : definition.name}</h2>
    {#if showProgress}
      <span class="unit-progress" title={`Level ${definition.level ?? 1}`}>
        L{definition.level ?? 1} · {definition.experienceNeeded
          ? `${unit.experience ?? 0}/${definition.experienceNeeded} XP`
          : "Max"}
      </span>
    {/if}
  </div>
  {#if unit.retreating}<span class="retreat-label">Retreating</span>{/if}
  {#if showName && unit.name}<span class="named-unit-type">{definition.name}</span>{/if}
  <div class="health-numbers">
    <b>{hp}</b>
    <span>{" "}/ {definition.hp} HP</span>
  </div>
  <div class="health-track" role="img" aria-label={`${hp} of ${definition.hp} current health`}>
    <span class="health-left" style={`width:${healthPercent}%`}></span>
  </div>
</div>
