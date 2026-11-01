import type { TileView } from "../game/view";
import type { RecruitmentContext, BattleView } from "../controller/view";
import { TERRAIN, TYPES } from "../game/catalog";
import type { TerrainType } from "../game/types";
import { campaignMission } from "../game/campaign-progress";

function protectionFact(terrain: TerrainType): string {
  const { role, damageModifiers: modifiers } = TERRAIN[terrain];
  if (role === "ford") return "Land troops: +1 damage received. Merfolk: no exposure.";
  if (role === "marsh") return "Land troops: +1 damage received. Merfolk: −1 damage received.";
  if (role === "hill") return "Land troops: −1 damage received. Merfolk: +1 damage received.";
  if (role === "forest" || role === "grove")
    return "Land troops: −1 ranged damage. Merfolk: +1 damage from all attacks.";
  if (modifiers?.melee === -1 && modifiers?.ranged === -1)
    return "Protection: −1 damage from all attacks";
  const effects = Object.entries(modifiers ?? {})
    .filter(([, value]) => value !== 0)
    .map(([kind, value]) => `${value! > 0 ? "+" : ""}${value} ${kind} damage`);
  return effects.length ? `Protection: ${effects.join(", ")}` : "Protection: none";
}

function terrainFacts(terrain: TerrainType): string[] {
  const role = TERRAIN[terrain].role;
  const facts: string[] = [];
  if (role !== "water") facts.push(protectionFact(terrain));
  if (["forest", "hill", "grove", "mountain", "cave", "keep"].includes(role)) {
    facts.push("Blocks ranged shots through this hex");
  }
  if (role === "village") {
    facts.push(
      "+2 gold at the start of its owner’s turn",
      "Heals a friendly occupant by 4 HP at the start of its turn",
    );
  }
  if (role === "grove") facts.push("Heals its owner’s occupant by 4 HP at the start of its turn");
  if (role === "keep") {
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

  const hasOwnership =
    TERRAIN[tile.terrain].role === "keep" ||
    ["village", "grove", "ruins", "fort", "monastery"].includes(TERRAIN[tile.terrain].role);
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
    description: TERRAIN[tile.terrain].description,
    ownership: hasOwnership
      ? { label: `${ownerLabel} ${tile.terrain}`, className: tile.owner || "neutral" }
      : null,
    facts,
    recruitment: recruitmentView(view, tile, view.recruitment),
  };
}
