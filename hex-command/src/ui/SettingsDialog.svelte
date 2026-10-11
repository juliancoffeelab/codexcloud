<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  interface Props {
    view: BattleView;
    actions: BattleCommands;
    dialog?: HTMLDialogElement;
    onNewBattle: () => void;
    onNewCampaign: () => void;
  }
  let { view, actions, dialog = $bindable(), onNewBattle, onNewCampaign }: Props = $props();
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
  <button id="new-campaign" class="primary" onclick={onNewCampaign}>New campaign</button>
  <button
    id="new-game"
    class="quiet"
    disabled={view.game.turn === "enemy" && !view.game.winner}
    onclick={onNewBattle}
  >
    New battle
  </button>
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
        Each unit attacks once per turn. A surviving defender replies once with its strongest weapon
        of the same kind: melee answers melee; ranged answers ranged. Both weapons must reach.
        Forests and hills block ranged shots through them.
      </p>
      <p>
        Blade, Pierce, and Impact damage interact with armor. Soldiers take 1 less Blade and Pierce
        damage, and 1 more Impact damage. Commanders take 1 less Blade damage. Archers and scouts
        have no modifiers. Armor and weakness change base damage first, then terrain cover and
        shallow-water exposure apply. No rounding is needed. Damage cannot fall below 0. A fully
        blocked hit still spends the attack, and a surviving defender can reply. The combat preview
        shows the exact outcome; there are no hit-chance rolls.
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
</dialog>
