<script lang="ts">
  import type { BattleView, BattleCommands } from "../controller/view";
  import { CAMPAIGNS } from "../game/campaign-data";
  import { campaignMission, dialoguePending, returnsWithStory } from "../game/campaign-progress";
  import { TYPES, isLeader } from "../game/catalog";
  import { artSprite } from "../presentation/art";
  import { pixelSprites } from "../presentation/pixel-art";
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
  const goals = $derived(
    [...(view.game.objectives?.goals ?? []), ...(view.game.objectives?.survival ?? [])]
      .filter((status) => status.visible)
      .map((status) => ({ ...status.progress, failed: status.failed })),
  );
  const optionalGoals = $derived(
    (view.game.objectives?.optional ?? [])
      .filter((status) => status.visible)
      .map((status) => ({
        ...status.progress,
        failed: status.failed,
      })),
  );
  const survivors = $derived(
    view.game.units.filter(
      (unit) =>
        unit.side === "player" &&
        !isLeader(unit) &&
        unit.role !== "courier" &&
        !mission?.requiredCompanions?.includes(unit.character ?? "") &&
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
  {#if !view.game.winner}{#each view.game.objectives?.notices ?? [] as notice}<p
        class="recruit-hint"
        role="status"
      >
        {notice}
      </p>{/each}{/if}
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
        <li
          class:completed-goal={goal.done}
          class:failed-goal={goal.failed}
          aria-label={goal.failed
            ? `${goal.label}: failed`
            : goal.done
              ? `${goal.label}: completed`
              : undefined}
        >
          <span>{goal.label}</span>
          {#if goal.total > 1}
            <strong>{goal.current}/{goal.total}</strong>
          {/if}
        </li>
      {/each}
    </ul>
    {#if optionalGoals.length}
      <h3>Optional objectives</h3>
      <ul class="mission-items mission-goals">
        {#each optionalGoals as goal}
          <li
            class:completed-goal={goal.done}
            class:failed-goal={goal.failed}
            aria-label={goal.failed
              ? `${goal.label}: failed`
              : goal.done
                ? `${goal.label}: completed`
                : undefined}
          >
            <span>{goal.label}</span>
            {#if goal.total > 1}
              <strong>{goal.current}/{goal.total}</strong>
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
            )} gold. Equipment from survivors left behind goes to your leader.
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
                <span class="recall-copy">
                  <strong class="recall-name">
                    {unit.name ?? TYPES[unit.type].name}
                    {#each unit.items ?? [] as item, index (`${item.id}-${index}`)}
                      {#if pixelSprites[`terrain_prop_items_${item.id}`]}
                        <img
                          class="recall-item pixel-art"
                          src={pixelSprites[`terrain_prop_items_${item.id}`]}
                          alt={item.name}
                          title={item.name}
                          width="32"
                          height="32"
                        />
                      {:else}
                        <small>{item.name}</small>
                      {/if}
                    {/each}
                  </strong>
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

<style>
  .recall-copy {
    min-width: 0;
  }
  .recall-name {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    min-height: 32px;
  }
  .recall-item {
    flex: 0 0 32px;
    object-fit: contain;
  }
</style>
