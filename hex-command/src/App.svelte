<script lang="ts">
  import { onMount } from "svelte";
  import { BattleController } from "./controller/battle";
  import { registerBattleTools } from "./controller/tools";
  import { unlockSound, playSound } from "./presentation/sound";
  import TurnBar from "./ui/TurnBar.svelte";
  import Battlefield from "./ui/Battlefield.svelte";
  import UnitPanel from "./ui/UnitPanel.svelte";
  import TerrainPanel from "./ui/TerrainPanel.svelte";
  import SettingsDialog from "./ui/SettingsDialog.svelte";
  import MapPicker from "./ui/MapPicker.svelte";
  const controller = new BattleController();
  const view = controller.view;
  const actions = controller.commands;
  let settingsDialog = $state<HTMLDialogElement>();
  let resetDialog: HTMLDialogElement;
  let mapChoice = $state("mirror");
  function openMapPicker() {
    settingsDialog?.close();
    mapChoice = $view.game.mapId;
    resetDialog.showModal();
  }
  function openSettings() {
    settingsDialog?.showModal();
    playSound("select");
  }
  function startNewBattle() {
    resetDialog.close();
    actions.reset(mapChoice);
  }
  function globalKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && !document.querySelector("dialog[open]"))
      actions.clearSelection(true);
  }
  onMount(() => {
    // Sound activation requires a real browser gesture; loading/replay stays silent.
    const passiveGestures = ["pointerdown", "pointerup", "touchend"] as const;
    const otherGestures = ["keydown", "click"] as const;
    const activateSound = (event: Event) => {
      void unlockSound(event);
    };
    for (const event of passiveGestures)
      document.addEventListener(event, activateSound, { capture: true, passive: true });
    for (const event of otherGestures)
      document.addEventListener(event, activateSound, { capture: true });
    controller.initialize();
    const unregisterTools = registerBattleTools(controller);
    return () => {
      controller.dispose();
      unregisterTools();
      for (const event of [...passiveGestures, ...otherGestures])
        document.removeEventListener(event, activateSound, { capture: true });
    };
  });
</script>

<svelte:document onkeydown={globalKeydown} />
<main>
  <TurnBar view={$view} {actions} onSettings={openSettings} />
  <p id="status" class="status" role="status" aria-live="polite">{$view.status}</p>
  <div class="workspace">
    <Battlefield view={$view} {actions} />
    <aside>
      <UnitPanel view={$view} {actions} onNewBattle={openMapPicker} />
      <TerrainPanel view={$view} {actions} />
    </aside>
  </div>
</main>
<SettingsDialog view={$view} {actions} bind:dialog={settingsDialog} onNewBattle={openMapPicker} />
<dialog bind:this={resetDialog} id="reset-dialog">
  <h2>Start a new battle?</h2>
  <p>Choose your battlefield. Your current battle will be replaced.</p>
  <MapPicker bind:selected={mapChoice} />
  <div class="dialog-actions">
    <button id="keep-playing" class="quiet" onclick={() => resetDialog.close()}>
      Keep playing
    </button>
    <button id="confirm-reset" class="primary" onclick={startNewBattle}>New battle</button>
  </div>
</dialog>
