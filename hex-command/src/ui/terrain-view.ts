import type { TileView } from "../game/view";
import type { RecruitmentContext, BattleView } from "../controller/view";
import { TERRAIN, TYPES } from "../game/catalog";
import type { TerrainType } from "../game/types";
import { campaignMission } from "../game/campaign-progress";

const descriptions: Record<TerrainType, string> = {
  "burned-village": "Burned homes. No income, healing or protection.",
  marsh: "Mud and reeds expose land troops; merfolk find protection here.",
  ruins: "Broken walls shelter against ranged attacks.",
  fort: "Earthworks protect against every damage kind. Capture them to hold a crossing.",
  grove: "A sacred woodland refuge. Its owner’s occupants heal 4 HP each turn.",
  plains: "Open grassland with easy footing.",
  signpost: "A waymarker beside the road. Follow the scenario objective to leave the area.",
  road: "A winding dirt road through grassland. Normal movement and no protection.",
  forest: "Trees shelter land troops and obstruct ranged shots. Merfolk are exposed here.",
  hill: "High ground shelters land troops and obstructs ranged shots. Merfolk are exposed here.",
  water: "Deep water is impassable to land units. Merfolk can swim through it.",
  bridge: "A sturdy crossing over water with easy footing, but no protection.",
  ford: "Shallow water can be crossed on foot. The open water leaves units exposed.",
  landing: "A wooden jetty connects the river to its bank. Normal movement and no protection.",
  ferry: "A river ferry is moored alongside the landing. Normal movement and no protection.",
  house: "A single riverside home. Provides no income or healing.",
  mountain:
    "A sheer mountain ridge, impassable to land units. Blocks ranged attacks through the tile.",
  cave: "A quarry entrance at the foot of a mountain. Blocks ranged attacks through the tile. Scenario objectives describe what can be found inside.",
  village: "Capture this settlement by moving a unit onto it.",
  keep: "A fortified home for its army’s commander. Blocks ranged attacks through the tile.",
};

function protectionFact(terrain: TerrainType): string {
  if (terrain === "ford") return "Land troops: +1 damage received. Merfolk: no exposure.";
  if (terrain === "marsh") return "Land troops: +1 damage received. Merfolk: −1 damage received.";
  if (terrain === "hill") return "Land troops: −1 damage received. Merfolk: +1 damage received.";
  if (terrain === "fort") return "Protection: −1 damage from all attacks";
  if (terrain === "forest" || terrain === "grove")
    return "Land troops: −1 ranged damage. Merfolk: +1 damage from all attacks.";
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
      "Heals a friendly occupant by 4 HP at the start of its turn",
    );
  }
  if (terrain === "grove")
    facts.push("Heals its owner’s occupant by 4 HP at the start of its turn");
  if (terrain === "keep") {
    facts.push(
      "Its owner’s leader can recruit into adjacent empty terrain passable to the recruit",
      "Select an empty adjacent hex to recruit",
      "Move onto a keep to claim it",
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
  if (!tile || view.game.presentation?.fog.includes(tile.id)) return null;

  const hasOwnership = ["village", "keep", "grove", "ruins", "fort"].includes(tile.terrain);
  let ownerLabel = "Unclaimed";
  if (tile.owner === "player") ownerLabel = "Your";
  if (tile.owner === "enemy") ownerLabel = "Enemy";
  const facts = terrainFacts(tile.terrain);
  const campaign = view.game.campaign;
  const mission = campaign && campaignMission(campaign);
  const seal =
    mission &&
    [...mission.goals, ...(mission.optionalGoals ?? [])].find(
      (goal) => goal.kind === "seal" && goal.tiles.includes(tile.id),
    );
  if (seal?.kind === "seal") {
    facts.push(
      campaign!.marks.includes(`sealed:${tile.id}`)
        ? "Binding stone sealed; it will not raise the dead"
        : `Enter with ${seal.unitType ? `a friendly ${TYPES[seal.unitType].name.toLowerCase()}` : "a friendly unit"} to seal this binding stone`,
    );
  }

  return {
    tile,
    name: TERRAIN[tile.terrain].name,
    description: descriptions[tile.terrain],
    ownership: hasOwnership
      ? { label: `${ownerLabel} ${tile.terrain}`, className: tile.owner || "neutral" }
      : null,
    facts,
    recruitment: recruitmentView(view, tile, view.recruitment),
  };
}
