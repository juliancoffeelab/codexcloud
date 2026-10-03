<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import { CAMPAIGNS } from "../game/campaign-data";
  import { campaignMission, goalProgress, dialoguePending } from "../game/campaign-progress";
  import { TYPES } from "../game/engine";
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
  const survivors = $derived(
    view.game.units.filter(
      (unit) => unit.side === "player" && !TYPES[unit.type].leader && unit.role !== "courier",
    ),
  );
  const missionMapId = $derived(view.game.mapId);
  $effect(() => {
    missionMapId;
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
      actions.nextMission(recall);
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
    <h3>Hints</h3>
    <ul class="mission-items">
      {#each mission.hints as hint}<li><span>{hint}</span></li>{/each}
    </ul>
    <h3>Objectives</h3>
    <ul class="mission-items mission-goals">
      {#each goals as goal}
        <li class:completed-goal={goal.done}>
          <span>{goal.label}</span>
          <strong>{goal.current}/{goal.total}</strong>
        </li>
      {/each}
      <li>
        <span>
          Keep your leader alive{mission.goals.some((goal) => goal.kind === "escort")
            ? " and protect the courier"
            : ""}.
        </span>
      </li>
      {#each mission.events.filter((event) => event.label) as event}
        <li class:completed-goal={campaign.marks.includes(`event:${event.id}`)}>
          <span>
            {event.label}{event.gold ? ` (+${event.gold} gold)` : ""}
            <span class="optional-label">[optional]</span>
          </span>
        </li>
      {/each}
    </ul>
  </section>
  {#if view.game.winner && !dialoguePending(campaign)}
    <section class="campaign-panel" aria-label="Mission result">
      <h2>{mission.map.name}</h2>

      <div class="mission-debrief">
        <h3>
          {view.game.winner === "enemy"
            ? "The company must try again"
            : campaign.completed
              ? "Campaign complete"
              : "The company regroups"}
        </h3>
        <p>{campaign.outcome}</p>
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
                {@html artSprite(view.artStyle, "units", unit.type, 0, 0, 36, 44)}
                <span>
                  <strong>{unit.name ?? TYPES[unit.type].name}</strong>
                  <span>{unit.hp} / {TYPES[unit.type].hp} HP</span>
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
