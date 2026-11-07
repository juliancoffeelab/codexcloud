<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    host = $bindable(),
    phase = "loading",
    loaded = 0,
    total = 0,
    audioLoaded = 0,
    audioTotal = 0,
    children,
    navigation,
  }: {
    host?: HTMLDivElement;
    phase?: "loading" | "preparing" | "ready";
    loaded?: number;
    total?: number;
    audioLoaded?: number;
    audioTotal?: number;
    children?: Snippet;
    navigation?: Snippet;
  } = $props();
</script>

<section class="map-section" aria-label="Battlefield">
  <div
    id="map"
    data-phaser-battlefield
    class="map-scroll phaser-battlefield battlefield-window"
    class:map-ready={phase === "ready"}
    aria-busy={phase !== "ready"}
  >
    <div class="battlefield-clip" bind:this={host}>{@render navigation?.()}</div>
    {#if phase !== "ready"}
      <div class="map-loading" role="status">
        <div class="loading-card">
          <strong aria-live="polite">
            {phase === "preparing" ? "Preparing battlefield…" : "Loading battlefield…"}
          </strong>
          <progress
            aria-label="Current battlefield resources"
            max={total || 1}
            value={total && phase === "loading" ? loaded : undefined}
          ></progress>
          <span aria-hidden="true">
            {#if total && phase === "loading"}
              {loaded} / {total}
              {audioTotal ? "ready" : "images ready"}
              {#if audioTotal}
                · {audioLoaded} / {audioTotal} sounds{/if}
            {:else}Preparing game resources…{/if}
          </span>
        </div>
      </div>
    {/if}
    {@render children?.()}
  </div>
</section>

<style>
  .battlefield-window {
    position: relative;
    width: 100%;
    height: 100%;
    max-height: none;
    border: 0;
    background: transparent;
    overflow: clip;
    touch-action: none;
    cursor: pointer;
  }
  .battlefield-clip {
    position: absolute;
    inset: 0;
    width: var(--battlefield-clip-width, 100%);
    height: 100%;
    overflow: clip;
    border: 1px solid var(--line);
    border-radius: 4px;
    background: #19231c;
  }
  .map-loading {
    position: absolute;
    inset: 0;
    z-index: 2;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: clamp(48px, 25vh, 180px);
    pointer-events: none;
  }
  .loading-card {
    display: grid;
    grid-template-rows: 21px 10px 21px;
    gap: 12px;
    width: min(320px, 80%);
    color: var(--text);
    font:
      14px/1.5 system-ui,
      sans-serif;
    font-variant-numeric: tabular-nums;
  }
  .loading-card progress {
    width: 100%;
    height: 10px;
    accent-color: var(--accent, #6fdec0);
  }
  .loading-card span {
    color: var(--muted);
  }
  .battlefield-window :global(canvas) {
    display: block;
    position: absolute;
    top: 0;
    left: 0;
    image-rendering: pixelated;
  }
</style>
