<script lang="ts">
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
  let savedGamesPanel: HTMLDetailsElement;
  let savedGames: { open(): void };
  export function openSavedGames() {
    dialog?.showModal();
    savedGamesPanel.open = true;
    savedGames.open();
    savedGamesPanel.querySelector("summary")?.scrollIntoView({ block: "nearest" });
  }
  function closeOnBackdropClick(event: MouseEvent) {
    if (!dialog || event.target !== dialog) return;
    const bounds = dialog.getBoundingClientRect();
    const clickedOutside =
      event.clientX < bounds.left ||
      event.clientX > bounds.right ||
      event.clientY < bounds.top ||
      event.clientY > bounds.bottom;
    if (clickedOutside) dialog.close();
  }
</script>

<dialog bind:this={dialog} id="settings-dialog" onclick={closeOnBackdropClick}>
  <div class="settings-heading">
    <h2>Settings</h2>
    <button
      id="close-settings"
      class="icon-button"
      aria-label="Close settings"
      onclick={() => dialog?.close()}
    >
      ×
    </button>
  </div>
  <label class="sound-control" for="sound-effects">
    <span>Sound effects</span>
    <input
      id="sound-effects"
      type="checkbox"
      checked={view.sound}
      onchange={(event) => void actions.setSound(event.currentTarget.checked)}
    />
  </label>
  <button id="new-game" class="primary" onclick={onNewBattle}>New game</button>
  <details
    class="panel"
    bind:this={savedGamesPanel}
    ontoggle={() => {
      if (savedGamesPanel.open) savedGames.open();
    }}
  >
    <summary>Saved games</summary>
    <SavedProgress bind:this={savedGames} {view} {actions} onLoad={() => dialog?.close()} />
  </details>
  <details class="panel">
    <summary>Field manual</summary>
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
  </details>
  {#if view.game.campaign}
    <details class="panel">
      <summary>Campaign conversations</summary>
      {#each view.game.campaign.dialogue.slice(0, view.game.campaign.dialogueIndex) as line}<p>
          <strong>{line.speaker}</strong>
          {line.text}
        </p>{/each}
    </details>
  {/if}
  <details class="dispatch">
    <summary>Battle journal</summary>
    <ol id="log">
      {#each view.game.log as text}<li>{text}</li>{/each}
    </ol>
  </details>
  <details class="panel" id="opponent-difficulty">
    <summary>Opponent difficulty · {view.opponent.difficulty}</summary>
    <p>
      {#if view.opponent.planner && view.opponent.recruiter}
        Planner: {view.opponent.planner}
        <br />
        Recruiter: {view.opponent.recruiter}
      {:else}
        Policy: {view.opponent.policy}
      {/if}
    </p>
  </details>
  <details class="panel" id="battle-debug">
    <summary>Debug tools</summary>
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
    <TraceControls {view} onStart={() => dialog?.close()} />
    <h3>Mirror Night</h3>
    <p>
      Start a new Mirror battle at night: explore fog, capture villages to light their lamps, and
      leave a red light at every death. This replaces the current battle.
    </p>
    <button
      onclick={() => {
        try {
          actions.startCampaign("mirror-night");
          dialog?.close();
        } catch (error) {
          actions.error(error);
        }
      }}
    >
      Start Mirror Night
    </button>
  </details>
</dialog>
