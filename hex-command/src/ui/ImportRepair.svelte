<script lang="ts">
  import { onDestroy } from "svelte";
  import DialogWindow from "./DialogWindow.svelte";
  import SaveIcon from "./SaveIcon.svelte";
  import { runName } from "./save-browser";
  import type { BattleView, BattleCommands } from "../controller/view";
  import type {
    ImportFailure,
    RepairInventory,
    RepairRecord,
    RepairTarget,
  } from "../save/repair-types";
  let {
    view,
    actions,
    onNewGame,
  }: { view: BattleView; actions: BattleCommands; onNewGame: () => void } = $props();
  let dialog = $state<HTMLDialogElement>();
  let confirmation = $state<HTMLDialogElement>();
  let shown = $state.raw<ImportFailure | null>(null);
  let reviewing = $state(false);
  let inventory = $state<RepairInventory>({ records: [], errors: [], authoritative: false });
  let busy = $state(false);
  let error = $state("");
  let pending = $state<{ target: RepairTarget; label: string; copy: string }>();
  let armed = $state(false);
  let timer: ReturnType<typeof setTimeout>;
  const unknownFlash = $derived(
    inventory.records.find((record) => record.kind === "flash" && !record.runId),
  );
  const campaigns = $derived.by(() => {
    const groups = new Map<string, { name: string; records: RepairRecord[] }>();
    for (const record of inventory.records)
      if (record.runId) {
        const group = groups.get(record.runId) ?? {
          name: record.entry ? runName(record.entry) : (record.runName ?? record.runId),
          records: [],
        };
        group.records.push(record);
        groups.set(record.runId, group);
      }
    return [...groups].map(([id, group]) => ({ id, ...group }));
  });
  const unknown = $derived(inventory.records.filter((record) => !record.runId));
  $effect(() => {
    const failure = view.importFailure;
    if (!failure) {
      confirmation?.close();
      dialog?.close();
      shown = null;
    } else if (failure !== shown) {
      shown = failure;
      if (!dialog?.open) {
        reviewing = false;
        dialog?.showModal();
      }
    }
  });
  async function refresh() {
    inventory = await actions.repairInventory();
  }
  export async function open() {
    confirmation?.close();
    pending = undefined;
    error = "";
    reviewing = true;
    dialog?.showModal();
    busy = true;
    try {
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  function ask(target: RepairTarget, label: string, copy: string) {
    pending = { target, label, copy };
    armed = false;
    clearTimeout(timer);
    timer = setTimeout(() => (armed = true), 350);
    confirmation?.showModal();
    confirmation?.querySelector<HTMLButtonElement>(".cancel")?.focus();
  }
  async function remove() {
    if (!pending || !armed || busy) return;
    busy = true;
    error = "";
    try {
      await actions.repairSavedData(pending.target);
      confirmation?.close();
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  async function retry() {
    busy = true;
    error = "";
    try {
      if (!(await actions.retryImport())) {
        await refresh();
        error = "Import is still blocked. Review the remaining records and error details.";
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  function play() {
    dialog?.close();
    onNewGame();
  }
  onDestroy(() => clearTimeout(timer));
</script>

{#snippet row(record: RepairRecord)}
  <div class="record" data-repair-key={record.key}>
    <div class="record-copy">
      <strong>{record.label}</strong>
      {#if record.error}<p class="damage">{record.error}</p>{:else}<p>Readable saved data</p>{/if}
      <details class="key">
        <summary>Storage record</summary>
        <code>{record.key}</code>
      </details>
    </div>
    {#if record.kind === "index"}
      <button
        class="quiet"
        disabled={busy}
        onclick={() =>
          ask(
            { kind: "index" },
            "Rebuild history list?",
            "Rebuild the list from identifiable snapshots. Unidentified records remain available for manual cleanup.",
          )}
      >
        Rebuild history list
      </button>
    {:else}
      <button
        class="delete"
        disabled={busy}
        aria-label={`Delete ${record.label}`}
        onclick={() =>
          ask(
            { kind: "record", key: record.key },
            `Delete ${record.label}?`,
            "This source record will be deleted. Other records will remain. This cannot be undone.",
          )}
      >
        <SaveIcon name="trash" />
      </button>
    {/if}
  </div>
{/snippet}
<DialogWindow
  title="Saved data needs repair"
  bind:dialog
  width="540px"
  tall={reviewing}
  canDismiss={!busy}
>
  <p>
    Existing saves could not be imported. They have been retained. Saving is paused until the data
    is repaired or explicitly deleted.
  </p>
  {#if !reviewing}
    <button class="primary" onclick={() => open()}>Review saved data</button>
  {:else}
    {#if inventory.authoritative}<p>
        Another tab completed the import. Import the saved game here to continue.
      </p>{/if}
    {#each inventory.errors as issue}<p class="damage" role="alert">{issue}</p>{/each}
    {#if unknownFlash}<p class="warning">
        <strong>Flash save ownership is unknown.</strong>
         Campaign deletion will leave it untouched. We strongly recommend deleting this unidentified flash
        save before continuing.
      </p>{/if}
    {#each campaigns as campaign (campaign.id)}
      <details class="campaign" open data-repair-run={campaign.id}>
        <summary>{campaign.name}</summary>
        {#each campaign.records as record (record.key)}{@render row(record)}{/each}
        <button
          class="delete-campaign"
          disabled={busy}
          onclick={() =>
            ask(
              { kind: "campaign", runId: campaign.id },
              "Delete campaign source saves?",
              `Delete all source records known to belong to ${campaign.name}, including its flash save when ownership is known. ${unknownFlash ? "The unidentified flash save will remain; we strongly recommend deleting it separately. " : ""}This cannot be undone.`,
            )}
        >
          <SaveIcon name="trash" />Delete campaign source saves
        </button>
      </details>
    {/each}
    {#if unknown.length}<h3>Other source records</h3>
      {#each unknown as record (record.key)}{@render row(record)}{/each}{/if}
    {#if !inventory.records.length && !inventory.errors.length && !inventory.authoritative}<p>
        No source saves remain. You can finish import and choose a new game.
      </p>{/if}
    <button class="primary" disabled={busy} onclick={retry}>Import remaining saves</button>
    <button
      class="delete-all"
      disabled={busy || inventory.authoritative}
      onclick={() =>
        ask(
          { kind: "all" },
          "Delete all source saves?",
          "All retained source saves, including unidentified records and the flash save, will be deleted. This cannot be undone.",
        )}
    >
      <SaveIcon name="trash" />Delete all source saves
    </button>
  {/if}
  {#if error}<p class="damage" role="alert">{error}</p>{/if}
  <button class="play" disabled={busy} onclick={play}>Choose a new game without saving</button>
  <button class="debug" onclick={() => actions.downloadDebugReport()}>Download debug report</button>
  <details class="failure">
    <summary>Import error details</summary>
    <pre>{view.importFailure?.detail}</pre>
  </details>
</DialogWindow>
<DialogWindow
  title={pending?.label ?? "Confirm source deletion"}
  bind:dialog={confirmation}
  width="400px"
  canDismiss={!busy}
>
  <p>{pending?.copy}</p>
  {#if error}<p class="damage" role="alert">{error}</p>{/if}
  <button class="confirm-delete" disabled={busy || !armed} onclick={remove}>
    {pending?.target.kind === "index" ? "Rebuild history list" : "Delete source data"}
  </button>
  <button class="cancel" disabled={busy} onclick={() => confirmation?.close()}>Cancel</button>
</DialogWindow>

<style>
  p {
    font:
      14px/1.5 system-ui,
      sans-serif;
    color: var(--muted);
    overflow-wrap: anywhere;
  }
  .damage {
    color: #e6a698;
  }
  .warning {
    padding: 12px;
    border: 1px solid #bc8650;
    color: #e1c492;
  }
  .warning strong {
    display: block;
  }
  .campaign {
    border-top: 1px solid var(--line);
    padding: 16px 0;
  }
  .campaign > summary {
    font:
      600 20px/1.4 Georgia,
      serif;
    cursor: pointer;
  }
  .record {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 0;
    border-bottom: 1px solid var(--line);
  }
  .record-copy {
    flex: 1;
    min-width: 0;
  }
  .record strong {
    font:
      15px/1.4 system-ui,
      sans-serif;
  }
  .record p {
    font-size: 12px;
    margin: 5px 0;
  }
  .key {
    font:
      11px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
  }
  code {
    overflow-wrap: anywhere;
  }
  .delete {
    padding: 8px;
    border: 0;
    color: #d09686;
    background: none;
  }
  .delete-campaign,
  .delete-all {
    display: flex;
    align-items: center;
    gap: 7px;
    color: #dc9c8b;
    background: none;
    border: 0;
    margin: 12px 0;
    padding: 8px 0;
  }
  .primary,
  .play,
  .debug,
  .confirm-delete,
  .cancel {
    display: block;
    width: 100%;
    margin: 12px 0;
    min-height: 42px;
  }
  .primary {
    background: var(--gold);
    color: var(--bg);
  }
  .play {
    color: #db9888;
    border: 0;
    background: none;
  }
  .debug {
    background: none;
  }
  .confirm-delete {
    background: #a64e40;
    color: #fff2eb;
    border-color: #c07763;
  }
  .failure {
    margin: 16px 0;
    font:
      12px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .quiet {
    font-size: 12px;
  }
</style>
