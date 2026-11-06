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
  }
  let { view, actions, dialog = $bindable(), onNewBattle }: Props = $props();
  type Screen = "hub" | "saves" | "manual" | "conversations" | "journal" | "debug";
  const titles: Record<Screen, string> = {
    hub: "Settings",
    saves: "Saved games",
    manual: "Field manual",
    conversations: "Campaign conversations",
    journal: "Battle journal",
    debug: "Debug tools",
  };
  let screen = $state<Screen>("hub");
  let body = $state<HTMLDivElement>();
  let frame: { focusHeading(): void };
  let savedGames: { open(): void };
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
    rememberPosition();
    const previous = screen;
    screen = next;
    if (next === "saves") savedGames.open();
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
    screen = "saves";
    savedGames.open();
    dialog?.showModal();
    void tick().then(() => {
      if (screen === "saves" && dialog?.open && body) {
        body.scrollTop = positions.get("saves") ?? 0;
        frame.focusHeading();
      }
    });
  }
  async function startMirror() {
    try {
      await actions.startCampaign("mirror-night");
      close();
    } catch (error) {
      actions.error(error);
    }
  }
</script>

{#snippet chevron()}
  <svg class="chevron" viewBox="0 0 16 16" aria-hidden="true"><path d="m6 3 5 5-5 5" /></svg>
{/snippet}
{#snippet entry(label: string, target: Screen)}
  <button class="setting-row" data-window={target} onclick={() => navigate(target)}>
    <span class="label">{label}</span>
    {@render chevron()}
  </button>
{/snippet}
<DialogWindow
  bind:this={frame}
  bind:dialog
  bind:body
  onBeforeClose={rememberPosition}
  id="settings-dialog"
  title={titles[screen]}
  width={screen === "hub" ? "420px" : screen === "saves" ? "500px" : "680px"}
  tall={screen !== "hub"}
  onBack={screen === "hub"
    ? undefined
    : () => {
        void navigate("hub");
      }}
>
  <div class="settings-hub" hidden={screen !== "hub"}>
    <section>
      <h3 class="group-label">Game</h3>
      <label class="setting-row sound-row" for="sound-effects">
        <span class="label">Sound effects</span>
        <input
          id="sound-effects"
          type="checkbox"
          checked={view.sound}
          onchange={(event) => void actions.setSound(event.currentTarget.checked)}
        />
      </label>
      <details class="difficulty" id="opponent-difficulty">
        <summary class="setting-row">
          <span class="label">Opponent difficulty</span>
          <span class="value">{view.opponent.difficulty}</span>
          {@render chevron()}
        </summary>
        <p class="difficulty-details">
          {#if view.opponent.planner && view.opponent.recruiter}
            Planner: {view.opponent.planner}
            <br />
            Recruiter: {view.opponent.recruiter}
          {:else}Policy: {view.opponent.policy}{/if}
        </p>
      </details>
      {@render entry("Saved games", "saves")}
      <button class="new-game" disabled={saving || saved || !view.canSave} onclick={saveGame}>
        {saving ? "Saving…" : saved ? "Saved" : "Save game"}
      </button>
      <button id="new-game" class="new-game" onclick={newBattle}>New game</button>
    </section>
    <section>
      <h3 class="group-label">Reference</h3>
      {@render entry("Field manual", "manual")}
      {#if view.game.campaign}{@render entry("Campaign conversations", "conversations")}{/if}
      {@render entry("Battle journal", "journal")}
    </section>
    <section>
      <h3 class="group-label">Advanced</h3>
      {@render entry("Debug tools", "debug")}
    </section>
  </div>
  <div hidden={screen !== "saves"}>
    <SavedProgress bind:this={savedGames} {view} {actions} onLoad={close} />
  </div>
  <div class="reference" hidden={screen !== "manual"}>
    <div class="manual">
      <p>Each unit moves once and attacks once. Attacking ends its turn.</p>
      <!-- Keep punctuation adjacent to the inline emphasis. -->
      <!-- prettier-ignore -->
      <p>
        Capture a village by entering it. Each owned village earns
        <strong>2 gold</strong> at the start of your turn and heals a
        friendly occupant by <strong>4 HP</strong>.
      </p>
      <p>
        Select an enemy, then tap a weapon to attack. Use the magnifying glass for damage details.
        The weapon’s count is its maximum number of strikes. The attacker strikes first, then the
        defender answers with its strongest legal weapon of the same kind: melee answers melee;
        ranged answers ranged. Strikes alternate, skipping a weapon that has used all its strikes,
        until both weapons finish or either unit dies. Both weapons must reach. Forests and hills
        block ranged shots through them.
      </p>
      <p>
        Each strike rolls damage independently within its weapon’s range. Armor, weaknesses and
        terrain modify each roll; damage cannot fall below 0. Check the unit’s armor and terrain
        details for its modifiers. There is no separate hit-or-miss roll. The combat preview shows
        the range of remaining HP and the chance of death. Green shows current HP guaranteed to
        remain, yellow shows HP that might be lost, red shows guaranteed loss, and blue shows HP
        that could be gained. The thin white marker shows current HP. These are final HP outcomes,
        including healing.
      </p>
      <p>
        Drain restores HP after each strike, based on the HP actually removed, rounded down and
        capped at maximum HP. Defensive thorns then reflect damage from each incoming melee strike,
        including a killing hit or one that deals no damage. Strikes prevented by death do not roll,
        heal or trigger thorns.
      </p>
      <p>
        Each unit’s Movement costs list shows how much movement it spends entering each terrain.
        Occupied hexes block movement. Units have no zone of control and do not intercept attacks;
        protect a fragile unit by blocking the enemy’s approach, rather than standing behind it.
        Forests reduce ranged damage by 1 for land races. Hills reduce their damage received by 1.
        Land races cannot cross deep water; bridges provide a crossing. Shallow water and marsh add
        1 to their damage received. Merfolk swim through deep water, receive no shallow-water
        penalty, and take 1 less damage in marsh. Forests, groves and hills slow merfolk and add 1
        to their damage received. Merfolk move normally on plains.
      </p>
      <p>
        Keep your leader on your keep, then select an adjacent empty hex passable to the recruit.
        Merfolk may recruit into water. The unit appears on that hex and acts next turn.
      </p>
      <p>
        All unit families have a finite advancement ladder. Dealing damage earns experience;
        defeating an enemy earns more, including during retaliation. Select a ready unit with enough
        XP to advance. Advancement preserves wounds and consumes its turn. Returning companions and
        recalled survivors keep their XP and rank between campaign missions. Retrying restores the
        company that entered that mission.
      </p>
      <p>
        In skirmishes, defeat the enemy commander to win. In campaigns, follow the mission
        objectives and protect your leader. Groves heal their owner’s occupants; forts protect
        against all attacks; ruins shelter against ranged attacks. Marsh slows heavy units. Bones
        resist arrows but are weak against impact; wolves are fast but vulnerable to pierce damage.
        Keyboard: Tab to the map, use arrow keys to browse hexes, Enter to select. Escape clears
        selection.
      </p>
    </div>
  </div>
  <div class="reference conversations" hidden={screen !== "conversations"}>
    {#if view.game.campaign}
      {#each view.game.campaign.dialogue.slice(0, view.game.campaign.dialogueIndex) as line}
        <p>
          <strong>{line.speaker}</strong>
          <br />
          {line.text}
        </p>
      {:else}<p>No conversations yet.</p>{/each}
    {/if}
  </div>
  <div class="reference" hidden={screen !== "journal"}>
    <ol id="log">
      {#each view.game.log as text}<li>{text}</li>{:else}<li>No entries yet.</li>{/each}
    </ol>
  </div>
  <div class="reference" id="battle-debug" hidden={screen !== "debug"}>
    <p>
      Download the current battle, the last five enemy turns, and saved progress from this browser
      to help investigate issues.
    </p>
    <p>
      Enemy-turn recordings start when you open the game and clear when you reload or change
      missions. The report includes all saved runs and enemies hidden by fog.
    </p>
    <button onclick={() => actions.downloadDebugReport()}>Download debug report</button>
    <h3>Performance trace</h3>
    <TraceControls {view} onStart={close} />
    <h3>Mirror Night</h3>
    <p>
      Start a new Mirror battle at night: explore fog, capture villages to light their lamps, and
      leave a red light at every death. This replaces the current battle.
    </p>
    <button onclick={startMirror}>Start Mirror Night</button>
  </div>
</DialogWindow>

<style>
  .settings-hub {
    padding-top: 8px;
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
    background: none;
    border: 0;
    border-bottom: 1px solid #39413688;
    border-radius: 0;
    color: #e1e3d6;
    font:
      14px/1.45 system-ui,
      sans-serif;
  }
  section > .setting-row:last-child {
    border-bottom: 0;
  }
  .label {
    flex: 1;
  }
  .value {
    color: var(--muted);
    font-size: 12px;
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
  button.setting-row:hover {
    color: var(--gold);
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
  .difficulty {
    margin: 0;
    padding: 0;
    border: 0;
  }
  .difficulty summary {
    list-style: none;
    cursor: pointer;
  }
  .difficulty summary::-webkit-details-marker {
    display: none;
  }
  .difficulty[open] .chevron {
    transform: rotate(90deg);
  }
  .difficulty-details {
    margin: 0;
    padding: 5px 0 12px;
    font-size: 12px;
  }
  .new-game {
    min-height: 44px;
    width: 100%;
    margin: 12px 0 4px;
    border: 1px solid #7f744d;
    border-radius: 4px;
    background: #d8ba6f08;
    color: #dfcc95;
    font-weight: 500;
  }
  .new-game:hover {
    background: #d8ba6f12;
  }
  .reference {
    padding-top: 4px;
  }
  .reference h3 {
    font:
      20px/1.25 Georgia,
      serif;
    margin: 24px 0 12px;
  }
  .conversations strong {
    color: var(--text);
  }
</style>
