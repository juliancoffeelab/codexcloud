import type { TileView } from "../game/view";
import type { RecruitmentContext, BattleView } from "../controller/view";
import { TERRAIN, TYPES } from "../game/engine";
import type { TerrainType } from "../game/types";

const descriptions: Record<TerrainType, string> = {
  marsh: "Soft mud and reeds surround shallow pools.",
  ruins:
    "Broken walls shelter against ranged attacks. Walk onto a marked ruin to seal its binding stone.",
  fort: "Earthworks protect against every damage kind. Capture them to hold a crossing.",
  grove: "A sacred woodland refuge. Its owner’s occupants heal 2 HP each turn.",
  plains: "Open grassland with easy footing.",
  forest: "Trees shelter units and obstruct ranged shots.",
  hill: "Rocky high ground offers cover and obstructs ranged shots.",
  water: "Deep water cannot be crossed.",
  bridge: "A sturdy crossing over water with easy footing, but no protection.",
  ford: "Shallow water can be crossed on foot. The open water leaves units exposed.",
  village: "Capture this settlement by moving a unit onto it.",
  keep: "A fortified home for its army’s commander.",
};

function protectionFact(terrain: TerrainType): string {
  if (terrain === "ford") return "Exposure: +1 damage received from all attacks";
  if (terrain === "hill" || terrain === "fort") return "Protection: −1 damage from all attacks";
  if (["forest", "grove", "ruins"].includes(terrain))
    return "Protection: −1 damage from ranged attacks";
  return "Protection: none";
}

function terrainFacts(terrain: TerrainType): string[] {
  const facts: string[] = [];
  if (terrain !== "water") facts.push(protectionFact(terrain));
  if (["forest", "hill", "grove"].includes(terrain)) {
    facts.push("Blocks ranged shots through this hex");
  }
  if (terrain === "village") {
    facts.push(
      "+2 gold at the start of its owner’s turn",
      "Heals a friendly occupant by 2 HP at the start of its turn",
    );
  }
  if (terrain === "grove")
    facts.push("Heals its owner’s occupant by 2 HP at the start of its turn");
  if (terrain === "keep") {
    facts.push(
      "Its owner’s leader can recruit into adjacent empty land hexes",
      "Select an empty adjacent hex to recruit",
      "Keeps cannot be captured",
    );
  }
  return facts;
}

function recruitmentView(view: BattleView, tile: TileView, context: RecruitmentContext | null) {
  if (!context) return null;
  const { game } = view;
  const isPlayerTurn = game.turn === "player" && !game.winner;
  const options = game.recruitmentOptions("player").map((type) => ({
    type,
    name: TYPES[type].name,
    cost: TYPES[type].cost,
    hp: TYPES[type].hp,
    move: TYPES[type].move,
    description: TYPES[type].description,
    enabled: Boolean(
      isPlayerTurn &&
      context.stationed &&
      context.commander &&
      game
        .recruitTiles(context.commander.id, type)
        .some((destination) => destination.id === tile.id),
    ),
  }));

  let hint = "New recruits act next turn.";
  if (!context.stationed) hint = "Return your leader to this keep to recruit.";
  else if (!isPlayerTurn) hint = "Recruit on your turn.";
  if (
    isPlayerTurn &&
    context.stationed &&
    game.gold.player < Math.min(...options.map((option) => option.cost))
  ) {
    hint += " Not enough gold.";
  }
  return { options, hint };
}

export function terrainPanelView(view: BattleView) {
  const tile = view.game.tile(view.inspectedTile);
  if (!tile) return null;

  const hasOwnership = ["village", "keep", "grove", "ruins", "fort"].includes(tile.terrain);
  let ownerLabel = "Unclaimed";
  if (tile.owner === "player") ownerLabel = "Your";
  if (tile.owner === "enemy") ownerLabel = "Enemy";

  return {
    tile,
    name: TERRAIN[tile.terrain].name,
    description: descriptions[tile.terrain],
    ownership: hasOwnership
      ? { label: `${ownerLabel} ${tile.terrain}`, className: tile.owner || "neutral" }
      : null,
    facts: terrainFacts(tile.terrain),
    recruitment: recruitmentView(view, tile, view.recruitment),
  };
}
