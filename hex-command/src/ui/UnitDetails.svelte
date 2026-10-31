<script lang="ts">
  import type { Artwork } from "../presentation/art";
  import { iconMarkup } from "../presentation/icons";
  import type { UnitDetailsView } from "./unit-panel-view";
  import type { UnitType } from "../game/types";
  import UnitCard from "./UnitCard.svelte";
  import Arsenal from "./Arsenal.svelte";
  import { signedTerrainValue } from "./terrain-effects";

  interface Props {
    details: UnitDetailsView;
    artwork: Artwork;
    onAdvance: (type: UnitType) => void;
    onDrop?: (item: string) => void;
  }

  let { details, artwork, onAdvance, onDrop }: Props = $props();
</script>

<div class="unit-detail">
  <UnitCard unit={details.unit} faction={details.faction} {artwork} showName showProgress />
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
    {#each details.ability.facts as fact}
      <p class="ability-fact">{fact}</p>
    {/each}
    <p>{details.ability.description}</p>
    {#each details.promotions as promotion (promotion.type)}
      <button class="primary" onclick={() => onAdvance(promotion.type)}>
        Advance to {promotion.name} · {promotion.hp} HP
      </button>
      <p>{promotion.description}</p>
    {/each}
    <Arsenal unit={details.unit} {onDrop} />
    <div class="movement-costs">
      <h3>Terrain effects</h3>
      <table class="weapon-table terrain-effects-table" aria-label="Unit terrain effects">
        <thead>
          <tr>
            <th scope="col">Terrain</th>
            <th scope="col">Cost</th>
            <th scope="col">Damage received</th>
          </tr>
        </thead>
        <tbody>
          {#each details.terrainEffects as terrain (terrain.terrain)}
            <tr class={terrain.className}>
              <th scope="row">{terrain.name}</th>
              <td
                class="terrain-cost"
                title={terrain.className === "blocked" ? "Impassable" : undefined}
              >
                <b>{terrain.label}</b>
              </td>
              <td>
                {#each terrain.facts as fact}
                  <span class="terrain-damage-fact">
                    <b class:resistant={fact.value < 0} class:vulnerable={fact.value > 0}>
                      {signedTerrainValue(fact.value)}
                    </b>
                    <span class="terrain-damage-kind">
                      {fact.label ? `${fact.label} Damage` : "Damage"}
                    </span>
                  </span>
                {/each}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p>
        Cost is movement spent to enter. Damage is the change per hit, in addition to Damage
        received above.
      </p>
    </div>
  </div>
</div>
