<script lang="ts">
  import { dialoguePending } from "../game/campaign-progress";
  import type { BattleView, BattleCommands } from "../controller/view";
  import BattleBar from "./BattleBar.svelte";

  interface Props {
    view: BattleView;
    actions: BattleCommands;
    onSettings: () => void;
    onBriefing: () => void;
    briefingExpanded: boolean;
  }
  let { view, actions, onSettings, onBriefing, briefingExpanded }: Props = $props();

  const game = $derived(view.game);
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

<BattleBar
  gold={game.gold.player}
  {income}
  {ownedVillages}
  {totalVillages}
  round={game.round}
  {turnLabel}
  canEndTurn={isPlayerTurn}
  {canUndo}
  campaign={!!game.campaign}
  {briefingExpanded}
  onEndTurn={endTurn}
  onUndo={() => actions.undoMove()}
  {onSettings}
  {onBriefing}
/>
