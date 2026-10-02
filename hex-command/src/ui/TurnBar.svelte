<script lang="ts">
  import type { BattleView, BattleController } from "../controller/battle";

  interface Props {
    view: BattleView;
    controller: BattleController;
    onSettings: () => void;
  }
  let { view, controller, onSettings }: Props = $props();

  const game = $derived(view.game);
  const isPlayerTurn = $derived(game.turn === "player" && !game.winner);
  const income = $derived(game.income("player"));
  const ownedVillages = $derived(income / 2);
  const totalVillages = $derived(game.tiles.filter((tile) => tile.terrain === "village").length);
  const canUndo = $derived.by(() => {
    // Undo history belongs to the controller; refresh its availability with the view.
    view;
    return controller.canUndo();
  });
  const turnLabel = $derived.by(() => {
    if (game.winner === "player") return "Victory";
    if (game.winner === "enemy") return "Defeat";
    return game.turn === "player" ? "End turn" : "Enemy turn";
  });

  function endTurn(): void {
    void controller.endTurn().catch((error) => controller.error(error));
  }
</script>

<header class="battle-bar" aria-label="Battle controls">
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
    onclick={() => controller.undoMove()}
  >
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M9 4 4 9l5 5M4 9h9a7 7 0 0 1 7 7v4" />
    </svg>
  </button>
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
