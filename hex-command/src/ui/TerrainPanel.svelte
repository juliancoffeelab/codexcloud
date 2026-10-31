<script lang="ts">
  import type { BattleCommands, BattleView } from "../controller/view";
  import { artSprite } from "../presentation/art";
  import { terrainLayout } from "../presentation/terrain-layout";
  import { terrainPanelView } from "./terrain-view";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
  }
  let { view, actions }: Props = $props();

  const panel = $derived(terrainPanelView(view));
  const frameName = $derived(panel ? terrainLayout(panel.tile, view.game.mapId).name : "");
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
              title={recruit.description}
              disabled={!recruit.enabled}
              onclick={() => actions.recruitHere(recruit.type, panel.tile.id)}
            >
              <span>
                {recruit.name}{#if view.game.campaign}<small class="recruit-preview">
                    {recruit.hp} HP &nbsp; {recruit.move} move
                  </small>{/if}
              </span>
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
          view.artwork,
          "terrain",
          frameName,
          0,
          0,
          80,
          80,
          "terrain-card-art",
          "player",
          panel.tile.floor,
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
