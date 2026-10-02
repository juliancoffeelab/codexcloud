<script lang="ts">
  import type { Unit } from "../game/types";
  import { iconMarkup } from "../presentation/icons";
  import { arsenalView } from "./arsenal-view";

  let { unit }: { unit: Unit } = $props();
  const arsenal = $derived(arsenalView(unit));
</script>

<div class="arsenal">
  <h3>Attacks</h3>
  <table class="weapon-table" aria-label="Unit attacks">
    <thead>
      <tr>
        <th scope="col">Attack</th>
        <th scope="col">Damage</th>
        <th scope="col">Range</th>
      </tr>
    </thead>
    <tbody>
      {#each arsenal.weapons as weapon}
        <tr>
          <th scope="row">
            <span class="weapon-name">
              {@html iconMarkup(weapon.damageType)}
              <strong>{weapon.name}</strong>
            </span>
            <span class="weapon-detail weapon-category">{weapon.categoryLabel}</span>
          </th>
          <td class="weapon-damage">
            <b>{weapon.damage}</b>
            {" "}{weapon.damageLabel}
          </td>
          <td>{weapon.rangeLabel}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
<div class="resistances">
  <h3>Damage received</h3>
  <div class="resistance-list">
    {#each arsenal.resistances as resistance}
      <span class={`resistance ${resistance.className}`}>
        <span>{resistance.name}</span>
        <b>{resistance.multiplier}%</b>
      </span>
    {/each}
  </div>
  <p class="resistance-hint">Base damage multiplier, rounded before terrain cover.</p>
</div>
