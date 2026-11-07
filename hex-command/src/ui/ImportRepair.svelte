<script lang="ts">
  import { onDestroy } from "svelte";
  import DialogWindow from "./DialogWindow.svelte";
  import SaveIcon from "./SaveIcon.svelte";
  import { runName, saveTime, chapter, roman } from "./save-browser";
  import {
    repairModel,
    issueText,
    indexMessage,
    recordName,
    campaignConfirmation,
    recordConfirmation,
    type RepairConfirmation,
  } from "./repair-view";
  import type { BattleView, BattleCommands } from "../controller/view";
  import type {
    LoadProblem,
    RepairInventory,
    RepairRecord,
    RepairTarget,
  } from "../save/repair-types";
  import type { CurrentEntry } from "../save/metadata";
  let {
    view,
    actions,
    onNewGame,
    onManage,
  }: { view: BattleView; actions: BattleCommands; onNewGame: () => void; onManage: () => void } =
    $props();
  let dialog = $state<HTMLDialogElement>(),
    confirmation = $state<HTMLDialogElement>();
  let shown = $state.raw<LoadProblem | null>(null);
  let reviewing = $state(false),
    busy = $state(false),
    error = $state("");
  let inventory = $state<RepairInventory>({
    records: [],
    errors: [],
    authoritative: false,
    completedElsewhere: false,
  });
  let pending = $state<{
    target: RepairTarget | { kind: "resume"; expected?: CurrentEntry | null };
    message: RepairConfirmation;
  }>();
  let armed = $state(false);
  let timer: ReturnType<typeof setTimeout>;
  const model = $derived(repairModel(inventory, view.loadProblem));
  const kind = $derived(view.loadProblem?.kind);
  const title = $derived(
    reviewing
      ? model.ready
        ? "Saved games are ready"
        : "Repair saved games"
      : kind === "read"
        ? "Couldn’t read saved progress"
        : kind === "full"
          ? "Not enough space"
          : kind === "blocked"
            ? "Storage access is blocked"
            : "Repair saved games",
  );
  $effect(() => {
    const problem = view.loadProblem;
    if (!problem) {
      confirmation?.close();
      dialog?.close();
      shown = null;
    } else if (problem !== shown) {
      if (problem.kind === "ready" && shown?.kind !== "ready") {
        confirmation?.close();
        pending = undefined;
      }
      shown = problem;
      if (!dialog?.open) {
        reviewing = problem.kind === "data" || problem.kind === "ready";
        dialog?.showModal();
        if (reviewing) void refreshSafely();
      }
    }
  });
  async function refresh() {
    inventory = await actions.repairInventory();
  }
  async function refreshSafely() {
    busy = true;
    try {
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  export async function open() {
    confirmation?.close();
    pending = undefined;
    clearTimeout(timer);
    error = "";
    reviewing = true;
    if (!dialog?.open) dialog?.showModal();
    await refreshSafely();
  }
  function ask(target: NonNullable<typeof pending>["target"], message: RepairConfirmation) {
    if (busy) return;
    pending = { target, message };
    armed = false;
    clearTimeout(timer);
    timer = setTimeout(() => (armed = true), 350);
    confirmation?.showModal();
    confirmation?.querySelector<HTMLButtonElement>(".cancel")?.focus();
  }
  function resume() {
    const flash = model.flash?.entry?.kind === "flash" ? model.flash.entry : null;
    ask(
      { kind: "resume", expected: flash },
      {
        title: flash ? "Restore saved game?" : "Finish and choose new game?",
        subject: flash ? `${runName(flash)} · ${saveTime(flash.savedAt)}` : undefined,
        copy: flash
          ? "Your current unsaved play will be replaced by this flash save."
          : "There is no flash save to restore. This finishes the update and opens New game, replacing any current unsaved play.",
        action: flash ? "Restore saved game" : "Finish update",
        destructive: false,
      },
    );
  }
  function reviewReadAgain() {
    ask(
      { kind: "resume" },
      {
        title: "Try reading saved progress again?",
        copy: "Stored progress hasn’t been identified. If it can be read, its flash save will replace your current unsaved play. If there is no flash save, New game will open.",
        action: "Try reading again",
        destructive: false,
      },
    );
  }
  function rebuild() {
    ask(
      { kind: "index" },
      {
        title: "Rebuild save list?",
        copy: "Rebuild the list from identifiable snapshots. Unidentified saves and conflicting campaign claims will remain for explicit cleanup.",
        action: "Rebuild save list",
        destructive: false,
      },
    );
  }
  function reset() {
    ask(
      { kind: "all" },
      {
        title: "Delete all retained saves?",
        copy: "All retained saves will be deleted, including flash saves, unidentified saves and save-list entries.",
        action: "Delete all retained saves",
        destructive: true,
      },
    );
  }
  async function confirmAction() {
    if (!pending || busy || !armed) return;
    const action = pending;
    busy = true;
    error = "";
    try {
      if (action.target.kind === "resume") {
        confirmation?.close();
        if (!(await actions.retrySavedProgress(action.target.expected))) await refresh();
      } else {
        await actions.repairSavedData(action.target);
        confirmation?.close();
        await refresh();
      }
    } catch (e) {
      error = String(e);
      try {
        await refresh();
      } catch {
        /* Original failure remains available. */
      }
    } finally {
      busy = false;
    }
  }
  async function readAgain() {
    busy = true;
    error = "";
    try {
      if (!(await actions.retrySavedProgress()))
        error = "Saved progress still couldn’t be read. Stored data has not been replaced.";
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
  function manage() {
    dialog?.close();
    onManage();
  }
  onDestroy(() => clearTimeout(timer));
</script>

{#snippet technical(record: RepairRecord)}
  <details class="technical">
    <summary>Technical details</summary>
    <code>{record.key}</code>
    {#if record.error}<pre>{record.error}</pre>{/if}
    {#if record.ownership === "conflicting"}{#each record.claims as claim}<pre>{runName(
            claim,
          )} · {claim.runId} · {claim.mission} · {saveTime(claim.savedAt)}</pre>{/each}{/if}
  </details>
{/snippet}
{#snippet row(record: RepairRecord, attention = false)}
  <div
    class:issue={attention}
    class:warning={attention && record.kind === "flash" && record.ownership !== "known"}
    class="record"
    data-repair-key={record.key}
  >
    <div class="record-top">
      <div class="copy">
        {#if attention && record.entry}<p class="campaign-context">{runName(record.entry)}</p>{/if}
        {#if record.entry?.kind === "turn" && chapter(record.entry).number}
          <span class="turn-name">
            <span class="numeral">{roman(chapter(record.entry).number)}</span>
            <strong>{chapter(record.entry).name}, {record.entry.round}</strong>
          </span>
        {:else}<strong>{recordName(record)}</strong>{/if}
        {#if record.entry}<p class="time">{saveTime(record.entry.savedAt)}</p>{/if}
        {#if record.issue}<p class="damage">{issueText(record)}</p>{/if}
      </div>
      {#if record.kind !== "index" && !(record.kind === "flash" && record.ownership !== "known") && record.ownership !== "conflicting" && record.issue !== "missing_snapshot"}
        <button
          class="trash"
          disabled={busy}
          aria-label={`Delete ${recordName(record)}`}
          onclick={() => ask({ kind: "record", key: record.key }, recordConfirmation(record))}
        >
          <SaveIcon name="trash" />
        </button>
      {/if}
    </div>
    {#if record.kind === "flash" && record.ownership !== "known"}
      <p>
        Deleting a campaign will leave this flash save untouched. We strongly recommend deleting it
        separately.
      </p>
      <button
        class="delete-link"
        disabled={busy}
        onclick={() => ask({ kind: "record", key: record.key }, recordConfirmation(record))}
      >
        <SaveIcon name="trash" />Delete flash save
      </button>
    {:else if record.issue === "missing_snapshot"}
      <button
        class="delete-link"
        disabled={busy}
        onclick={() => ask({ kind: "record", key: record.key }, recordConfirmation(record))}
      >
        <SaveIcon name="trash" />Remove missing turn from list
      </button>
    {:else if record.ownership === "conflicting"}
      <p>
        This turn is linked to {new Set(record.claims.map((claim) => claim.runId)).size} different playthroughs:
        {Array.from(new Set(record.claims.map(runName)))
          .sort()
          .join(" and ")}. Deleting a campaign will leave this turn untouched.
      </p>
      <button
        class="delete-link"
        disabled={busy}
        onclick={() => ask({ kind: "record", key: record.key }, recordConfirmation(record))}
      >
        <SaveIcon name="trash" />Delete unidentified turn save
      </button>
    {:else if record.kind === "index"}
      <p>{indexMessage(record)}</p>
      <button class="rebuild" disabled={busy} onclick={rebuild}>Rebuild save list</button>
    {/if}
    {@render technical(record)}
  </div>
{/snippet}
<DialogWindow
  {title}
  bind:dialog
  width="540px"
  tall={reviewing}
  canDismiss={!busy}
  className="repair-window"
>
  <div class="repair-scroll">
    {#if !reviewing}
      {#if kind === "read"}<p>
          The browser couldn’t read saved progress. We don’t know whether a flash save is available.
        </p>
        <p class="status-note">Saved data has been left untouched. Automatic saving is paused.</p>
        <button class="primary" disabled={busy} onclick={readAgain}>Try reading again</button>
      {:else if kind === "full"}<p>
          There isn’t enough space to finish updating saved games. Delete saves you no longer need,
          then finish the update.
        </p>
      {:else if kind === "blocked"}<p>
          The browser refused access to saved data. Allow storage in your browser’s site settings,
          or play without saving.
        </p>{/if}
      <button class:primary={kind === "full"} class="review" disabled={busy} onclick={() => open()}>
        Review saved games
      </button>
      <button class="play" disabled={busy} onclick={play}>New game without saving</button>
    {:else}
      <p class="intro" role="status">
        {busy && !inventory.records.length ? "Checking retained saves…" : model.summary}
      </p>
      {#if inventory.errors.length}<button class="review" disabled={busy} onclick={reviewReadAgain}>
          Try reading again
        </button>{/if}
      {#if model.attention.length}<p class="section-label">Needs attention</p>
        {#each model.attention as record (record.key)}{@render row(record, true)}{/each}{/if}
      {#if model.campaigns.length}<p class="section-label">
          {model.attention.length ? "Also retained" : "Saved games"}
        </p>
        {#each model.campaigns as campaign (campaign.id)}<details
            class="campaign"
            data-repair-run={campaign.id}
          >
            <summary>
              <span>
                {campaign.name}
                <small>
                  {campaign.records.filter((r) => r.kind === "turn").length}
                  {campaign.records.filter((r) => r.kind === "turn").length === 1
                    ? "turn save"
                    : "turn saves"}{campaign.records.some((r) => r.kind === "flash")
                    ? " · Flash save"
                    : ""}
                </small>
              </span>
            </summary>
            {#each campaign.records.filter((record) => !record.issue) as record (record.key)}{@render row(
                record,
              )}{/each}
            {#if !model.ready}<button
                class="delete-link"
                disabled={busy}
                onclick={() =>
                  ask(
                    { kind: "campaign", runId: campaign.id },
                    campaignConfirmation(inventory, campaign.id),
                  )}
              >
                <SaveIcon name="trash" />Delete campaign saves
              </button>{/if}
          </details>{/each}
      {/if}
    {/if}
    {#if error}<p class="damage" role="alert">{error}</p>{/if}
    <details class="technical failure">
      <summary>Technical details</summary>
      <pre>{view.loadProblem?.detail}</pre>
      {#each inventory.errors as issue}<pre>{issue}</pre>{/each}
    </details>
  </div>
  {#if reviewing}<footer>
      {#if model.hint}<p class="hint">{model.hint}</p>{/if}
      <button class="primary" disabled={busy || !model.canContinue} onclick={resume}>
        {model.continuation}
      </button>
      {#if model.ready}<button class="review" disabled={busy} onclick={manage}>
          Saved games
        </button>{/if}
      <button class="play" disabled={busy} onclick={play}>
        {model.ready ? "New game" : "New game without saving"}
      </button>
      <div class="support">
        <button onclick={() => actions.downloadDebugReport()}>Download debug report</button>
        <button class="delete-link" disabled={busy || model.ready} onclick={reset}>
          <SaveIcon name="trash" />Delete all retained saves
        </button>
      </div>
    </footer>{/if}
</DialogWindow>
<DialogWindow
  title={pending?.message.title ?? "Confirm saved-game action"}
  bind:dialog={confirmation}
  width="400px"
  canDismiss={!busy}
>
  {#if pending?.message.subject}<p class="subject">{pending.message.subject}</p>{/if}
  <p>{pending?.message.copy}</p>
  {#if pending?.message.notice}<p class="notice" class:warning={pending.message.warning}>
      {pending.message.notice}
    </p>{/if}
  {#if pending?.message.destructive}<p class="irreversible">This cannot be undone.</p>{/if}
  {#if error}<p class="damage" role="alert">{error}</p>{/if}
  <button
    class="confirm"
    class:destructive={pending?.message.destructive}
    disabled={busy || !armed}
    onclick={confirmAction}
  >
    {pending?.message.action}
  </button>
  <button class="cancel" disabled={busy} onclick={() => confirmation?.close()}>Cancel</button>
</DialogWindow>

<style>
  p {
    font:
      13px/1.5 system-ui,
      sans-serif;
    color: var(--muted);
    overflow-wrap: anywhere;
    margin: 0 0 12px;
  }
  .intro {
    color: #d4dbc6;
    font-size: 14px;
  }
  .section-label {
    text-transform: uppercase;
    letter-spacing: 1px;
    font-size: 10px;
    margin: 21px 0 10px;
  }
  .record {
    padding: 12px 0;
    border-bottom: 1px solid var(--line);
  }
  .record-top {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .copy {
    min-width: 0;
    flex: 1;
  }
  .turn-name {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .numeral {
    display: inline-grid;
    place-items: center;
    flex: none;
    min-width: 25px;
    height: 26px;
    padding: 0 3px;
    border: 1px solid #716b46;
    border-radius: 3px;
    color: #ddc476;
    font:
      600 17px Georgia,
      serif;
  }
  .record strong {
    font:
      15px/1.4 Georgia,
      serif;
  }
  .time,
  .campaign-context {
    font-size: 11px;
    margin: 4px 0;
  }
  .damage {
    color: #e2aa96;
    font-size: 12px;
  }
  .issue {
    border: 1px solid #745744;
    border-left: 3px solid #b77a64;
    border-radius: 4px;
    padding: 13px 12px;
    margin: 0 0 13px;
    background: #b77a6408;
  }
  .issue.warning {
    border-color: #8e7c44;
    border-left-color: #cbb16b;
    background: #cbb16b08;
  }
  .issue.warning strong {
    color: #dfc889;
  }
  .trash {
    background: none;
    border: 0;
    color: #d09a86;
    padding: 10px;
  }
  .delete-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 0;
    background: none;
    color: #d19a87;
    font-size: 12px;
    padding: 8px 0;
  }
  .technical {
    font:
      11px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
    margin-top: 10px;
  }
  .technical summary {
    cursor: pointer;
  }
  code,
  pre {
    overflow-wrap: anywhere;
    white-space: pre-wrap;
    font-size: 11px;
  }
  .campaign {
    border-top: 1px solid var(--line);
  }
  .campaign > summary {
    cursor: pointer;
    font:
      600 19px/1.3 Georgia,
      serif;
    padding: 15px 0;
  }
  .campaign small {
    display: block;
    font:
      11px/1.4 system-ui,
      sans-serif;
    color: var(--muted);
    margin-top: 5px;
  }
  .rebuild,
  .review {
    background: none;
    border: 1px solid #68734f;
    border-radius: 4px;
    min-height: 40px;
    font-size: 12px;
    padding: 8px 12px;
  }
  .rebuild {
    width: 100%;
  }
  .primary,
  .play,
  .confirm,
  .cancel,
  .review {
    display: block;
    width: 100%;
    min-height: 43px;
    margin: 10px 0;
    font-size: 13px;
  }
  .primary,
  .confirm {
    background: var(--gold);
    color: var(--bg);
    border: 1px solid var(--gold);
    border-radius: 4px;
    font-weight: 600;
  }
  .primary:disabled {
    background: #505b3c;
    color: #aab197;
    border-color: #5f6b49;
  }
  .confirm.destructive {
    background: #a45141;
    border-color: #bc7460;
    color: #fff0e5;
  }
  .play {
    background: none;
    border: 0;
    color: #d19a87;
    font-size: 12px;
  }
  .cancel {
    background: none;
    border: 1px solid #66724e;
    border-radius: 4px;
  }
  .subject {
    font:
      600 19px/1.4 Georgia,
      serif;
    color: #ece6d1;
  }
  .notice,
  .status-note {
    padding: 11px 12px;
    border-left: 2px solid #859467;
    background: #8594670a;
    color: #c8d0b9;
  }
  .notice.warning {
    border-color: #d1b567;
    color: #dfc68a;
  }
  .irreversible {
    font-size: 12px;
  }
  .failure {
    margin: 17px 0;
  }
  .hint {
    font-size: 11px;
    margin-bottom: 8px;
  }
  .support {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .support button {
    padding: 5px 0;
    border: 0;
    background: none;
    font-size: 10px;
    color: var(--muted);
  }
  .support .delete-link {
    color: #cb9681;
  }
  .support :global(svg) {
    width: 13px;
    height: 13px;
  }
  footer {
    position: sticky;
    bottom: 0;
    background: var(--surface);
    border-top: 1px solid var(--line);
    margin: 14px -20px -20px;
    padding: 11px 20px 14px;
    z-index: 1;
  }
</style>
