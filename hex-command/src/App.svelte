<script lang="ts">
  import { onMount } from "svelte";
  import { BattleController } from "./controller/battle";
  import { registerBattleTools } from "./controller/tools";
  import { unlockSound, playSound } from "./presentation/sound";
  import CampaignPicker from "./ui/CampaignPicker.svelte";
  import CampaignPanel from "./ui/CampaignPanel.svelte";
  import Dialogue from "./ui/Dialogue.svelte";
  import type { CampaignId } from "./game/types";
  import TurnBar from "./ui/TurnBar.svelte";
  import Battlefield from "./ui/Battlefield.svelte";
  import UnitPanel from "./ui/UnitPanel.svelte";
  import TerrainPanel from "./ui/TerrainPanel.svelte";
  import SettingsDialog from "./ui/SettingsDialog.svelte";
  import MapPicker from "./ui/MapPicker.svelte";
  const controller = new BattleController();
  const view = controller.view;
  const actions = controller.commands;
  let briefingExpanded = $state(false);
  const missionMapId = $derived($view.game.mapId);
  $effect(() => {
    missionMapId;
    briefingExpanded = false;
  });
  let settingsDialog = $state<HTMLDialogElement>();
  let campaignDialog: HTMLDialogElement;
  let campaignChoice = $state<CampaignId>("lantern-road");
  function openCampaignPicker() {
    settingsDialog?.close();
    campaignDialog.showModal();
  }
  function startCampaign() {
    briefingExpanded = false;
    actions.startCampaign(campaignChoice);
    campaignDialog.close();
  }
  let resetDialog: HTMLDialogElement;
  let mapChoice = $state("mirror");
  function openMapPicker() {
    settingsDialog?.close();
    mapChoice =
      $view.game.mapId === "river" || $view.game.mapId === "developer"
        ? $view.game.mapId
        : "mirror";
    resetDialog.showModal();
  }
  function openSettings() {
    settingsDialog?.showModal();
    playSound("select");
  }
  function startNewBattle() {
    briefingExpanded = false;
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
  <TurnBar
    view={$view}
    {actions}
    onSettings={openSettings}
    {briefingExpanded}
    onBriefing={() => (briefingExpanded = !briefingExpanded)}
  />
  <p id="status" class="status" role="status" aria-live="polite">{$view.status}</p>
  <CampaignPanel
    view={$view}
    {actions}
    bind:expanded={briefingExpanded}
    onNewCampaign={openCampaignPicker}
  />
  <div class="workspace">
    <Battlefield view={$view} {actions} />
    <aside>
      <UnitPanel view={$view} {actions} onNewBattle={openMapPicker} />
      <TerrainPanel view={$view} {actions} />
    </aside>
  </div>
</main>
<SettingsDialog
  view={$view}
  {actions}
  bind:dialog={settingsDialog}
  onNewBattle={openMapPicker}
  onNewCampaign={openCampaignPicker}
/>
<Dialogue view={$view} {actions} />
<dialog class="campaign-dialog" bind:this={campaignDialog}>
  <h2>New campaign</h2>
  <p>Choose a company and a story. Starting replaces your current battle.</p>
  <CampaignPicker bind:selected={campaignChoice} artStyle={$view.artStyle} />
  <div class="dialog-actions">
    <button class="quiet" onclick={() => campaignDialog.close()}>Keep playing</button>
    <button class="primary" onclick={startCampaign}>Start campaign</button>
  </div>
</dialog>
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
