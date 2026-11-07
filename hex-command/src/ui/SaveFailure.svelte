<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { SaveFailure } from "../save/failure";
  let {
    view,
    actions,
    onManage,
  }: { view: BattleView; actions: BattleCommands; onManage: () => void } = $props();
  let dialog: HTMLDialogElement;
  let shown = $state.raw<SaveFailure | null>(null);
  const failure = $derived(view.saveFailure);
  const title = $derived(
    failure?.kind === "full"
      ? "Storage is full"
      : failure?.kind === "blocked"
        ? "Saving is unavailable"
        : "Progress couldn’t be saved",
  );
  $effect(() => {
    if (failure && failure !== shown) {
      shown = failure;
      dialog.showModal();
    } else if (!failure && dialog.open) dialog.close();
  });
  function manage() {
    dialog.close();
    onManage();
  }
</script>

<dialog class="save-failure-dialog" bind:this={dialog} aria-labelledby="save-failure-title">
  <h2 id="save-failure-title">{title}</h2>
  <p>
    {failure?.kind === "full"
      ? "Delete some older saves to make room for your progress."
      : failure?.kind === "blocked"
        ? "This browser is blocking access to saved games. Allow site storage in your browser settings."
        : "The game encountered an error while saving. Download a debug report so we can investigate."}
  </p>
  <p class="unsaved">Your latest progress hasn’t been fully saved.</p>
  {#if failure?.kind === "full"}
    <button class="primary" onclick={manage}>Manage saves</button>
    <button class="keep-playing" onclick={() => dialog.close()}>Keep playing</button>
  {:else if failure?.kind === "blocked"}
    <button class="primary" onclick={() => dialog.close()}>Close</button>
  {:else}
    <button class="primary" data-activation="release" onclick={() => actions.downloadDebugReport()}>
      Download debug report
    </button>
    <button class="quiet-close" onclick={() => dialog.close()}>Close</button>
  {/if}
</dialog>

<style>
  .save-failure-dialog {
    width: min(23rem, calc(100vw - 2rem));
    padding: 24px 22px;
    border-radius: 12px;
  }
  h2 {
    font:
      25px/1.2 Georgia,
      serif;
    margin: 0 0 12px;
  }
  p {
    font:
      14px/1.55 system-ui,
      sans-serif;
    color: var(--muted);
    margin: 0 0 18px;
  }
  .unsaved {
    font-size: 12px;
    color: #e1b7a6;
    margin-bottom: 24px;
  }
  button {
    display: block;
    width: 100%;
  }
  .keep-playing,
  .quiet-close {
    border: 0;
    background: none;
    margin-top: 7px;
  }
  .keep-playing {
    color: #d39185;
  }
</style>
