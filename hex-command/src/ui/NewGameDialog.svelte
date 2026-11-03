<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import type { CampaignId, UnitType } from "../game/types";
  import type { AiOptions, PlannerKind, RecruiterKind } from "../../rust/pkg/hex_command";
  import { DIFFICULTIES, PLANNERS, RECRUITERS, type SessionSetup } from "../game/setup";
  import { CAMPAIGNS } from "../game/campaign-data";
  import { BATTLE_MAPS } from "../game/maps";
  import { artSprite } from "../presentation/art";
  import { preloadTerrainMap } from "../presentation/terrain-preload";
  import ChoicePicker, { type Choice } from "./ChoicePicker.svelte";
  import MapThumbnail from "./MapThumbnail.svelte";
  let { view, actions }: { view: BattleView; actions: BattleCommands } = $props();
  let dialog: HTMLDialogElement;
  let confirmation: HTMLDialogElement;
  let mode = $state<"skirmish" | "campaign">("skirmish");
  let mapId = $state("mirror");
  let player = $state("human");
  let opponent = $state("skeletons");
  let campaignId = $state("lantern-road");
  let difficulty = $state("situational");
  let planner = $state("queued");
  let recruiter = $state("situational");
  let first = $state("random");
  let error = $state("");
  const factions: Choice[] = [
    { value: "human", label: "Humans", sprite: "commander" },
    { value: "woodland", label: "Woodland", sprite: "wildkeeper" },
    { value: "skeletons", label: "Skeletons", sprite: "necromancer" },
    { value: "coastal", label: "Merfolk", sprite: "tidekeeper" },
  ];
  const maps = BATTLE_MAPS.map((map) => ({
    value: map.id,
    label: map.id === "river-valley" ? "River Valley" : map.name,
    description: map.description,
    map,
  }));
  const campaigns = Object.values(CAMPAIGNS).map((c) => ({
    value: c.id,
    label: c.name,
    description: c.shortDescription,
    sprite: c.leader,
  }));
  const map = $derived(BATTLE_MAPS.find((m) => m.id === mapId)!);
  const campaign = $derived(CAMPAIGNS[campaignId as CampaignId]);
  const army = (id: string) => factions.find((f) => f.value === id)!;
  const title = (id: string) =>
    id === "aggressive-predictor" ? "Aggressive Predictor" : id[0]!.toUpperCase() + id.slice(1);
  const plannerChoices = PLANNERS.map((value) => ({ value, label: title(value) }));
  const recruiterChoices = RECRUITERS.map((value) => ({ value, label: title(value) }));
  function setDifficulty(id: string) {
    const preset = DIFFICULTIES.find((d) => d.id === id)!;
    difficulty = id;
    planner = preset.planner;
    recruiter = preset.recruiter;
  }
  function custom() {
    difficulty =
      DIFFICULTIES.find((d) => d.planner === planner && d.recruiter === recruiter)?.id ?? "custom";
  }
  export function open(tab: "skirmish" | "campaign") {
    mode = tab;
    if (tab === "campaign" && difficulty === "custom") setDifficulty("situational");
    error = "";
    mapId = BATTLE_MAPS.some((m) => m.id === view.game.mapId) ? view.game.mapId : "mirror";
    dialog.showModal();
    preloadTerrainMap(mode === "campaign" ? campaign.missions[0]!.map.id : mapId);
  }
  function requestStart() {
    if (view.game.mapId !== "developer" && !view.game.winner && view.completedTurns > 1) {
      confirmation.showModal();
      confirmation.querySelector<HTMLButtonElement>(".quiet")?.focus();
    } else start();
  }
  function start() {
    confirmation.close();
    const policy = (difficulty === "custom" ? "predictor" : difficulty) as AiOptions["policy"];
    const setup: SessionSetup = {
      first:
        mode === "campaign"
          ? "player"
          : first === "random"
            ? Math.random() < 0.5
              ? "player"
              : "enemy"
            : (first as "player" | "enemy"),
      enemy: {
        policy,
        diplomacy: "diplomatic",
        ...(mode === "skirmish"
          ? {
              blocks: {
                planner: planner as PlannerKind,
                recruiter: recruiter as RecruiterKind,
                campaign: "mission" as const,
              },
            }
          : {}),
      },
    };
    try {
      if (mode === "campaign") actions.startCampaign(campaignId as CampaignId, setup);
      else {
        preloadTerrainMap(mapId);
        actions.reset(mapId, {
          armies: mapId === "developer" ? undefined : [player, opponent],
          setup,
        });
      }
      dialog.close();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<dialog class="new-game-dialog" aria-label="New game" bind:this={dialog}>
  <header class="setup-heading">
    <h2>New game</h2>
    <button class="setup-close" aria-label="Close new game" onclick={() => dialog.close()}>
      ×
    </button>
  </header>
  <nav class="setup-tabs" aria-label="Game mode">
    <button
      aria-pressed={mode === "skirmish"}
      onclick={() => {
        mode = "skirmish";
        error = "";
        preloadTerrainMap(mapId);
      }}
    >
      Skirmish
    </button>
    <button
      aria-pressed={mode === "campaign"}
      onclick={() => {
        mode = "campaign";
        error = "";
        if (difficulty === "custom") setDifficulty("situational");
        preloadTerrainMap(campaign.missions[0]!.map.id);
      }}
    >
      Campaign
    </button>
  </nav>
  {#if mode === "skirmish"}
    <div class="setup-map">
      <MapThumbnail {map} />
      <div>
        <ChoicePicker
          label="Battlefield"
          choices={maps}
          bind:value={mapId}
          onchange={() => preloadTerrainMap(mapId)}
        />
        <p class="setup-meta">
          {map.description}
        </p>
      </div>
    </div>
    {#if mapId !== "developer"}
      <div class="setup-armies">
        {#each [{ id: player, label: "Your faction", side: "player" as const }, { id: opponent, label: "Opponent", side: "enemy" as const }] as faction}
          <div class="setup-army" data-side={faction.side}>
            <span class="setup-label">{faction.label}</span>
            {@html artSprite(
              view.artwork,
              "units",
              army(faction.id).sprite as UnitType,
              0,
              0,
              64,
              64,
              "",
              faction.side,
            )}
            {#if faction.side === "player"}<ChoicePicker
                label="Your faction"
                choices={factions}
                bind:value={player}
              />{:else}<ChoicePicker
                label="Opponent faction"
                choices={factions}
                bind:value={opponent}
                side="enemy"
              />{/if}
          </div>
        {/each}
      </div>
    {/if}
  {:else}
    <span class="setup-label">Your campaign</span>
    <article class="setup-story">
      <div class="setup-story-heading">
        {@html artSprite(view.artwork, "units", campaign.leader, 0, 0, 64, 64)}
        <div>
          <ChoicePicker
            label="Campaign"
            choices={campaigns}
            bind:value={campaignId}
            onchange={() => preloadTerrainMap(campaign.missions[0]!.map.id)}
          />
          <em>{campaign.tagline}</em>
        </div>
      </div>
      <p>{campaign.shortDescription}</p>
      <span class="setup-meta">
        {campaign.missions.length} missions · {campaign.race === "human"
          ? "Human"
          : campaign.race === "skeletons"
            ? "Skeleton"
            : campaign.race === "merfolk"
              ? "Merfolk"
              : "Woodland"} armies
      </span>
    </article>
  {/if}
  <div class="setup-difficulty">
    <span class="setup-label">
      {mode === "skirmish" ? "Opponent difficulty" : "Campaign difficulty"}{difficulty === "custom"
        ? " · Custom"
        : ""}
    </span>
    <div class="setup-segments">
      {#each DIFFICULTIES as d}<button
          aria-pressed={difficulty === d.id}
          onclick={() => setDifficulty(d.id)}
        >
          {d.name}
        </button>{/each}
    </div>
    <p class="setup-meta">
      {difficulty === "classic"
        ? "A forgiving opponent with straightforward plans."
        : difficulty === "situational"
          ? "A balanced opponent that responds to your army."
          : difficulty === "predictor"
            ? "Plans ahead and anticipates the troops you can recruit."
            : "Custom planner and recruiter."}
    </p>
  </div>
  {#if mode === "skirmish"}
    <div class="setup-field">
      <span class="setup-label">First turn</span>
      <ChoicePicker
        label="First turn"
        choices={[
          { value: "random", label: "Random" },
          { value: "player", label: "You" },
          { value: "enemy", label: "Opponent" },
        ]}
        bind:value={first}
      />
    </div>
    <details class="setup-advanced">
      <summary>Advanced options</summary>
      <div class="setup-field">
        <span>Opponent planner</span>
        <ChoicePicker
          label="Opponent planner"
          choices={plannerChoices}
          bind:value={planner}
          onchange={custom}
        />
      </div>
      <div class="setup-field">
        <span>Opponent recruiter</span>
        <ChoicePicker
          label="Opponent recruiter"
          choices={recruiterChoices}
          bind:value={recruiter}
          onchange={custom}
        />
      </div>
    </details>
  {/if}
  {#if error}<p role="alert">{error}</p>{/if}
  <div class="setup-actions">
    <button class="quiet" onclick={() => dialog.close()}>Keep playing</button>
    <button class="primary" onclick={requestStart}>Start {mode}</button>
  </div>
</dialog>
<dialog
  class="replace-game-dialog"
  role="alertdialog"
  aria-labelledby="replace-title"
  aria-describedby="replace-description"
  bind:this={confirmation}
>
  <h2 id="replace-title">Leave your current battle?</h2>
  <p id="replace-description">Your unfinished battle will be lost.</p>
  <div class="setup-actions">
    <button
      class="quiet"
      onclick={() => {
        confirmation.close();
        dialog.close();
      }}
    >
      Keep playing
    </button>
    <button class="primary" onclick={start}>Start new game</button>
  </div>
</dialog>
