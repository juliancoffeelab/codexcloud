<script lang="ts">
  import { tick } from "svelte";
  import DialogWindow from "./DialogWindow.svelte";
  import TraceControls from "./TraceControls.svelte";
  import SavedProgress from "./SavedProgress.svelte";
  import type { BattleView, BattleCommands } from "../controller/view";
  interface Props {
    view: BattleView;
    actions: BattleCommands;
    dialog?: HTMLDialogElement;
    onNewBattle: () => void;
    onRepair: () => void;
    showCommanderArrow: boolean;
    commanderArrowError: string;
    onCommanderArrowChange: (enabled: boolean) => void;
  }
  let {
    view,
    actions,
    dialog = $bindable(),
    onNewBattle,
    onRepair,
    showCommanderArrow,
    commanderArrowError,
    onCommanderArrowChange,
  }: Props = $props();
  type Screen = "hub" | "saves";
  const titles: Record<Screen, string> = {
    hub: "Menu",
    saves: "Saved games",
  };
  let screen = $state<Screen>("hub");
  let body = $state<HTMLDivElement>();
  let frame: { focusHeading(): void };
  let savedGames: { open(): Promise<void> };
  let saving = $state(false);
  let saved = $state(false);
  async function saveGame() {
    saving = true;
    try {
      await actions.saveGame();
      saved = true;
    } catch {
      /* The controller exposes the save-failure dialog. */
    } finally {
      saving = false;
    }
  }
  const positions = new Map<Screen, number>();
  function rememberPosition() {
    positions.set(screen, body?.scrollTop ?? 0);
  }
  function close() {
    rememberPosition();
    dialog?.close();
  }
  function newBattle() {
    rememberPosition();
    onNewBattle();
  }
  async function navigate(next: Screen) {
    if (next === "saves" && view.loadProblem && view.loadProblem.kind !== "ready") {
      onRepair();
      return;
    }
    rememberPosition();
    const previous = screen;
    screen = next;
    if (next === "saves") void savedGames.open();
    await tick();
    if (!dialog?.open || screen !== next || !body) return;
    body.scrollTop = positions.get(next) ?? 0;
    if (next === "hub") {
      body
        .querySelector<HTMLButtonElement>(`[data-window="${previous}"]`)
        ?.focus({ preventScroll: true });
    } else frame.focusHeading();
  }
  export function open(focusId?: string) {
    screen = "hub";
    saved = false;
    dialog?.showModal();
    void tick().then(() => {
      if (screen === "hub" && dialog?.open && body) {
        body.scrollTop = positions.get("hub") ?? 0;
        if (focusId) body.querySelector<HTMLElement>(`#${focusId}`)?.focus({ preventScroll: true });
      }
    });
  }
  export function openSavedGames() {
    if (view.loadProblem && view.loadProblem.kind !== "ready") {
      onRepair();
      return;
    }
    screen = "saves";
    void savedGames.open();
    dialog?.showModal();
    void tick().then(() => {
      if (screen === "saves" && dialog?.open && body) {
        body.scrollTop = positions.get("saves") ?? 0;
        frame.focusHeading();
      }
    });
  }
  let starting = $state(false);
  let testBattle = $state<"developer" | "mirror-night" | "bell-night">();
  let confirmation = $state<HTMLDialogElement>();
  function requestTest(id: "developer" | "mirror-night" | "bell-night") {
    testBattle = id;
    confirmation?.showModal();
  }
  async function startDebug() {
    if (starting || !testBattle) return;
    starting = true;
    try {
      if (testBattle === "developer") await actions.reset("developer");
      else await actions.startCampaign(testBattle);
      confirmation?.close();
      close();
    } catch (error) {
      actions.error(error);
    } finally {
      starting = false;
    }
  }
</script>

{#snippet flask()}
  <svg class="flask" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M9 3h6M10 3v6l-6 10a1.3 1.3 0 0 0 1.1 2h13.8a1.3 1.3 0 0 0 1.1-2L14 9V3M7.5 15h9" />
  </svg>
{/snippet}
<DialogWindow
  bind:this={frame}
  bind:dialog
  bind:body
  onBeforeClose={rememberPosition}
  id="settings-dialog"
  title={titles[screen]}
  width={screen === "hub" ? "420px" : "500px"}
  tall={screen !== "hub"}
  onBack={screen === "hub" ? undefined : () => void navigate("hub")}
>
  <div class="settings-hub" hidden={screen !== "hub"}>
    <button class="new-game" disabled={saving || saved || !view.canSave} onclick={saveGame}>
      {saving ? "Saving…" : saved ? "Saved" : "Save game"}
    </button>
    <button class="new-game" data-window="saves" onclick={() => navigate("saves")}>
      {view.loadProblem && view.loadProblem.kind !== "ready" ? "Repair saved games" : "Saved games"}
    </button>
    <button id="new-game" class="new-game" onclick={newBattle}>New game</button>
    <label class="setting-row sound-row" for="sound-effects">
      <span class="label">Sound effects</span>
      <input
        id="sound-effects"
        type="checkbox"
        checked={view.sound}
        onchange={(event) => void actions.setSound(event.currentTarget.checked)}
      />
    </label>
    <details class="advanced">
      <summary class="setting-row">
        <span class="label">Advanced</span>
        <svg class="chevron" viewBox="0 0 16 16" aria-hidden="true"><path d="m3 6 5 5 5-5" /></svg>
      </summary>
      <label class="setting-row sound-row">
        <span class="label">Commander return arrow</span>
        <input
          type="checkbox"
          checked={showCommanderArrow}
          onchange={(event) => onCommanderArrowChange(event.currentTarget.checked)}
        />
      </label>
      <p class="help">Show a return arrow when your commander is outside the map view.</p>
      {#if commanderArrowError}
        <div role="alert">
          <p>
            The commander arrow preference could not be read or saved. Changes apply to this
            session.
          </p>
          <pre>{commanderArrowError}</pre>
          <button onclick={() => onCommanderArrowChange(showCommanderArrow)}>
            Retry saving preference
          </button>
        </div>
      {/if}
      <h3 class="group-label">Debug tools</h3>
      <p class="help">
        Download the battle, last five enemy turns, and saved progress. Includes enemies hidden by
        fog. Enemy-turn recordings clear on reload or mission change.
      </p>
      <button
        class="debug-action"
        data-activation="release"
        onclick={() => actions.downloadDebugReport()}
      >
        Download debug report
      </button>
      <h3 class="group-label">Performance trace</h3>
      <TraceControls {view} onStart={close} />
      <h3 class="group-label">Test maps</h3>
      <p class="help">Starting a test map replaces the current battle.</p>
      {#each [{ id: "developer", label: "Developer map" }, { id: "bell-night", label: "Bell Night" }, { id: "mirror-night", label: "Mirror Night" }] as test}
        <button
          class="debug-action test-launch"
          data-activation="release"
          disabled={starting}
          onclick={() => requestTest(test.id as "developer" | "bell-night" | "mirror-night")}
        >
          {@render flask()}
          <span>Start {test.label}</span>
        </button>
      {/each}
    </details>
    <a
      class="credits-link"
      href={`${import.meta.env.BASE_URL}assets/audio/palette/CREDITS.txt`}
      target="_blank"
      rel="noreferrer"
      aria-label="Credits (opens in a new tab)"
    >
      Credits <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M14 4h6v6M20 4 10 14M10 5H5a1 1 0 0 0-1 1v13a1 1 0 0 0 1 1h13a1 1 0 0 0 1-1v-5" />
      </svg>
    </a>
    <footer>
      <span>Difficulty: {view.opponent.difficulty}</span>
      <span>Build: {import.meta.env.VITE_BUILD_COMMIT}</span>
    </footer>
  </div>
  <div hidden={screen !== "saves"}>
    <SavedProgress bind:this={savedGames} {view} {actions} onLoad={close} />
  </div>
</DialogWindow>
<DialogWindow
  title="Start test map?"
  bind:dialog={confirmation}
  canDismiss={!starting}
  width="380px"
>
  <p>This replaces your current battle. Saved progress remains available in Saved games.</p>
  <button disabled={starting} onclick={() => confirmation?.close()}>Cancel</button>
  <button disabled={starting} data-activation="release" onclick={() => void startDebug()}>
    {starting ? "Starting…" : "Start test map"}
  </button>
</DialogWindow>

<style>
  .settings-hub {
    padding-top: 16px;
    font-family: system-ui, sans-serif;
  }
  .group-label {
    color: #919b87;
    font:
      600 10px/1.4 system-ui,
      sans-serif;
    letter-spacing: 1.5px;
    text-transform: uppercase;
    margin: 18px 0 3px;
  }
  .setting-row {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 52px;
    width: 100%;
    margin: 0;
    padding: 10px 0;
    text-align: left;
    text-decoration: none;
    background: none;
    border: 0;
    border-bottom: 1px solid #39413688;
    border-radius: 0;
    color: #e1e3d6;
    font:
      14px/1.45 system-ui,
      sans-serif;
  }
  .label {
    flex: 1;
  }
  .chevron {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: #a8ae97;
    stroke-width: 1.6;
    flex-shrink: 0;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .sound-row {
    cursor: pointer;
  }
  .sound-row input {
    appearance: none;
    width: 23px;
    height: 23px;
    margin: 0 1px 0 0;
    border: 1px solid #8f885c;
    border-radius: 4px;
    background: #151e15;
    display: grid;
    place-items: center;
  }
  .sound-row input:checked {
    background: #cfb76d;
    border-color: #cfb76d;
  }
  .sound-row input:checked::after {
    content: "";
    width: 11px;
    height: 6px;
    border-left: 2.5px solid #24291b;
    border-bottom: 2.5px solid #24291b;
    transform: rotate(-45deg) translateY(-1px);
  }
  .new-game {
    min-height: 44px;
    width: 100%;
    margin: 0 0 8px;
    border: 1px solid #7f744d;
    border-radius: 4px;
    background: #d8ba6f08;
    color: #dfcc95;
    font-weight: 500;
  }
  .new-game:hover {
    background: #d8ba6f12;
  }

  .advanced {
    margin: 0;
    padding: 0;
    border: 0;
  }
  .advanced summary {
    list-style: none;
    cursor: pointer;
    border: 0;
  }
  .advanced summary::-webkit-details-marker {
    display: none;
  }
  .advanced[open] summary .chevron {
    transform: rotate(180deg);
  }
  .advanced > label {
    border: 0;
  }
  .help {
    font:
      12px/1.5 system-ui,
      sans-serif;
    color: var(--muted);
    margin: 4px 0 12px;
  }
  .debug-action {
    width: 100%;
    min-height: 44px;
    margin: 6px 0 0;
    text-align: left;
  }
  .test-launch {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .flask,
  .credits-link svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    flex-shrink: 0;
  }
  .credits-link {
    display: flex;
    align-items: center;
    gap: 7px;
    width: fit-content;
    min-height: 44px;
    color: var(--muted);
    font:
      12px system-ui,
      sans-serif;
    text-decoration: none;
  }
  footer {
    display: flex;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
    border-top: 1px solid var(--line);
    margin-top: 4px;
    padding-top: 12px;
    color: var(--muted);
    font:
      11px/1.5 system-ui,
      sans-serif;
  }
</style>
