<script lang="ts">
  import { flushSync, onMount } from "svelte";
  import type { createBattleApplication } from "./battle-app";

  let application = $state<ReturnType<typeof createBattleApplication>>();
  let failure = $state<string>();

  onMount(() => {
    let current = true;
    void import("./battle-app")
      .then((module) => {
        if (!current) return;
        const ready = module.createBattleApplication();
        try {
          flushSync(() => (application = ready));
        } catch (error) {
          ready.controller.dispose();
          application = undefined;
          throw error;
        }
        window.dispatchEvent(new Event("hex-command-ready"));
      })
      .catch((error: unknown) => {
        if (current)
          failure = error instanceof Error ? (error.stack ?? error.message) : String(error);
      });
    return () => {
      current = false;
    };
  });
</script>

{#if application}
  <application.component controller={application.controller} />
{:else}
  <main class="loading-shell" aria-busy={!failure}>
    <header class="battle-bar" aria-label="Battle controls">
      <div class="header-economy" aria-label="Battle statistics loading">
        <span class="treasury">
          <i class="coin" aria-hidden="true"></i>
          <b>—</b>
        </span>
        <span class="income">
          <b>—</b>
          <span>/turn</span>
        </span>
        <span class="villages" aria-label="Villages">—/—</span>
      </div>
      <button id="end-turn" class="primary" disabled>
        <span>End turn</span>
        <span class="round">Turn —</span>
      </button>
      <button class="icon-button" aria-label="Undo move" disabled>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M9 4 4 9l5 5M4 9h9a7 7 0 0 1 7 7v4" />
        </svg>
      </button>
      <button class="icon-button" aria-label="Settings" disabled>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="m9 3 1-1h4l1 3 3 1 3-1 2 4-2 2v3l2 2-2 4-3-1-3 1-1 3h-4l-1-3-3-1-3 1-2-4 2-2v-3L1 9l2-4 3 1 3-1z"
          />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>
    </header>
    <div class="workspace">
      <section class="map-section" aria-label="Battlefield loading">
        <div class="map-scroll loading-map">
          <div class="loading-message" role="status" aria-live="polite">
            {#if failure}
              <p>The game could not start. Reloading keeps your saved battle.</p>
              <button class="quiet" onclick={() => window.location.reload()}>Reload game</button>
              <details>
                <summary>Error details</summary>
                <pre>{failure}</pre>
              </details>
            {:else}
              <p>Loading the game…</p>
            {/if}
          </div>
        </div>
      </section>
    </div>
  </main>
{/if}

<style>
  .workspace {
    display: block;
  }
  .loading-map {
    height: min(720px, calc(100svh - 120px));
    display: grid;
    place-items: center;
  }
  .loading-message {
    max-width: 32rem;
    padding: 24px;
    color: var(--muted);
    font:
      14px/1.6 system-ui,
      sans-serif;
  }
  .loading-message button,
  .loading-message details {
    margin-top: 14px;
  }
  .loading-message pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  @media (max-width: 1000px) {
    .loading-map {
      height: 55svh;
    }
  }
</style>
