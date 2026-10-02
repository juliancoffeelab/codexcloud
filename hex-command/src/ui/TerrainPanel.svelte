<script lang="ts">
  import type { BattleController, BattleView } from "../controller/battle";
  import { artSprite } from "../presentation/art";
  import { terrainPanelView } from "./terrain-view";

  interface Props {
    view: BattleView;
    controller: BattleController;
  }
  let { view, controller }: Props = $props();

  const inspectedTile = $derived(view.game.tile(view.inspectedTile));
  const recruitment = $derived(controller.recruitmentContext(inspectedTile));
  const panel = $derived(terrainPanelView(view, recruitment));
</script>

<section id="terrain-panel" class="panel" aria-label="Selected terrain" hidden={!panel}>
  {#if panel}
    {#if panel.recruitment}
      <div class="terrain-recruitment">
        <h3>Recruit here</h3>
        <div class="recruit-list">
          {#each panel.recruitment.options as recruit}
            <button
              class="recruit-button"
              data-recruit={recruit.type}
              disabled={!recruit.enabled}
              onclick={() => controller.recruitHere(recruit.type, panel.tile.id)}
            >
              <span>{recruit.name}</span>
              <span>{recruit.cost} gold</span>
            </button>
          {/each}
        </div>
        <p class="recruit-hint">{panel.recruitment.hint}</p>
      </div>
    {/if}
    <div class="terrain-detail">
      <div class="terrain-portrait">
        {@html artSprite(
          view.artStyle,
          "terrain",
          panel.tile.terrain,
          0,
          0,
          80,
          80,
          "terrain-card-art",
        )}
        <span class="unit-side">Terrain</span>
      </div>
      <div class="terrain-info">
        <h2>{panel.name}</h2>
        {#if panel.ownership}
          <p class={`terrain-owner ${panel.ownership.className}`}>{panel.ownership.label}</p>
        {/if}
        <p>{panel.description}</p>
        <ul class="terrain-facts" role="list">
          {#each panel.facts as fact}
            <li>{fact}</li>
          {/each}
        </ul>
      </div>
    </div>
  {/if}
</section>
