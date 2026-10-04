<script lang="ts">
  import { dialoguePending } from "../game/campaign-progress";
  import type { BattleView, BattleCommands } from "../controller/view";
  import { iconMarkup } from "../presentation/icons";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
    onSettings: () => void;
    onBriefing: () => void;
    briefingExpanded: boolean;
  }
  let { view, actions, onSettings, onBriefing, briefingExpanded }: Props = $props();

  const game = $derived(view.game);
  const isShowcase = $derived(game.mapId === "developer");
  const isPlayerTurn = $derived(
    game.turn === "player" && !game.winner && !dialoguePending(game.campaign),
  );
  const income = $derived(game.income("player"));
  const ownedVillages = $derived(income / 2);
  const totalVillages = $derived(game.tiles.filter((tile) => tile.terrain === "village").length);
  const canUndo = $derived(view.canUndo);
  const turnLabel = $derived.by(() => {
    if (game.winner === "player") return "Victory";
    if (game.winner === "enemy") return "Defeat";
    return game.turn === "player" ? "End turn" : "Enemy turn";
  });

  function endTurn(): void {
    void actions.endTurn().catch((error) => actions.error(error));
  }
</script>

<header class="battle-bar" class:campaign-controls={!!game.campaign} aria-label="Battle controls">
  {#if isShowcase}
    <span class="showcase-title">Developer showcase</span>
    <button id="reset-showcase" onclick={() => actions.reset("developer")}>Reset showcase</button>
  {:else}
    <div class="header-economy">
      <span class="treasury" aria-label="Gold">
        <i class="coin" aria-hidden="true"></i>
        <b id="gold">{game.gold.player}</b>
      </span>
      <span class="income" title="Coins earned each turn">
        <b id="income">+{income}</b>
        <span>/turn</span>
      </span>
      <span class="villages" title="Owned villages">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="m3 11 9-8 9 8M5 10v11h14V10M10 21v-7h4v7" />
        </svg>
        <!-- Keep the count adjacent: its displayed text is 0/4, without added spaces. -->
        <!-- prettier-ignore -->
        <span><b id="villages">{ownedVillages}</b>/<span id="village-total">{totalVillages}</span></span>
      </span>
    </div>
    <button id="end-turn" class="primary" disabled={!isPlayerTurn} onclick={endTurn}>
      <span id="turn-label">{turnLabel}</span>
      <span id="round" class="round" title={`Round ${game.round}`}>Turn {game.round}</span>
    </button>
    <button
      id="undo-move"
      class="icon-button"
      aria-label="Undo move"
      title="Undo move"
      disabled={!canUndo}
      onclick={() => actions.undoMove()}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M9 4 4 9l5 5M4 9h9a7 7 0 0 1 7 7v4" />
      </svg>
    </button>
  {/if}
  {#if game.campaign}
    <button
      id="mission-briefing-button"
      class="icon-button"
      aria-label="Mission briefing"
      title="Mission briefing"
      aria-expanded={briefingExpanded}
      aria-controls="mission-overview"
      onclick={onBriefing}
    >
      {@html iconMarkup("search")}
    </button>
  {/if}
  <button
    id="settings-button"
    class="icon-button"
    aria-label="Settings"
    title="Settings"
    onclick={onSettings}
  >
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="m9 3 1-1h4l1 3 3 1 3-1 2 4-2 2v3l2 2-2 4-3-1-3 1-1 3h-4l-1-3-3-1-3 1-2-4 2-2v-3L1 9l2-4 3 1 3-1z"
      />
      <circle cx="12" cy="12" r="3" />
    </svg>
  </button>
</header>

<style>
  .showcase-title {
    margin-right: auto;
    font:
      14px system-ui,
      sans-serif;
    color: var(--muted);
  }
  #reset-showcase {
    flex-shrink: 0;
  }
</style>
