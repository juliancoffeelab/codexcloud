<script lang="ts">
  import { flushSync, onMount } from "svelte";
  import type { ApplicationBoot } from "./boot";
  import { errorDetails } from "./error-details";
  import { installPressActivation } from "./ui/press";
  import BattleBar from "./ui/BattleBar.svelte";
  import BattlefieldWindow from "./ui/BattlefieldWindow.svelte";

  let { boot }: { boot: ApplicationBoot } = $props();
  let application = $state<Awaited<ApplicationBoot["ready"]>>();
  let failure = $state<string>();

  onMount(() => {
    const detachPress = installPressActivation(document);
    let current = true;
    void boot.ready
      .then((ready) => {
        if (!current) return;
        try {
          flushSync(() => (application = ready));
        } catch (error) {
          boot.dispose();
          application = undefined;
          throw error;
        }
        window.dispatchEvent(new Event("hex-command-ready"));
      })
      .catch((error: unknown) => {
        if (current) failure = errorDetails(error);
      });
    return () => {
      detachPress();
      current = false;
      boot.dispose();
    };
  });
</script>

{#if application}
  <application.component
    controller={application.controller}
    battlefield={application.battlefield}
    previews={application.previews}
  />
{:else}
  <main class="battle-layout loading-shell" aria-busy={!failure}>
    <BattleBar />
    <div class="battle-content">
      <div class="workspace map-only">
        <BattlefieldWindow phase={failure ? "ready" : "loading"}>
          {#if failure}
            <div class="startup-failure" role="alert">
              <p>The game could not start. Reloading keeps your saved battle.</p>
              <button
                data-activation="release"
                class="quiet"
                onclick={() => window.location.reload()}
              >
                Reload game
              </button>
              <details>
                <summary>Error details</summary>
                <pre>{failure}</pre>
              </details>
            </div>
          {/if}
        </BattlefieldWindow>
      </div>
    </div>
  </main>
{/if}

<style>
  .startup-failure {
    position: absolute;
    inset: 24px;
    overflow: auto;
    color: var(--muted);
    font:
      14px/1.6 system-ui,
      sans-serif;
  }
  .startup-failure pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
