<script lang="ts">
  import type { HudArtworkCache } from "./presentation/hud-artwork";
  import { onMount, untrack } from "svelte";
  import { errorDetails } from "./error-details";
  import { BattleController } from "./controller/battle";
  import { registerBattleTools } from "./controller/tools";
  import { audioReadiness, playHudSound, prepareSounds } from "./presentation/sound";
  import { hudAudioReadiness } from "./presentation/hud-sound";
  import { prepareHudSound } from "./presentation/sound";
  import { itemSounds } from "./presentation/item-sound";
  import { missionSounds } from "./presentation/campaign-sound";
  import { CAMPAIGNS } from "./game/campaign-data";
  import NewGameDialog from "./ui/NewGameDialog.svelte";
  import CampaignPanel from "./ui/CampaignPanel.svelte";
  import Dialogue from "./ui/Dialogue.svelte";
  import TurnBar from "./ui/TurnBar.svelte";
  import Battlefield from "./ui/Battlefield.svelte";
  import UnitPanel from "./ui/UnitPanel.svelte";
  import TerrainPanel from "./ui/TerrainPanel.svelte";
  import GroundItemsPanel from "./ui/GroundItemsPanel.svelte";
  import SaveFailure from "./ui/SaveFailure.svelte";
  import ImportRepair from "./ui/ImportRepair.svelte";
  let repair: { open(): Promise<void> };
  let settings: { open(focusId?: string): void; openSavedGames(): void };
  import SettingsDialog from "./ui/SettingsDialog.svelte";
  let {
    controller,
    battlefield,
    previews,
  }: {
    controller: BattleController;
    battlefield: Promise<import("./presentation/phaser-battlefield").PreparedBattlefield>;
    previews: HudArtworkCache;
  } = $props();
  const view = $derived(controller.view);
  const actions = $derived(controller.commands);
  const soundCampaign = $derived(
    $view.game.campaign ? CAMPAIGNS[$view.game.campaign.id] : undefined,
  );
  const soundMission = $derived(soundCampaign?.missions[$view.game.campaign?.mission ?? 0]);
  $effect(() => {
    $view.game.mapId;
    const sounds = itemSounds([
      ...(soundMission?.itemIds ?? []),
      ...$view.game.units.flatMap((unit) => unit.items?.map((item) => item.id) ?? []),
      ...$view.game.groundItems.map((entry) => entry.item.id),
    ]);
    prepareSounds([
      ...(soundMission
        ? missionSounds(
            soundMission,
            soundCampaign && soundMission === soundCampaign.missions.at(-1)
              ? soundCampaign.ending
              : [],
          )
        : []),
      ...sounds,
    ]);
  });
  const inspectionOpen = $derived(
    $view.details.kind !== "none" || !!($view.game.winner && !$view.game.campaign),
  );
  let briefingExpanded = $state(false);
  const missionMapId = $derived($view.game.mapId);
  $effect(() => {
    missionMapId;
    briefingExpanded = false;
  });
  const commanderArrowKey = "hex-command-commander-arrow";
  let showCommanderArrow = $state(false);
  let commanderArrowError = $state("");
  function setCommanderArrow(enabled: boolean) {
    showCommanderArrow = enabled;
    commanderArrowError = "";
    try {
      localStorage.setItem(commanderArrowKey, String(enabled));
    } catch (error) {
      commanderArrowError = errorDetails(error);
    }
  }
  let settingsDialog = $state<HTMLDialogElement>();
  let newGame: { open: (mode: "skirmish" | "campaign", onBack?: () => void) => void };
  $effect(() => {
    if ($view.needsNewGame) {
      // Opening reads dialog state; only the controller's request should
      // trigger this effect, not a player changing tabs inside the picker.
      untrack(() => {
        settingsDialog?.close();
        newGame?.open("campaign");
      });
    }
  });
  function openCampaignPicker() {
    settingsDialog?.close();
    newGame.open("campaign");
  }
  function openMapPicker() {
    settingsDialog?.close();
    newGame.open("skirmish");
  }
  function openSettings() {
    settings.open();
    playHudSound();
  }
  function openNewGameFromSettings() {
    settingsDialog?.close();
    newGame.open("skirmish", () => settings.open("new-game"));
  }
  function globalKeydown(event: KeyboardEvent) {
    if (event.target instanceof HTMLCanvasElement) return;
    if (event.key === "Escape" && !document.querySelector("dialog[open]"))
      actions.clearSelection(true);
  }
  function clickAway(event: MouseEvent) {
    if (
      event.target instanceof Element &&
      !document.querySelector("dialog[open]") &&
      !event.target.closest(".phaser-battlefield, aside, button, select, input, summary, a, dialog")
    )
      actions.clearSelection(true);
  }
  onMount(() => {
    try {
      showCommanderArrow = localStorage.getItem(commanderArrowKey) === "true";
    } catch (error) {
      commanderArrowError = errorDetails(error);
    }
    const artworkFailed = (event: Event) => {
      if (
        event.target instanceof SVGImageElement &&
        event.target.getAttribute("href")?.includes("assets/sprites/")
      )
        actions.error(new Error("Artwork could not load. Please reload to retry."));
    };
    document.addEventListener("error", artworkFailed, true);
    controller.initialize();
    const unregisterTools = registerBattleTools(controller);
    return () => {
      prepareSounds([]);
      unregisterTools();
      document.removeEventListener("error", artworkFailed, true);
    };
  });
</script>

<svelte:document onkeydown={globalKeydown} onclick={clickAway} />
{#if !$view.needsNewGame}<main class="battle-layout">
    <TurnBar
      view={$view}
      {actions}
      onSettings={openSettings}
      {briefingExpanded}
      onBriefing={() => (briefingExpanded = !briefingExpanded)}
    />
    <p id="status" class="status" role="status" aria-live="polite">{$view.status}</p>
    {#if $hudAudioReadiness.error}
      <div role="alert">
        <p>Menu audio unavailable.</p>
        <details>
          <summary>Error details</summary>
          <pre>{$hudAudioReadiness.error}</pre>
        </details>
        <button onclick={() => void prepareHudSound()}>Retry menu audio</button>
      </div>
    {/if}
    {#if $audioReadiness.error}
      <div role="alert">
        <p>Audio unavailable.</p>
        <details>
          <summary>Error details</summary>
          <pre>{$audioReadiness.error}</pre>
        </details>
        <button onclick={() => void battlefield.then((owner) => owner.retryAudio())}>
          Retry audio
        </button>
      </div>
    {:else}
      <p class="status" role="status">
        {$audioReadiness.ready ? "Audio prepared." : "Preparing audio… Menus remain available."}
      </p>
    {/if}
    <div class="battle-content">
      <div class="campaign-overlay">
        <CampaignPanel
          view={$view}
          {actions}
          bind:expanded={briefingExpanded}
          onNewCampaign={openCampaignPicker}
        />
      </div>
      <div class="workspace" class:map-only={!inspectionOpen}>
        <Battlefield {previews} view={$view} {actions} {battlefield} {showCommanderArrow} />
        {#if inspectionOpen}
          <aside data-touch-scroll aria-label="Unit and terrain details">
            <UnitPanel view={$view} {actions} onNewBattle={openMapPicker} />
            <TerrainPanel view={$view} {actions} {previews} />
            <GroundItemsPanel view={$view} />
          </aside>
        {/if}
      </div>
    </div>
  </main>{/if}
<SettingsDialog
  view={$view}
  {actions}
  bind:dialog={settingsDialog}
  bind:this={settings}
  onNewBattle={openNewGameFromSettings}
  onRepair={() => {
    settingsDialog?.close();
    void repair.open();
  }}
  {showCommanderArrow}
  {commanderArrowError}
  onCommanderArrowChange={setCommanderArrow}
/>
{#if !$view.needsNewGame}<Dialogue view={$view} {actions} />{/if}
<NewGameDialog bind:this={newGame} view={$view} {actions} />

<SaveFailure view={$view} {actions} onManage={() => settings.openSavedGames()} />
<ImportRepair
  bind:this={repair}
  view={$view}
  {actions}
  onNewGame={openMapPicker}
  onManage={() => settings.openSavedGames()}
/>

<style>
  .campaign-overlay {
    position: absolute;
    inset: 0 var(--page-padding) auto;
    max-height: 100%;
    overflow: auto;
    z-index: 5;
    pointer-events: none;
  }
  .campaign-overlay :global(section) {
    pointer-events: auto;
  }
  main > [role="alert"] {
    position: absolute;
    inset: 80px 12px auto;
    max-height: 50%;
    overflow: auto;
    padding: 12px;
    background: var(--surface);
    border: 1px solid var(--line);
    z-index: 20;
  }
  [role="alert"] pre {
    max-height: 16rem;
    overflow: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
