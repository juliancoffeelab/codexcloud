<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { CurrentEntry, TurnEntry } from "../save/metadata";
  import { runName, chapter, roman, saveTime } from "./save-browser";
  let {
    view,
    actions,
    onLoad = () => {},
  }: { view: BattleView; actions: BattleCommands; onLoad?: () => void } = $props();
  let entries = $state<TurnEntry[]>([]);
  let recovery = $state<CurrentEntry>();
  const sources = $derived(
    [...entries, ...(recovery ? [recovery] : [])].sort(
      (a, b) => b.savedAt - a.savedAt || a.id.localeCompare(b.id),
    ),
  );
  let failure = $state("");
  let busy = $state(false);
  let run = $state("");
  let selected = $state("");
  let limit = $state(30);
  let managing = $state(false);
  const runs = $derived([...new Map(sources.map((entry) => [entry.runId, entry])).values()]);
  const choices = $derived(
    runs.map((entry) => ({
      value: entry.runId,
      label: runName(entry),
      description: `Last saved ${saveTime(Math.max(...sources.filter((item) => item.runId === entry.runId).map((item) => item.savedAt)))}`,
    })),
  );
  const historical = $derived(entries.filter((entry) => entry.runId === run));
  const groups = $derived.by(() => {
    const result = new Map<string, { number: number; name: string; entries: TurnEntry[] }>();
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
    recovery = actions.savedRecovery();
    if (!sources.some((entry) => entry.runId === run))
      run = recovery?.runId ?? entries[0]?.runId ?? "";
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
  }
  async function perform(action: () => Promise<void>): Promise<boolean> {
    if (busy) return false;
    busy = true;
    failure = "";
    try {
      await action();
      refresh();
      return true;
    } catch {
      failure =
        "This operation could not be completed. Download a debug report if the problem continues.";
      return false;
    } finally {
      busy = false;
    }
  }
  function changeRun(id: string) {
    run = run === id ? "" : id;
    selected = "";
    limit = 30;
    managing = false;
  }
  async function remove(entry: TurnEntry) {
    if (confirm(`Delete turn ${entry.round} from ${chapter(entry).name}?`))
      await perform(() => actions.deleteSavedProgress(entry.id));
  }
  async function removeRun() {
    const current = recovery?.runId === run;
    const name = choices.find((choice) => choice.value === run)?.label ?? "this game";
    if (
      confirm(
        `${current ? "Delete all other saves" : "Delete all saves"} for ${name}?${current ? " Your current position will be kept." : ""}`,
      )
    ) {
      await perform(() => actions.deleteSavedRun(run));
      managing = false;
    }
  }
</script>

<section class="saved-games-browser" aria-label="Saved games" aria-busy={busy}>
  {#if failure}<p role="alert" class="save-alert">{failure}</p>
    <button onclick={() => actions.downloadDebugReport()}>Download debug report</button>{/if}
  {#if runs.length}
    {#each choices as game (game.value)}
      <div class="run-controls">
        <button
          class="run-toggle"
          aria-label={game.label}
          aria-expanded={run === game.value}
          aria-controls={`history-${game.value}`}
          onclick={() => changeRun(game.value)}
        >
          <span class="run-title">
            {game.label}
            <svg
              class:collapsed={run !== game.value}
              width="16"
              height="16"
              viewBox="0 0 16 16"
              aria-hidden="true"
            >
              <path d="m4 6 4 4 4-4" fill="none" stroke="currentColor" stroke-width="2" />
            </svg>
          </span>
          <span class="last-saved">{game.description}</span>
        </button>
        {#if run === game.value}<button
            class="manage-button"
            aria-expanded={managing}
            onclick={() => (managing = !managing)}
          >
            Manage
          </button>{/if}
      </div>
      <div id={`history-${game.value}`} hidden={run !== game.value}>
        {#if run === game.value}
          {#if managing}<div class="run-management">
              <button disabled={busy} onclick={() => perform(() => actions.pruneSavedRun(run))}>
                Prune older saves
              </button>
              <button class="delete-button" disabled={busy} onclick={removeRun}>
                {recovery?.runId === run ? "Delete other saves" : "Delete all saves"}
              </button>
            </div>{/if}
          <div class="save-history">
            {#each groups as group (`${group.number}:${group.name}`)}
              <section class="chapter-group" aria-label={group.name}>
                <h3>
                  {#if group.number}<span
                      class="chapter-number"
                      aria-label={`Chapter ${group.number}`}
                    >
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
                          {saveTime(entry.savedAt)}
                        </span>
                      </button>
                      {#if selected === entry.id}<div class="row-actions">
                          <button
                            class="load-button"
                            disabled={busy}
                            onclick={async () => {
                              if (await perform(() => actions.restoreProgress(entry.id))) onLoad();
                            }}
                          >
                            Load
                          </button>
                          <button
                            class="delete-button"
                            disabled={busy}
                            onclick={() => remove(entry)}
                          >
                            Delete
                          </button>
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
        {/if}
      </div>
    {/each}
  {:else if !failure}<p class="empty-history">No saved games.</p>{/if}
</section>

<style>
  .saved-games-browser {
    min-width: 0;
  }
  .run-controls {
    margin: 10px 0 0;
    padding-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
    border-bottom: 1px solid var(--line);
  }
  .run-toggle {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    padding: 2px 0;
    border: 0;
    background: none;
    text-align: left;
  }
  .run-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font:
      600 19px/1.2 Georgia,
      serif;
  }
  .run-title svg {
    flex-shrink: 0;
  }
  .run-title svg.collapsed {
    transform: rotate(-90deg);
  }
  .last-saved {
    font:
      12px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
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
    margin: 12px 0 4px;
    padding: 6px;
    border: 1px solid #665c3e;
    border-radius: 4px;
    background: #cbb66e0b;
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
    margin: 0 6px;
    min-height: 56px;
    border-bottom: 1px solid var(--line);
  }
  li.expanded {
    margin-inline: 0;
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
