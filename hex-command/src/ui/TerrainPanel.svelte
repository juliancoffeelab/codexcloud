<script lang="ts">
  import type { BattleCommands, BattleView } from "../controller/view";
  import { terrainPreviewFrame, type TerrainPreviewCache } from "../presentation/terrain-preview";
  import TerrainPreview from "./TerrainPreview.svelte";
  import { terrainPanelView } from "./terrain-view";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
    previews: TerrainPreviewCache;
  }
  let { view, actions, previews }: Props = $props();

  const panel = $derived(terrainPanelView(view));
  const frame = $derived(panel ? terrainPreviewFrame(panel.tile, view.artwork) : undefined);
</script>

<section id="terrain-panel" class="panel" aria-label="Selected terrain" hidden={!panel}>
  {#if panel}
    {#if panel.recruitment}
      <div class="terrain-recruitment">
        <h3>{panel.recruitment.options.length ? "Recruit here" : "Recruitment unavailable"}</h3>
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
        {#if frame}<TerrainPreview {frame} cache={previews} />{/if}
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
