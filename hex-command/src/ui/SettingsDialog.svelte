<script lang="ts">
  import type { BattleView, BattleController } from "../controller/battle";
  import type { ArtStyle } from "../presentation/art";
  interface Props {
    view: BattleView;
    controller: BattleController;
    dialog?: HTMLDialogElement;
    onNewBattle: () => void;
  }
  let { view, controller, dialog = $bindable(), onNewBattle }: Props = $props();
  let selectedArt = $state<ArtStyle>("normal");
  $effect(() => {
    if (!view.artLoading) selectedArt = view.artStyle;
  });
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
  function changeArt(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    if (value === "normal" || value === "low-poly" || value === "pixel")
      void controller.setArtStyle(value);
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
  <label class="art-control" for="art-style">
    <span>Art style</span>
    <select id="art-style" bind:value={selectedArt} disabled={view.artLoading} onchange={changeArt}>
      <option value="normal">Normal</option>
      <option value="low-poly">Low-poly</option>
      <option value="pixel">Pixel</option>
    </select>
  </label>
  <label class="sound-control" for="sound-effects">
    <span>Sound effects</span>
    <input
      id="sound-effects"
      type="checkbox"
      checked={view.sound}
      onchange={(event) => void controller.setSound(event.currentTarget.checked)}
    />
  </label>
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
        friendly occupant by <strong>2 HP</strong>.
      </p>
      <p>
        Select an enemy, then tap a weapon to attack. Use the magnifying glass for damage details.
        Each unit attacks once per turn. A surviving defender replies once with its strongest weapon
        of the same kind: melee answers melee; ranged answers ranged. Both weapons must reach.
        Forests and hills block ranged shots through them.
      </p>
      <p>
        Blade, Pierce, and Impact damage interact with armor. Soldiers receive 75% of Blade and
        Pierce base damage, and 125% of Impact base damage. Commanders receive 75% of Blade base
        damage. Archers and scouts receive 100%. Damage is rounded to the nearest whole number after
        resistance, then terrain cover and shallow-water exposure apply. Every hit deals at least 1
        damage. The combat preview shows the exact outcome; there are no hit-chance rolls.
      </p>
      <p>
        Forests cost 2 movement and reduce ranged damage by 1. Hills cost 2 movement and reduce all
        damage by 1. Scouts spend only 1 movement in forests and hills. Water is impassable; bridges
        cost 1 movement. Shallow water costs 3 for soldiers and commanders, 2 for archers and
        scouts, and adds 1 to damage received.
      </p>
      <p>
        Keep your commander on your keep, then select an adjacent empty land hex and choose a
        recruit. The unit appears on that hex and acts next turn.
      </p>
      <p>
        Defeat the enemy commander to win. Keyboard: Tab to the map, use arrow keys to browse hexes,
        Enter to select. Escape clears selection.
      </p>
    </div>
  </details>
  <details class="dispatch">
    <summary>Battle journal</summary>
    <ol id="log">
      {#each view.game.log as text}<li>{text}</li>{/each}
    </ol>
  </details>
</dialog>
