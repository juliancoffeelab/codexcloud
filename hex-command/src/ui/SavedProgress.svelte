<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { ProgressEntry } from "../save/progress";
  let { view, actions }: { view: BattleView; actions: BattleCommands } = $props();
  let dialog: HTMLDialogElement;
  let entries = $state<ProgressEntry[]>([]);
  let failure = $state("");
  let run = $state("");
  let limit = $state(30);
  const runs = $derived([...new Map(entries.map((entry) => [entry.runId, entry])).values()]);
  const filtered = $derived(entries.filter((entry) => !run || entry.runId === run));
  function refresh() {
    try {
      entries = actions.savedProgress();
    } catch (error) {
      failure = String(error);
    }
  }
  export function open() {
    failure = "";
    limit = 30;
    refresh();
    dialog.showModal();
  }
  function perform(action: () => void) {
    failure = "";
    try {
      action();
      refresh();
    } catch (error) {
      failure = error instanceof Error ? (error.stack ?? error.message) : String(error);
    }
  }
</script>

<dialog bind:this={dialog} class="saved-progress-dialog" aria-labelledby="saved-progress-heading">
  <div class="settings-heading">
    <h2 id="saved-progress-heading">Saved progress</h2>
    <button class="icon-button" aria-label="Close saved progress" onclick={() => dialog.close()}>
      ×
    </button>
  </div>
  <p role="status">{view.persistence}</p>
  {#if view.saveError}<p role="alert">{view.saveError}</p>{/if}
  {#if failure}<p role="alert">{failure}</p>{/if}
  <p>
    Progress is stored in this browser and works offline. Clearing browser data removes it. Cloud
    sync is postponed.
  </p>
  <p>
    Turn starts form your history. Each continuation also keeps its latest recovery with available
    move undo. Loading a save keeps your previous continuation.
  </p>
  <button onclick={() => perform(() => actions.retrySave())}>Retry saving current play</button>
  <label>
    Run
    <select bind:value={run} onchange={() => (limit = 30)}>
      <option value="">All runs</option>
      {#each runs as entry}<option value={entry.runId}>
          {entry.run} · {entry.runId.slice(0, 8)}
        </option>{/each}
    </select>
  </label>
  {#if run}
    <div class="save-controls">
      <button onclick={() => perform(() => actions.pruneSavedRun(run))}>
        Thin older turn history
      </button>
      <button
        onclick={() => {
          if (
            confirm(
              "Delete this run's saved progress? The currently played checkpoint and its recovery will be kept.",
            )
          )
            perform(() => actions.deleteSavedRun(run));
        }}
      >
        Delete run saves
      </button>
    </div>
  {/if}
  <p>
    Recent turns are kept densely; older turns are thinned progressively. Automatic cleanup
    preserves mission starts and your currently played checkpoint.
  </p>
  <ol class="save-list">
    {#each filtered.slice(0, limit) as entry (entry.id)}
      <li>
        <strong>{entry.run} · {entry.runId.slice(0, 8)}</strong>
        <span>{entry.mission} · Turn {entry.round} · {entry.side}</span>
        <span>
          {entry.kind === "recovery"
            ? "Continuation recovery"
            : entry.missionStart
              ? "Mission start"
              : "Turn start"} · Continuation {entry.branchId.slice(0, 8)}
        </span>
        <time datetime={new Date(entry.savedAt).toISOString()}>
          {new Date(entry.savedAt).toLocaleString()}
        </time>
        {#if entry.current}<strong>Currently playing</strong>{/if}
        <div class="save-controls">
          <button
            disabled={entry.current}
            onclick={() => {
              perform(() => actions.restoreProgress(entry.id));
              if (!failure) dialog.close();
            }}
          >
            Load
          </button>
          <button
            disabled={entry.protected}
            onclick={() => {
              if (confirm("Delete this saved checkpoint?"))
                perform(() => actions.deleteSavedProgress(entry.id));
            }}
          >
            Delete
          </button>
          {#if entry.protected}<span>Current checkpoint protected</span>{/if}
        </div>
      </li>
    {:else}<li>No saved progress yet.</li>{/each}
  </ol>
  {#if filtered.length > limit}<button onclick={() => (limit += 30)}>Show more</button>{/if}
</dialog>

<style>
  .saved-progress-dialog {
    width: min(46rem, calc(100vw - 2rem));
    max-height: 85vh;
    overflow: auto;
  }
  .save-list {
    padding-left: 1.5rem;
  }
  .save-list li {
    padding: 0.8rem 0;
    border-bottom: 1px solid #65705a;
  }
  .save-list span,
  .save-list time {
    display: block;
  }
  .save-controls {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  label {
    display: block;
    margin-top: 1rem;
  }
</style>
