<script lang="ts">
  import { errorDetails } from "../error-details";
  import { onMount } from "svelte";
  import type { HudArtworkCache } from "../presentation/hud-artwork";
  import type { MotionKind } from "../presentation/motion";
  import type { BattleCommands, BattleView } from "../controller/view";
  import BattlefieldWindow from "./BattlefieldWindow.svelte";
  import type {
    BattlefieldProgress,
    createPhaserBattlefield,
  } from "../presentation/phaser-battlefield";

  let {
    view,
    actions,
    battlefield,
    previews,
    showCommanderArrow = false,
  }: {
    showCommanderArrow?: boolean;
    view: BattleView;
    previews: HudArtworkCache;
    actions: BattleCommands;
    battlefield: Promise<import("../presentation/phaser-battlefield").PreparedBattlefield>;
  } = $props();
  const mapIdentity = $derived(`${view.sessionVersion}:${view.game.mapId}`);
  let progress = $state.raw<BattlefieldProgress>();
  const loading = $derived(
    progress?.identity === mapIdentity
      ? progress
      : { phase: "loading" as const, loaded: 0, total: 0, audioLoaded: 0, audioTotal: 0 },
  );
  let host = $state.raw<HTMLDivElement>();
  let runtime = $state.raw<ReturnType<createPhaserBattlefield>>();
  let renderError = $state<string>();
  let moduleFailure = $state(false);
  let commanderHint = $state<number | null>(null);
  let description = $state("Battlefield loading");
  let reducedMotion = $state(false);
  let attempt = $state(0);

  onMount(() => {
    const preference = matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => (reducedMotion = preference.matches);
    update();
    preference.addEventListener("change", update);
    return () => preference.removeEventListener("change", update);
  });
  $effect(() => {
    attempt;
    let disposed = false;
    let active: ReturnType<createPhaserBattlefield> | undefined;
    void battlefield
      .then((module) => {
        if (disposed || renderError) return;
        moduleFailure = false;
        try {
          active = module.mount(
            host!,
            () => actions,
            (message) => {
              if (!disposed) renderError = message;
            },
            (label) => {
              if (!disposed) description = label;
            },
            previews,
            (state) => {
              if (!disposed) progress = state;
            },
          );
          runtime = active;
        } catch (error) {
          renderError = errorDetails(error);
        }
      })
      .catch((error) => {
        if (disposed) return;
        moduleFailure = true;
        renderError = errorDetails(error);
      });
    return () => {
      disposed = true;
      runtime = undefined;
      active?.game.destroy(true);
    };
  });
  $effect(() => {
    try {
      runtime?.scene.present({ view, reducedMotion });
    } catch (error) {
      renderError = errorDetails(error);
    }
  });
  $effect(() => {
    const node = host;
    if (!node) return;
    const hint = (event: Event) => {
      commanderHint = (event as CustomEvent<number | null>).detail;
    };
    node.addEventListener("battlefieldcommander", hint);
    return () => node.removeEventListener("battlefieldcommander", hint);
  });
  $effect(() => {
    const node = host?.closest("#map");
    if (!node) return;
    const handle = (event: Event) => {
      const { tile, kind } = (event as CustomEvent<{ tile: string; kind: MotionKind }>).detail;
      runtime?.scene.pulse(tile, kind);
    };
    node.addEventListener("battlefieldmotion", handle);
    return () => node.removeEventListener("battlefieldmotion", handle);
  });
</script>

<BattlefieldWindow
  bind:host
  phase={renderError ? "ready" : loading.phase}
  loaded={loading.loaded}
  total={loading.total}
  audioLoaded={loading.audioLoaded}
  audioTotal={loading.audioTotal}
>
  {#snippet navigation()}
    {#if showCommanderArrow && commanderHint !== null && loading.phase === "ready" && !renderError}
      <button
        class="commander-return"
        aria-label="Return to commander"
        title="Return to commander"
        onclick={() => runtime?.scene.returnToCommander()}
      >
        <svg
          viewBox="0 0 24 24"
          aria-hidden="true"
          style={`transform: rotate(${commanderHint}deg)`}
        >
          <path d="M12 20V4M5 11l7-7 7 7" />
        </svg>
      </button>
    {/if}
  {/snippet}
  {#if renderError}
    <div role="alert" class="map-render-error">
      <pre>{renderError}</pre>
      <button
        onclick={() => {
          if (moduleFailure) {
            window.location.reload();
            return;
          }
          renderError = undefined;
          attempt++;
        }}
      >
        {moduleFailure ? "Reload game" : "Retry map rendering"}
      </button>
    </div>
  {/if}
  {#if progress?.error}
    <div class="prefetch-error" role="alert">
      <p>Some artwork could not be prepared. The current battlefield remains available.</p>
      <details>
        <summary>Error details</summary>
        <pre>{progress.error}</pre>
      </details>
      <button onclick={() => runtime?.scene.retryResources()}>Retry artwork</button>
    </div>
  {/if}
</BattlefieldWindow>
<p id="battlefield-controls" class="status">
  Drag to pan. Pinch or Control or Command plus wheel to zoom. Keyboard: plus and minus to zoom,
  zero to reset zoom. Escape clears selection.
</p>
<p id="battlefield-description" class="status" aria-live="polite">{description}</p>

<style>
  .commander-return {
    position: absolute;
    right: 12px;
    bottom: 12px;
    z-index: 2;
    width: 44px;
    height: 44px;
    padding: 9px;
    border: 1px solid var(--gold);
    border-radius: 50%;
    background: var(--surface);
    color: var(--gold);
  }
  .commander-return svg {
    display: block;
    width: 100%;
    height: 100%;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .prefetch-error pre,
  .map-render-error pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .prefetch-error,
  .map-render-error {
    position: absolute;
    z-index: 3;
    inset: 12px 12px auto;
    max-height: calc(100% - 24px);
    overflow: auto;
    padding: 12px;
    background: var(--surface);
    border: 1px solid var(--line);
  }
</style>
