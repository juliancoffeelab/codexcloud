<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import { CAMPAIGNS } from "../game/campaign-data";
  import {
    campaignMission,
    goalProgress,
    dialoguePending,
    returnsWithStory,
  } from "../game/campaign-progress";
  import { TYPES, isLeader } from "../game/catalog";
  import { artSprite } from "../presentation/art";
  let {
    view,
    actions,
    onNewCampaign,
    expanded = $bindable(false),
  }: {
    view: BattleView;
    actions: BattleCommands;
    onNewCampaign: () => void;
    expanded?: boolean;
  } = $props();
  let recall = $state<string[]>([]);
  const campaign = $derived(view.game.campaign);
  const definition = $derived(campaign ? CAMPAIGNS[campaign.id] : null);
  const mission = $derived(campaign ? campaignMission(campaign) : null);
  const goals = $derived(mission?.goals.map((goal) => goalProgress(view.game, goal)) ?? []);
  const optionalGoals = $derived(
    mission?.optionalGoals?.map((goal) => goalProgress(view.game, goal)) ?? [],
  );
  const survivors = $derived(
    view.game.units.filter(
      (unit) =>
        unit.side === "player" &&
        !isLeader(unit) &&
        unit.role !== "courier" &&
        !mission?.requiredCompanions?.includes(unit.name ?? "") &&
        !(campaign && returnsWithStory(campaign, unit)),
    ),
  );
  const missionMapId = $derived(view.game.mapId);
  const winner = $derived(view.game.winner);
  $effect(() => {
    missionMapId;
    winner;
    recall = [];
  });
  function toggle(id: string) {
    recall = recall.includes(id)
      ? recall.filter((value) => value !== id)
      : recall.length < 4
        ? [...recall, id]
        : recall;
  }
  function next() {
    try {
      actions.nextMission(recall.filter((id) => survivors.some((unit) => unit.id === id)));
      expanded = false;
    } catch (error) {
      actions.error(error);
    }
  }
  function retry() {
    actions.retryMission();
    expanded = false;
  }
</script>

{#if campaign && definition && mission}
  <section
    id="mission-overview"
    class="campaign-panel mission-overview"
    hidden={!expanded}
    aria-label="Mission briefing"
  >
    <h2>{mission.map.name}</h2>
    <p class="mission-heading">
      {definition.name}
      {campaign.mission + 1}/{definition.missions.length}
    </p>
    <h3>Briefing</h3>
    <p>{mission.briefing}</p>
    {#if mission.hints.length}
      <h3>Hints</h3>
      <ul class="mission-items">
        {#each mission.hints as hint}<li><span>{hint}</span></li>{/each}
      </ul>
    {/if}
    <h3>Objectives</h3>
    <ul class="mission-items mission-goals">
      {#each goals as goal}
        <li class:completed-goal={goal.done}>
          <span>{goal.label}</span>
          {#if goal.total > 1}
            <strong>{goal.current}/{goal.total}</strong>
          {:else if goal.done}
            <strong>✓</strong>
          {/if}
        </li>
      {/each}
      <li class:completed-goal={view.game.winner === "player"}>
        <span>
          Keep your leader alive{mission.goals.some((goal) => goal.kind === "escort") &&
          !mission.requiredCompanions?.length
            ? " and protect the courier"
            : ""}.
        </span>
        {#if view.game.winner === "player"}<strong>✓</strong>{/if}
      </li>
      {#each mission.requiredCompanions ?? [] as companion}
        <li class:completed-goal={view.game.winner === "player"}>
          <span>Keep {companion} alive.</span>
          {#if view.game.winner === "player"}<strong>✓</strong>{/if}
        </li>
      {/each}
    </ul>
    {#if optionalGoals.length}
      <h3>Optional objectives</h3>
      <ul class="mission-items mission-goals">
        {#each optionalGoals as goal}
          <li class:completed-goal={goal.done}>
            <span>{goal.label}</span>
            {#if goal.total > 1}
              <strong>{goal.current}/{goal.total}</strong>
            {:else if goal.done}
              <strong>✓</strong>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
  {#if view.game.winner && !dialoguePending(campaign)}
    <section id="mission-result" class="campaign-panel" aria-label="Mission result" tabindex="-1">
      <h2>{mission.map.name}</h2>

      <div class="mission-debrief">
        <h3>
          {view.game.winner === "enemy"
            ? "The company must try again"
            : campaign.completed
              ? "Campaign complete"
              : "The company regroups"}
        </h3>
        <p>
          {view.game.winner === "player" ? (mission.debrief ?? campaign.outcome) : campaign.outcome}
        </p>
        {#if view.game.winner === "player" && !campaign.completed}
          <p>
            Choose up to four survivors for the next mission. They return at full health. Carrying {Math.min(
              8,
              view.game.gold.player,
            )} gold.
          </p>
          <div class="recall-list">
            {#each survivors as unit}
              <label class="recall-choice" class:selected={recall.includes(unit.id)}>
                <input
                  type="checkbox"
                  checked={recall.includes(unit.id)}
                  disabled={!recall.includes(unit.id) && recall.length >= 4}
                  onchange={() => toggle(unit.id)}
                />
                {@html artSprite(view.artwork, "units", unit.type, 0, 0, 36, 44)}
                <span>
                  <strong>{unit.name ?? TYPES[unit.type].name}</strong>
                  <span>{unit.hp} / {TYPES[unit.type].hp} HP</span>
                  <span>
                    Level {TYPES[unit.type].level}
                    {#if TYPES[unit.type].experienceNeeded}
                      · {unit.experience ?? 0} / {TYPES[unit.type].experienceNeeded} XP
                    {:else}
                      · Fully advanced
                    {/if}
                  </span>
                </span>
              </label>
            {/each}
          </div>
          <button class="primary" onclick={next}>
            Continue to {definition.missions[campaign.mission + 1]?.map.name}
          </button>
        {:else if view.game.winner === "enemy"}
          <button class="primary" onclick={retry}>Retry mission</button>
        {:else}
          <button class="primary" onclick={onNewCampaign}>Choose another campaign</button>
        {/if}
      </div>
    </section>
  {/if}
{/if}
