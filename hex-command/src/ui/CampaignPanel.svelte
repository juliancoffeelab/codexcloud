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
  }: { view: BattleView; actions: BattleCommands; onNewCampaign: () => void } = $props();
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
  const upcoming = $derived(
    mission && campaign
      ? [
          ...new Set(
            mission.waves
              .filter(
                (wave, index) =>
                  !campaign.marks.includes(`wave:${index}`) &&
                  !(
                    wave.requiresUnsealed &&
                    campaign.marks.includes(`sealed:${wave.requiresUnsealed}`)
                  ),
              )
              .map((wave) => `Turn ${wave.round}: ${wave.label}`),
          ),
        ]
      : [],
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
    } catch (error) {
      actions.error(error);
    }
  }
</script>

{#if campaign && definition && mission}
  <section class="campaign-panel" aria-label="Campaign objectives">
    <div class="mission-heading">
      <span>{definition.name}</span>
      <span>Mission {campaign.mission + 1} / {definition.missions.length}</span>
    </div>
    <h2>{mission.map.name}</h2>
    {#if !view.game.winner}
      <div class="mission-goals">
        {#each goals as goal}
          <p class:completed-goal={goal.done}>
            <span aria-hidden="true">{goal.done ? "✓" : "◇"}</span>
            {goal.label}
            <strong>{goal.current} / {goal.total}</strong>
          </p>
        {/each}
        <p class="mission-warning">
          Keep your leader alive{mission.goals.some((goal) => goal.kind === "escort")
            ? " and protect the courier"
            : ""}.
        </p>
      </div>
      <details class="mission-briefing">
        <summary>Briefing and side quests</summary>
        <p>{mission.briefing}</p>
        {#each mission.events.filter((event) => event.label) as event}<p>
            {campaign.marks.includes(`event:${event.id}`) ? "✓ " : ""}{event.label}{event.gold
              ? ` (+${event.gold} gold)`
              : ""}
          </p>{/each}
        {#each upcoming as warning}<p>{warning}</p>{/each}
        <p>Gold comes from villages. Marked objectives have a broken gold border.</p>
      </details>
    {:else if !dialoguePending(campaign)}
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
          <button class="primary" onclick={() => actions.retryMission()}>Retry mission</button>
        {:else}
          <button class="primary" onclick={onNewCampaign}>Choose another campaign</button>
        {/if}
      </div>
    {/if}
  </section>
{/if}
