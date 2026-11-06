<script lang="ts">
  import { onDestroy } from "svelte";
  import DialogWindow from "./DialogWindow.svelte";
  import SaveIcon from "./SaveIcon.svelte";
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { CurrentEntry, TurnEntry } from "../save/metadata";
  import type { SaveSummary } from "../save/summary";
  import { runName, turnName, saveTime, saveOrder, savedGames } from "./save-browser";
  let {
    actions,
    view,
    onLoad = () => {},
  }: { actions: BattleCommands; view: BattleView; onLoad?: () => void } = $props();
  let browser: HTMLElement;
  let entries = $state<TurnEntry[]>([]);
  let recovery = $state<CurrentEntry>();
  let failure = $state("");
  let busy = $state(false);
  let expanded = $state<Record<string, boolean>>({});
  let limits = $state<Record<string, number>>({});
  let summaries = $state<Record<string, SaveSummary>>({});
  let summaryErrors = $state<Record<string, string>>({});
  let reading = $state<Record<string, boolean>>({});
  let generation = 0;
  const games = $derived(savedGames([...entries, ...(recovery ? [recovery] : [])]));
  const histories = $derived.by(() => {
    const result = new Map<string, TurnEntry[]>();
    for (const entry of entries) {
      const history = result.get(entry.runId) ?? [];
      history.push(entry);
      result.set(entry.runId, history);
    }
    for (const history of result.values()) history.sort(saveOrder);
    return result;
  });
  type Pending =
    | { kind: "load"; entry: TurnEntry }
    | { kind: "turn"; entry: TurnEntry }
    | {
        kind: "campaign";
        id: string;
        name: string;
        started: string;
        current: boolean;
        count: number;
      };
  let pending = $state<Pending>();
  let confirmation = $state<HTMLDialogElement>();
  let confirmationReady = $state(false);
  let armTimer: ReturnType<typeof setTimeout>;
  const confirmationTitle = $derived(
    pending?.kind === "load"
      ? "Load saved turn?"
      : pending?.kind === "turn"
        ? "Delete save?"
        : "Delete all campaign saves?",
  );
  const confirmationAction = $derived(
    pending?.kind === "load"
      ? "Load turn"
      : pending?.kind === "turn"
        ? "Delete save"
        : "Delete all campaign saves",
  );
  function firstSaved(entry: TurnEntry | CurrentEntry): string {
    return `${entry.kind === "turn" && entry.missionStart && entry.ordinal === 1 ? "Started" : "Earliest save"} ${saveTime(entry.savedAt)}`;
  }
  function refresh() {
    entries = actions.savedProgress();
    recovery = actions.savedRecovery();
  }
  export function open() {
    generation++;
    expanded = {};
    limits = {};
    summaries = {};
    summaryErrors = {};
    reading = {};
    failure = "";
    browser?.querySelectorAll<HTMLDetailsElement>("details").forEach((details) => {
      details.open = false;
    });
    try {
      refresh();
    } catch {
      failure =
        "Saved games could not be read. Download a debug report from Settings so we can investigate.";
    }
  }
  async function inspect(entry: TurnEntry) {
    expanded[entry.id] = !expanded[entry.id];
    if (!expanded[entry.id] || summaries[entry.id] || reading[entry.id]) return;
    const requestGeneration = generation;
    reading[entry.id] = true;
    delete summaryErrors[entry.id];
    try {
      const result = await actions.savedSummary(entry.id);
      if (requestGeneration === generation) summaries[entry.id] = result;
    } catch {
      if (requestGeneration === generation)
        summaryErrors[entry.id] = "This saved battle could not be read.";
    } finally {
      if (requestGeneration === generation) reading[entry.id] = false;
    }
  }
  function ask(action: Pending) {
    pending = action;
    failure = "";
    confirmationReady = false;
    clearTimeout(armTimer);
    // A rapid second tap must not confirm a dialog that has just appeared.
    armTimer = setTimeout(() => {
      confirmationReady = true;
    }, 350);
    confirmation?.showModal();
    confirmation?.querySelector<HTMLButtonElement>(".cancel")?.focus();
  }
  function removeRun(id: string) {
    const game = games.find((game) => game.id === id);
    if (!game) return;
    ask({
      kind: "campaign",
      id,
      name: runName(game.latest),
      started: firstSaved(game.earliest),
      current: view.activeRunId === id,
      count: (histories.get(id)?.length ?? 0) + (recovery?.runId === id ? 1 : 0),
    });
  }
  async function confirmAction() {
    if (!pending || busy || !confirmationReady) return;
    const action = pending;
    busy = true;
    failure = "";
    try {
      if (action.kind === "load") await actions.restoreProgress(action.entry.id);
      else if (action.kind === "turn") await actions.deleteSavedProgress(action.entry.id);
      else await actions.deleteSavedRun(action.id);
      refresh();
      confirmation?.close();
      if (action.kind === "load") onLoad();
    } catch {
      failure =
        "This operation could not be completed. Download a debug report if the problem continues.";
    } finally {
      busy = false;
    }
  }
  onDestroy(() => {
    generation++;
    clearTimeout(armTimer);
  });
</script>

<section class="saved-games-browser" bind:this={browser} aria-label="Saved games" aria-busy={busy}>
  {#if failure && !confirmation?.open}
    <p role="alert" class="save-alert">{failure}</p>
    <button onclick={() => actions.downloadDebugReport()}>Download debug report</button>
  {/if}
  {#if view.saveFailure?.kind === "full"}
    <p class="space-warning" role="status">
      <strong>Latest progress hasn’t been saved</strong>
      Delete campaign saves or individual turns to free space.
    </p>
  {/if}
  {#each games as game (game.id)}
    {@const history = histories.get(game.id) ?? []}
    <details class="campaign" data-run-id={game.id}>
      <summary class="campaign-toggle">
        <span class="campaign-copy">
          <span class="campaign-name">{runName(game.latest)}</span>
          <span class="campaign-time">
            {firstSaved(game.earliest)} · Last saved {saveTime(game.latest.savedAt)}
          </span>
        </span>
        <svg class="chevron" viewBox="0 0 16 16" aria-hidden="true"><path d="m6 3 5 5-5 5" /></svg>
      </summary>
      <div class="history">
        {#each history.slice(0, limits[game.id] ?? 30) as entry (entry.id)}
          {@const stats = summaries[entry.id]}
          <div class="save" class:expanded={expanded[entry.id]} data-save-id={entry.id}>
            <div class="save-row">
              <div class="save-copy">
                <span class="turn">{turnName(entry)}</span>
                <span class="save-time">
                  {saveTime(entry.savedAt)}{entry.manual ? " · Manual" : ""}
                </span>
              </div>
              <button
                class="row-icon load"
                disabled={busy}
                aria-label={`Load ${turnName(entry)}`}
                title="Load saved turn"
                onclick={() => ask({ kind: "load", entry })}
              >
                <SaveIcon name="load" />
              </button>
              <button
                class="row-icon inspect"
                disabled={busy}
                aria-label={`Details for ${turnName(entry)}`}
                aria-expanded={!!expanded[entry.id]}
                aria-controls={`summary-${entry.id}`}
                title="Saved battle details"
                onclick={() => inspect(entry)}
              >
                <SaveIcon name="details" />
              </button>
            </div>
            {#if expanded[entry.id]}
              <div
                class="save-summary"
                id={`summary-${entry.id}`}
                aria-label={`Battle summary for ${turnName(entry)}`}
              >
                {#if stats}
                  <dl class="summary-stats">
                    <div>
                      <dt><SaveIcon name="gold" />Gold</dt>
                      <dd class="gold">{stats.gold}</dd>
                    </div>
                    <div>
                      <dt><SaveIcon name="villages" />Villages</dt>
                      <dd>{stats.villages.owned} / {stats.villages.total}</dd>
                    </div>
                    <div>
                      <dt><SaveIcon name="objectives" />Objectives</dt>
                      <dd>
                        {stats.objectives
                          ? `${stats.objectives.completed} / ${stats.objectives.total}`
                          : "—"}
                      </dd>
                    </div>
                    {#each stats.units as units (units.level)}<div>
                        <dt><SaveIcon name="units" />Units lvl. {units.level}</dt>
                        <dd>{units.count}</dd>
                      </div>{/each}
                  </dl>
                {:else if summaryErrors[entry.id]}<p class="summary-error" role="alert">
                    {summaryErrors[entry.id]}
                  </p>
                {:else}<p class="reading" role="status">Reading saved battle…</p>{/if}
                <div class="summary-footer">
                  <button
                    class="delete-save"
                    disabled={busy}
                    onclick={() => ask({ kind: "turn", entry })}
                  >
                    <SaveIcon name="trash" />Delete save
                  </button>
                </div>
              </div>
            {/if}
          </div>
        {:else}<p class="empty-history">No earlier saves.</p>{/each}
        {#if history.length > (limits[game.id] ?? 30)}<button
            class="show-more"
            onclick={() => {
              limits[game.id] = (limits[game.id] ?? 30) + 30;
            }}
          >
            Show more
          </button>{/if}
        <div class="campaign-management">
          <button class="delete-campaign" disabled={busy} onclick={() => removeRun(game.id)}>
            <SaveIcon name="trash" />Delete all campaign saves
          </button>
        </div>
      </div>
    </details>
  {:else}{#if !failure}<p class="empty-history">No saved games.</p>{/if}{/each}
</section>
<DialogWindow title={confirmationTitle} bind:dialog={confirmation} width="380px" canDismiss={!busy}>
  {#if pending}
    {#if pending.kind === "campaign"}
      <p class="confirmation-name">{pending.name}</p>
      <p class="confirmation-time">{pending.started}</p>
      {#if pending.current}<div class="current-warning">
          <strong>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="m12 3 10 18H2L12 3zM12 9v5M12 17v1" />
            </svg>
            <span>This is the game you’re currently playing.</span>
          </strong>
          <p>Deleting its saves will end this game and open New game.</p>
        </div>{/if}
      <p class="confirmation-copy">
        All {pending.count} saves will be deleted, including chapter starts. This cannot be undone.
      </p>
    {:else}
      <p class="confirmation-name">{runName(pending.entry)}</p>
      <p class="confirmation-turn">{turnName(pending.entry)}</p>
      <p class="confirmation-time">{saveTime(pending.entry.savedAt)}</p>
      <p class="confirmation-copy">
        {pending.kind === "load"
          ? "Your current position will be replaced."
          : "This save will be deleted. This cannot be undone."}
      </p>
    {/if}
    {#if failure}<p class="save-alert" role="alert">{failure}</p>{/if}
    <button
      class="confirm-action"
      class:destructive={pending.kind !== "load"}
      disabled={busy || !confirmationReady}
      onclick={confirmAction}
    >
      {confirmationAction}
    </button>
    <button class="cancel" disabled={busy} onclick={() => confirmation?.close()}>Cancel</button>
  {/if}
</DialogWindow>

<style>
  .saved-games-browser {
    min-width: 0;
  }
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
  .campaign {
    margin: 0;
    padding: 0;
    border: 0;
  }
  .campaign + .campaign {
    border-top: 1px solid var(--line);
  }
  .campaign-toggle {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 88px;
    padding: 20px 0 18px;
    cursor: pointer;
    list-style: none;
  }
  .campaign-toggle::-webkit-details-marker {
    display: none;
  }
  .campaign-copy {
    flex: 1;
    min-width: 0;
  }
  .campaign-name {
    display: block;
    font:
      600 20px/1.2 Georgia,
      serif;
  }
  .campaign-time {
    display: block;
    color: var(--muted);
    font:
      10px/1.45 system-ui,
      sans-serif;
    margin-top: 6px;
  }
  .chevron {
    width: 17px;
    height: 17px;
    flex-shrink: 0;
    fill: none;
    stroke: #b8bea8;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .campaign[open] > summary .chevron {
    transform: rotate(90deg);
  }
  .history {
    padding-bottom: 13px;
  }
  .save-row {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: 74px;
    border-top: 1px solid var(--line);
  }
  .save-copy {
    flex: 1;
    min-width: 0;
    padding: 13px 0;
  }
  .turn {
    display: block;
    color: #e8e7da;
    font:
      15px/1.4 Georgia,
      serif;
  }
  .save-time {
    display: block;
    font:
      11px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
    margin-top: 5px;
    overflow-wrap: anywhere;
  }
  .row-icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    flex: 0 0 40px;
    padding: 8px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    --save-icon-size: 18px;
  }
  .load {
    color: #d9c07d;
  }
  .inspect {
    color: #b1b9a3;
  }
  .inspect[aria-expanded="true"] {
    background: #cbb66e12;
    color: #dfcb91;
  }
  .expanded .turn {
    color: #ede4c7;
  }
  .save-summary {
    background: #cbb66e06;
    padding: 14px 13px 4px;
    margin-bottom: 16px;
  }
  .summary-stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 21px 16px;
    margin: 0;
  }
  dt {
    display: flex;
    align-items: center;
    gap: 5px;
    color: #aab29f;
    font:
      11px/1.4 system-ui,
      sans-serif;
  }
  dd {
    margin: 5px 0 0;
    padding-left: 19px;
    color: #e9e7d6;
    font:
      500 20px/1.2 Georgia,
      serif;
  }
  dd.gold {
    color: #e1c67e;
  }
  .summary-footer {
    display: flex;
    justify-content: flex-end;
    margin-top: 13px;
    padding-top: 2px;
  }
  .delete-save,
  .delete-campaign {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    min-height: 36px;
    padding: 5px 0;
    border: 0;
    background: none;
    color: #cb9588;
    font:
      12px/1.4 system-ui,
      sans-serif;
  }
  .delete-save {
    font-size: 11px;
  }
  .campaign-management {
    margin-top: 16px;
    padding: 14px 0 0;
  }
  .show-more {
    margin: 12px 0;
  }
  .reading,
  .summary-error,
  .empty-history,
  .save-alert {
    font:
      13px/1.5 system-ui,
      sans-serif;
    margin: 12px 0;
  }
  .reading {
    min-height: 90px;
  }
  .save-alert,
  .summary-error {
    color: #e1a097;
  }
  .confirmation-name {
    font:
      20px/1.3 Georgia,
      serif;
    margin: 18px 0 5px;
  }
  .confirmation-turn {
    margin: 8px 0 4px;
    font:
      15px/1.4 Georgia,
      serif;
  }
  .confirmation-time {
    font:
      11px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
    margin: 0;
  }
  .confirmation-copy {
    font:
      13px/1.55 system-ui,
      sans-serif;
    margin: 20px 0;
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
    width: 17px;
    height: 17px;
    flex: 0 0 17px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
  }
  .current-warning p {
    font:
      12px/1.5 system-ui,
      sans-serif;
    color: #e3b1a6;
    margin: 8px 0 0;
  }
  .confirm-action,
  .cancel {
    display: block;
    width: 100%;
    min-height: 43px;
    border-radius: 4px;
    font-size: 13px;
  }
  .confirm-action {
    color: var(--bg);
    background: var(--gold);
    border: 1px solid var(--gold);
  }
  .confirm-action.destructive {
    background: #ad5445;
    border: 1px solid #c87464;
    color: #fff1e9;
    font-weight: 600;
  }
  .cancel {
    margin-top: 9px;
    background: none;
    border: 1px solid #5b654f;
  }
  @media (max-width: 360px) {
    .summary-stats {
      column-gap: 10px;
    }
    dt {
      font-size: 10px;
      gap: 4px;
    }
    dd {
      padding-left: 18px;
    }
  }
</style>
