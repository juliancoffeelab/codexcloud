import { TYPES, TERRAIN } from "../game/catalog";
import type { LiveGame } from "../game/live-game";

export interface SaveSummary {
  gold: number;
  villages: { owned: number; total: number };
  objectives: { completed: number; total: number } | null;
  units: { level: number; count: number }[];
}
/** Read resolved player facts from the selected saved battle, without advancing it. */
export function summarizeSave(game: LiveGame): SaveSummary {
  const armies = new Set([
    "player",
    ...game.factions.filter((f) => f.controller.kind === "human").map((f) => f.id),
  ]);
  const levels = new Map([
    [1, 0],
    [2, 0],
    [3, 0],
  ]);
  for (const unit of game.units) {
    if (!game.faction(unit.id).controllable) continue;
    const level = TYPES[unit.type].level;
    if (level === undefined) throw new Error("Native unit level is missing.");
    levels.set(level, (levels.get(level) ?? 0) + 1);
  }
  const villages = game.tiles.filter((tile) => TERRAIN[tile.terrain].role === "village");
  const objectives = game.objectives
    ? [...game.objectives.goals, ...game.objectives.optional].filter((o) => o.visible)
    : null;
  return {
    gold: [...armies].reduce((sum, id) => sum + (game.gold[id] ?? 0), 0),
    villages: {
      owned: villages.filter((tile) => armies.has(tile.owner ?? "")).length,
      total: villages.length,
    },
    objectives: objectives
      ? { completed: objectives.filter((o) => o.achieved).length, total: objectives.length }
      : null,
    units: [...levels].sort(([a], [b]) => a - b).map(([level, count]) => ({ level, count })),
  };
}
