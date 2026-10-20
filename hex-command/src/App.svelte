<script lang="ts">
  import { preloadTerrainMap } from "./presentation/terrain-preload";
  import { onMount } from "svelte";
  import { BattleController } from "./controller/battle";
  import { registerBattleTools } from "./controller/tools";
  import { unlockSound, playSound } from "./presentation/sound";
  import NewGameDialog from "./ui/NewGameDialog.svelte";
  import CampaignPanel from "./ui/CampaignPanel.svelte";
  import Dialogue from "./ui/Dialogue.svelte";
  import TurnBar from "./ui/TurnBar.svelte";
  import Battlefield from "./ui/Battlefield.svelte";
  import UnitPanel from "./ui/UnitPanel.svelte";
  import TerrainPanel from "./ui/TerrainPanel.svelte";
  import SettingsDialog from "./ui/SettingsDialog.svelte";
  let { controller }: { controller: BattleController } = $props();
  const view = $derived(controller.view);
  const actions = $derived(controller.commands);
  const inspectionOpen = $derived(
    !!($view.selected || $view.inspectedTile || ($view.game.winner && !$view.game.campaign)),
  );
  let briefingExpanded = $state(false);
  const missionMapId = $derived($view.game.mapId);
  $effect(() => {
    missionMapId;
    briefingExpanded = false;
  });
  let settingsDialog = $state<HTMLDialogElement>();
  let newGame: { open: (mode: "skirmish" | "campaign") => void };
  function openCampaignPicker() {
    settingsDialog?.close();
    newGame.open("campaign");
  }
  function openMapPicker() {
    settingsDialog?.close();
    newGame.open("skirmish");
  }
  function openSettings() {
    settingsDialog?.showModal();
    playSound("select");
  }
  function globalKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && !document.querySelector("dialog[open]"))
      actions.clearSelection(true);
  }
  function clickAway(event: MouseEvent) {
    if (
      event.target instanceof Element &&
      !document.querySelector("dialog[open]") &&
      !event.target.closest(".hex, aside, button, select, input, summary, a, dialog")
    )
      actions.clearSelection(true);
  }
  onMount(() => {
    const artworkFailed = (event: Event) => {
      if (
        event.target instanceof SVGImageElement &&
        event.target.getAttribute("href")?.includes("assets/sprites/")
      )
        actions.error(new Error("Artwork could not load. Please reload to retry."));
    };
    document.addEventListener("error", artworkFailed, true);
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
    preloadTerrainMap($view.game.mapId);
    controller.initialize();
    const unregisterTools = registerBattleTools(controller);
    return () => {
      controller.dispose();
      unregisterTools();
      document.removeEventListener("error", artworkFailed, true);
      for (const event of [...passiveGestures, ...otherGestures])
        document.removeEventListener(event, activateSound, { capture: true });
    };
  });
</script>

<svelte:document onkeydown={globalKeydown} onclick={clickAway} />
<main class="battle-layout">
  <TurnBar
    view={$view}
    {actions}
    onSettings={openSettings}
    {briefingExpanded}
    onBriefing={() => (briefingExpanded = !briefingExpanded)}
  />
  <p id="status" class="status" role="status" aria-live="polite">{$view.status}</p>
  <div class="battle-content">
    <CampaignPanel
      view={$view}
      {actions}
      bind:expanded={briefingExpanded}
      onNewCampaign={openCampaignPicker}
    />
    <div class="workspace" class:map-only={!inspectionOpen}>
      <Battlefield view={$view} {actions} />
      {#if inspectionOpen}
        <aside aria-label="Unit and terrain details">
          <UnitPanel view={$view} {actions} onNewBattle={openMapPicker} />
          <TerrainPanel view={$view} {actions} />
        </aside>
      {/if}
    </div>
  </div>
</main>
<SettingsDialog view={$view} {actions} bind:dialog={settingsDialog} onNewBattle={openMapPicker} />
<Dialogue view={$view} {actions} />
<NewGameDialog bind:this={newGame} view={$view} {actions} />
