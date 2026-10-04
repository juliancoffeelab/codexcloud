<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import { ART_STYLES, artSpriteInfo } from "../presentation/art";
  import { DEVELOPER_TERRAIN_SAMPLES, DEVELOPER_UNIT_SAMPLES } from "../game/developer-map";

  let {
    view,
    actions,
    overview = false,
  }: { view: BattleView; actions: BattleCommands; overview?: boolean } = $props();
  const game = $derived(view.game);
  const unit = $derived(game.unit(view.selected));
  const tile = $derived(game.tile(view.inspectedTile ?? unit?.tile));
  const spriteSources = $derived(
    tile
      ? [
          {
            kind: "Terrain sprite",
            info: artSpriteInfo(view.artStyle, "terrain", tile.terrain),
          },
          ...(unit
            ? [
                {
                  kind: "Unit sprite",
                  info: artSpriteInfo(view.artStyle, "units", unit.type, unit.side),
                },
              ]
            : []),
        ]
      : [],
  );
  const unitTypes = new Set(DEVELOPER_UNIT_SAMPLES.map((sample) => sample.type)).size;

  function changeArt(event: Event): void {
    const style = (event.currentTarget as HTMLSelectElement).value;
    if (style === "normal" || style === "low-poly" || style === "pixel")
      void actions.setArtStyle(style).catch((error) => actions.error(error));
  }
</script>

{#if overview}
  <section class="showcase-overview" aria-label="Showcase overview">
    <div class="overview-heading">
      <p>
        {DEVELOPER_TERRAIN_SAMPLES.length} terrains · {unitTypes} unit types · {DEVELOPER_UNIT_SAMPLES.length}
        units
      </p>
      <label>
        <span>Art style</span>
        <select value={view.artStyle} disabled={view.artLoading} onchange={changeArt}>
          {#each Object.entries(ART_STYLES) as [style, details]}
            <option value={style}>{details.label}</option>
          {/each}
        </select>
      </label>
    </div>
    <p class="showcase-key">
      Each pair: <span class="player">left / player</span>
      ,
      <span class="enemy">right / enemy</span>
      . Labels are raw IDs. Select a hex to inspect it.
    </p>
  </section>
{:else}
  <section class="showcase-selection" aria-label="Selected showcase sample">
    <h3>Sample details</h3>
    {#if tile}
      <dl>
        <div>
          <dt>Terrain</dt>
          <dd><code>{tile.terrain}</code></dd>
        </div>
        <div>
          <dt>Coordinates</dt>
          <dd>
            <code>({tile.col}, {tile.row})</code>
            <span>zero-based</span>
          </dd>
        </div>
        {#if unit}
          <div>
            <dt>Unit type</dt>
            <dd><code>{unit.type}</code></dd>
          </div>
          <div>
            <dt>Unit ID</dt>
            <dd><code>{unit.id}</code></dd>
          </div>
          <div>
            <dt>Team</dt>
            <dd class={unit.side}><code>{unit.side}</code></dd>
          </div>
        {/if}
      </dl>
      {#each spriteSources as sprite (sprite.kind)}
        {#if sprite.info}
          <div class="sprite-source">
            <h4>{sprite.kind}</h4>
            <dl>
              <div>
                <dt>Source</dt>
                <dd>
                  <a href={sprite.info.source} target="_blank" rel="noreferrer">
                    {sprite.info.source}
                  </a>
                </dd>
              </div>
              <div>
                <dt>Native size</dt>
                <dd><code>{sprite.info.width} × {sprite.info.height}</code></dd>
              </div>
              <div>
                <dt>Crop origin</dt>
                <dd><code>({sprite.info.crop.x}, {sprite.info.crop.y})</code></dd>
              </div>
              <div>
                <dt>Crop size</dt>
                <dd><code>{sprite.info.crop.width} × {sprite.info.crop.height}</code></dd>
              </div>
            </dl>
            {#if sprite.info.composite}<p>{sprite.info.composite}</p>{/if}
          </div>
        {/if}
      {/each}
    {:else}
      <p>Select a terrain or unit hex.</p>
    {/if}
  </section>
{/if}

<style>
  .showcase-overview,
  .showcase-selection {
    font:
      13px/1.5 system-ui,
      sans-serif;
    color: var(--muted);
  }
  .showcase-overview {
    margin-bottom: 12px;
  }
  .overview-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px 16px;
  }
  .overview-heading label {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .overview-heading select {
    min-width: 110px;
  }
  .showcase-key {
    margin-top: 8px;
  }
  .player {
    color: #6fdec0;
  }
  .enemy {
    color: #ff9590;
  }
  .showcase-selection {
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
  }
  h3 {
    color: var(--gold);
    font-size: 14px;
  }
  h4 {
    margin: 0;
    color: var(--text);
    font-size: 13px;
  }
  .sprite-source {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--line);
  }
  .sprite-source a {
    color: var(--mint);
    text-decoration: underline;
    text-underline-offset: 2px;
    font:
      12px ui-monospace,
      monospace;
  }
  .sprite-source a:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }
  .sprite-source p {
    margin-top: 8px;
  }
  dl {
    margin: 8px 0 0;
  }
  dl > div {
    display: grid;
    grid-template-columns: 100px minmax(0, 1fr);
    gap: 8px;
    padding: 3px 0;
  }
  dd {
    margin: 0;
    overflow-wrap: anywhere;
    color: var(--text);
  }
  dd span {
    color: var(--muted);
  }
  code {
    font-family: ui-monospace, monospace;
    font-size: 12px;
  }
</style>
