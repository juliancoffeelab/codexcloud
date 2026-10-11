<script lang="ts">
  import { flushSync, onMount } from "svelte";
  import type { createBattleApplication } from "./battle-app";
  import BattleBar from "./ui/BattleBar.svelte";

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
  <main class="battle-layout loading-shell" aria-busy={!failure}>
    <BattleBar />
    <div class="battle-content">
      <div class="workspace map-only">
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
    </div>
  </main>
{/if}

<style>
  .loading-map {
    height: calc(100dvh - 120px);
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
</style>
