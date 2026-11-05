<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { ProgressEntry } from "../save/progress";
  import ChoicePicker from "./ChoicePicker.svelte";
  import { runName, chapter, roman, saveTime } from "./save-browser";
  let { view, actions }: { view: BattleView; actions: BattleCommands } = $props();
  let dialog: HTMLDialogElement;
  let entries = $state<ProgressEntry[]>([]);
  let failure = $state("");
  let run = $state("");
  let selected = $state("");
  let limit = $state(30);
  let managing = $state(false);
  const runs = $derived([...new Map(entries.map((entry) => [entry.runId, entry])).values()]);
  const choices = $derived(
    runs.map((entry) => ({
      value: entry.runId,
      label: runName(entry),
      description: `Last saved ${saveTime(Math.max(...entries.filter((item) => item.runId === entry.runId).map((item) => item.savedAt)))}`,
    })),
  );
  const historical = $derived(entries.filter((entry) => entry.runId === run && !entry.current));
  const groups = $derived.by(() => {
    const result = new Map<string, { number: number; name: string; entries: ProgressEntry[] }>();
    for (const entry of historical.slice(0, limit)) {
      const info = chapter(entry);
      const key = `${info.number}:${info.name}`;
      const group = result.get(key) ?? { ...info, entries: [] };
      group.entries.push(entry);
      result.set(key, group);
    }
    return [...result.values()];
  });
  function refresh() {
    entries = actions.savedProgress();
    if (!entries.some((entry) => entry.runId === run))
      run = entries.find((entry) => entry.current)?.runId ?? entries[0]?.runId ?? "";
    if (!entries.some((entry) => entry.id === selected)) selected = "";
  }
  export function open() {
    failure = "";
    limit = 30;
    managing = false;
    selected = "";
    try {
      refresh();
    } catch {
      failure =
        "Saved games could not be read. Download a debug report from Settings so we can investigate.";
    }
    dialog.showModal();
  }
  function perform(action: () => void): boolean {
    failure = "";
    try {
      action();
      refresh();
      return true;
    } catch {
      failure =
        "This operation could not be completed. Download a debug report if the problem continues.";
      return false;
    }
  }
  function changeRun() {
    selected = "";
    limit = 30;
    managing = false;
  }
  function remove(entry: ProgressEntry) {
    if (confirm(`Delete turn ${entry.round} from ${chapter(entry).name}?`))
      perform(() => actions.deleteSavedProgress(entry.id));
  }
  function removeRun() {
    const current = entries.some((entry) => entry.runId === run && entry.current);
    const name = choices.find((choice) => choice.value === run)?.label ?? "this game";
    if (
      confirm(
        `Delete saved progress for ${name}?${current ? " Your current position will be kept." : ""}`,
      )
    ) {
      perform(() => actions.deleteSavedRun(run));
      managing = false;
    }
  }
</script>

<dialog bind:this={dialog} class="saved-games-dialog" aria-labelledby="saved-games-heading">
  <header class="save-heading">
    <h2 id="saved-games-heading">Saved games</h2>
    <button class="icon-button" aria-label="Close saved games" onclick={() => dialog.close()}>
      ×
    </button>
  </header>
  {#if failure}<p role="alert" class="save-alert">{failure}</p>
    <button onclick={() => actions.downloadDebugReport()}>Download debug report</button>{/if}
  {#if runs.length}
    <div class="run-controls">
      <div class="run-picker">
        <ChoicePicker label="Saved game" {choices} bind:value={run} onchange={changeRun} />
        <p>{choices.find((choice) => choice.value === run)?.description}</p>
      </div>
      <button class="manage-button" aria-expanded={managing} onclick={() => (managing = !managing)}>
        Manage
      </button>
    </div>
    {#if managing}<div class="run-management">
        <button onclick={() => perform(() => actions.pruneSavedRun(run))}>Prune older saves</button>
        <button class="delete-button" onclick={removeRun}>Delete game saves</button>
      </div>{/if}
    <div class="save-history">
      {#each groups as group (`${group.number}:${group.name}`)}
        <section class="chapter-group" aria-label={group.name}>
          <h3>
            {#if group.number}<span class="chapter-number" aria-label={`Chapter ${group.number}`}>
                {roman(group.number)}
              </span>{/if}
            <span>{group.name}</span>
          </h3>
          <ul>
            {#each group.entries as entry (entry.id)}
              <li class:expanded={selected === entry.id} data-save-id={entry.id}>
                <button
                  class="save-row"
                  aria-expanded={selected === entry.id}
                  onclick={() => (selected = selected === entry.id ? "" : entry.id)}
                >
                  <span class="turn">Turn {entry.round}</span>
                  <span class="save-time">
                    {entry.kind !== "turn" ? "Saved position · " : ""}{saveTime(entry.savedAt)}
                  </span>
                </button>
                {#if selected === entry.id}<div class="row-actions">
                    <button
                      class="load-button"
                      onclick={() => {
                        if (perform(() => actions.restoreProgress(entry.id))) dialog.close();
                      }}
                    >
                      Load
                    </button>
                    <button class="delete-button" onclick={() => remove(entry)}>Delete</button>
                  </div>{/if}
              </li>
            {/each}
          </ul>
        </section>
      {:else}<p class="empty-history">No earlier saves.</p>{/each}
      {#if historical.length > limit}<button class="show-more" onclick={() => (limit += 30)}>
          Show more
        </button>{/if}
    </div>
  {:else if !failure}<p class="empty-history">No saved games.</p>{/if}
</dialog>

<style>
  .saved-games-dialog {
    width: min(30rem, calc(100vw - 1.25rem));
    max-width: none;
    max-height: 85dvh;
    padding: 0;
    overflow: auto;
  }
  .save-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 14px 12px;
    border-bottom: 1px solid var(--line);
  }
  .save-heading h2 {
    margin: 0;
    font:
      26px/1.1 Georgia,
      serif;
  }
  .run-controls {
    margin: 14px 12px 0;
    padding-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    border-bottom: 1px solid var(--line);
  }
  .run-picker {
    flex: 1;
    min-width: 0;
  }
  .run-picker :global(.choice-trigger) {
    padding: 0;
    border: 0;
    background: none;
    min-height: 32px;
    width: auto;
    max-width: 100%;
    font-weight: 600;
    gap: 8px;
  }
  .run-picker p {
    font:
      12px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
    margin: 2px 0 0;
  }
  .manage-button {
    min-height: 36px;
    padding: 6px 11px;
    font-size: 12px;
  }
  .run-management {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin: 10px 12px;
  }
  .run-management button {
    font-size: 12px;
    min-height: 36px;
    padding: 6px 10px;
  }
  .save-history {
    padding-bottom: 12px;
  }
  .chapter-group h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    padding: 13px 12px 4px;
    font:
      17px/1 Georgia,
      serif;
  }
  .chapter-number {
    display: grid;
    place-items: center;
    flex: 0 0 29px;
    width: 29px;
    height: 29px;
    border: 1px solid #75663e;
    border-radius: 3px;
    background: #cbb66e0b;
    color: var(--gold);
    font:
      600 17px/1 Georgia,
      serif;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 0 12px;
    min-height: 56px;
    border-bottom: 1px solid var(--line);
  }
  li.expanded {
    margin-inline: 4px;
    padding-inline: 8px;
    background: #c7b27212;
    border-color: #8d8050;
    border-radius: 6px;
  }
  .save-row {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    gap: 2px;
    min-height: 56px;
    padding: 7px 0;
    border: 0;
    border-radius: 0;
    background: transparent;
    text-align: left;
  }
  .save-row:hover {
    background: transparent;
  }
  .turn {
    font-size: 14px;
  }
  .expanded .turn {
    color: var(--gold);
  }
  .save-time {
    font-size: 11px;
    color: var(--muted);
    overflow-wrap: anywhere;
  }
  .row-actions {
    flex-shrink: 0;
    display: flex;
    gap: 7px;
  }
  .row-actions button {
    min-height: 32px;
    padding: 5px 10px;
    border-radius: 5px;
    font-size: 12px;
  }
  .load-button {
    background: var(--gold);
    border-color: var(--gold);
    color: var(--bg);
  }
  .delete-button {
    color: #e1a097;
    border-color: #946358;
    background: #b76f6315;
  }
  .empty-history,
  .save-alert {
    padding: 12px;
    margin: 0;
    font-size: 14px;
  }
  .save-alert {
    color: #e1a097;
  }
  .show-more {
    margin: 12px;
  }
</style>
