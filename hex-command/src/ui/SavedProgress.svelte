<script lang="ts">
  import DialogWindow from "./DialogWindow.svelte";
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { CurrentEntry, TurnEntry } from "../save/metadata";
  import {
    runName,
    chapter,
    roman,
    saveTime,
    saveOrder,
    savedGames,
    chapterGroups,
  } from "./save-browser";
  let {
    actions,
    view,
    onLoad = () => {},
    run = $bindable(""),
    onNavigate = () => {},
  }: {
    actions: BattleCommands;
    view: BattleView;
    onLoad?: () => void;
    run?: string;
    onNavigate?: (inGame: boolean) => void;
  } = $props();
  let entries = $state<TurnEntry[]>([]);
  let recovery = $state<CurrentEntry>();
  const games = $derived(savedGames([...entries, ...(recovery ? [recovery] : [])]));
  const game = $derived(games.find((game) => game.id === run));
  let failure = $state("");
  let busy = $state(false);
  let selected = $state("");
  let limit = $state(30);
  let confirmation = $state<HTMLDialogElement>();
  let deletion = $state<{
    id: string;
    name: string;
    started: string;
    current: boolean;
    count: number;
  }>();
  const historical = $derived(entries.filter((entry) => entry.runId === run).sort(saveOrder));
  const groups = $derived(chapterGroups(historical.slice(0, limit)));
  function firstSaved(entry: TurnEntry | CurrentEntry): string {
    return `${entry.kind === "turn" && entry.missionStart && entry.ordinal === 1 ? "Started" : "Earliest save"} ${saveTime(entry.savedAt)}`;
  }
  function refresh() {
    entries = actions.savedProgress();
    recovery = actions.savedRecovery();
    if (run && !games.some((game) => game.id === run)) changeRun("");
    if (!entries.some((entry) => entry.id === selected)) selected = "";
  }
  export function open() {
    run = "";
    failure = "";
    limit = 30;
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
    run = id;
    selected = "";
    limit = 30;
    onNavigate(!!run);
  }
  export function back() {
    changeRun("");
  }
  async function remove(entry: TurnEntry) {
    if (confirm(`Delete turn ${entry.round} from ${chapter(entry).name}?`))
      await perform(() => actions.deleteSavedProgress(entry.id));
  }
  function removeRun() {
    if (!game) return;
    deletion = {
      id: run,
      name: runName(game.latest),
      started: firstSaved(game.earliest),
      current: view.activeRunId === run,
      count: historical.length + (recovery?.runId === run ? 1 : 0),
    };
    confirmation?.showModal();
    confirmation?.querySelector<HTMLButtonElement>(".cancel-delete")?.focus();
  }
  async function confirmDelete() {
    if (!deletion) return;
    if (await perform(() => actions.deleteSavedRun(deletion!.id))) confirmation?.close();
  }
</script>

<section class="saved-games-browser" aria-label="Saved games" aria-busy={busy}>
  {#if failure}<p role="alert" class="save-alert">{failure}</p>
    <button onclick={() => actions.downloadDebugReport()}>Download debug report</button>{/if}
  {#if view.saveFailure?.kind === "full"}
    <p class="space-warning" role="status">
      <strong>Latest progress hasn’t been saved</strong>
      Delete campaign saves or individual turns to free space.
    </p>
  {/if}
  {#if game}
    <div class="run-controls">
      <div class="run-copy">
        <h3 class="run-title">{runName(game.latest)}</h3>
        <p class="first-saved">{firstSaved(game.earliest)}</p>
      </div>
    </div>
    <div class="save-history">
      {#each groups as group (group.entries[0]!.id)}
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
                  <span class="turn">
                    Turn {entry.round}{#if entry.manual}<span class="manual-label">
                        · Manual save
                      </span>{/if}
                  </span>
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
                    <button class="delete-button" disabled={busy} onclick={() => remove(entry)}>
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
    <button class="delete-campaign" disabled={busy} onclick={removeRun}>
      Delete campaign saves
    </button>
  {:else if games.length}
    <div class="game-list">
      {#each games as game (game.id)}
        <button class="game-choice" data-run-id={game.id} onclick={() => changeRun(game.id)}>
          <span class="run-copy">
            <span class="run-title">{runName(game.latest)}</span>
            <span class="first-saved">{firstSaved(game.earliest)}</span>
            <span class="last-saved">Last saved {saveTime(game.latest.savedAt)}</span>
          </span>
          <svg class="chevron" viewBox="0 0 16 16" aria-hidden="true">
            <path d="m6 3 5 5-5 5" />
          </svg>
        </button>
      {/each}
    </div>
  {:else if !failure}<p class="empty-history">No saved games.</p>{/if}
</section>
<DialogWindow
  title="Delete campaign saves?"
  bind:dialog={confirmation}
  width="380px"
  canDismiss={!busy}
>
  {#if deletion}
    <p class="delete-name">{deletion.name}</p>
    <p class="first-saved">{deletion.started}</p>
    {#if deletion.current}
      <div class="current-warning">
        <strong>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="m12 3 10 18H2L12 3zM12 9v5M12 17v1" />
          </svg>
          <span>This is the game you’re currently playing.</span>
        </strong>
        <p>Deleting its saves will end this game and open New game.</p>
      </div>
    {/if}
    <p class="delete-explanation">
      All {deletion.count} saves will be deleted, including chapter starts. This cannot be undone.
    </p>
    {#if failure}<p class="save-alert" role="alert">{failure}</p>{/if}
    <button class="confirm-delete" disabled={busy} onclick={confirmDelete}>
      Delete campaign saves
    </button>
    <button class="cancel-delete" disabled={busy} onclick={() => confirmation?.close()}>
      Cancel
    </button>
  {/if}
</DialogWindow>

<style>
  .space-warning {
    margin: 14px 0 0;
    padding: 11px 12px;
    border: 1px solid #8a7742;
    border-radius: 4px;
    background: #b9923410;
    color: #dfcc91;
    font:
      12px/1.45 system-ui,
      sans-serif;
  }
  .space-warning strong {
    display: block;
    margin-bottom: 3px;
  }
  .delete-campaign {
    display: block;
    width: 100%;
    min-height: 44px;
    margin-top: 24px;
    border: 1px solid #9f6256;
    border-radius: 4px;
    background: #bb645314;
    color: #e4a49a;
    font-size: 13px;
  }
  .manual-label {
    font-size: 11px;
    color: var(--muted);
  }
  .delete-name {
    font:
      20px/1.3 Georgia,
      serif;
    margin: 18px 0 5px;
  }
  .current-warning {
    margin: 20px 0;
    border: 1px solid #ac6658;
    border-left: 3px solid #d58372;
    background: #b8534117;
    border-radius: 4px;
    padding: 12px;
  }
  .current-warning strong {
    display: flex;
    gap: 8px;
    color: #f2afa1;
    font:
      600 13px/1.45 system-ui,
      sans-serif;
  }
  .current-warning svg {
    flex: 0 0 17px;
    width: 17px;
    height: 17px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    margin-top: 1px;
  }
  .current-warning p {
    font:
      12px/1.5 system-ui,
      sans-serif;
    color: #e3b1a6;
    margin: 8px 0 0;
  }
  .delete-explanation {
    font:
      13px/1.55 system-ui,
      sans-serif;
    margin: 20px 0;
  }
  .confirm-delete,
  .cancel-delete {
    display: block;
    width: 100%;
    min-height: 43px;
    border-radius: 4px;
    font-size: 13px;
  }
  .confirm-delete {
    background: #ad5445;
    border: 1px solid #c87464;
    color: #fff1e9;
    font-weight: 600;
  }
  .cancel-delete {
    margin-top: 9px;
    background: none;
    border: 1px solid #5b654f;
  }
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
  .run-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .run-title {
    margin: 0;
    font:
      600 19px/1.2 Georgia,
      serif;
  }
  .first-saved,
  .last-saved {
    margin: 0;
    font:
      11px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
  }
  .last-saved {
    color: #cbbd96;
  }
  .game-choice {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 18px 0;
    border: 0;
    border-bottom: 1px solid var(--line);
    border-radius: 0;
    background: none;
    text-align: left;
  }
  .game-choice .run-title {
    font-weight: 400;
  }
  .chevron {
    width: 17px;
    height: 17px;
    fill: none;
    stroke: var(--muted);
    stroke-width: 1.6;
    flex-shrink: 0;
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
