import type { Game } from "../engine";
import { campaignMission, endsRefugeMediation } from "../campaign-progress";
import type { Unit } from "../types";

export const DIPLOMACY_IDS = ["pragmatic", "restrained"] as const;
export type DiplomacyId = (typeof DIPLOMACY_IDS)[number];

/** A testing preference, never a change to legal attacks or authored allegiance. */
export function protectsWithdrawal(game: Game, side: Unit["side"], mode: DiplomacyId): boolean {
  return Boolean(
    mode === "restrained" &&
    side === "player" &&
    game.campaign &&
    campaignMission(game.campaign).refuge &&
    !game.campaign.refuge?.escalated,
  );
}

export function diplomacyAllowsAttack(
  game: Game,
  unit: Unit,
  target: Unit,
  weapon: string,
  from: string,
  mode: DiplomacyId,
): boolean {
  if (!protectsWithdrawal(game, unit.side, mode)) return true;
  if (target.retreating) return false;
  return (
    !endsRefugeMediation(game, target) ||
    game.preview(unit, target, weapon, from).damage < target.hp
  );
}

/** Do not seal a withdrawing patrol's next legal step toward its home edge/keep. */
export function withdrawalLanes(game: Game, side: Unit["side"], mode: DiplomacyId): Set<string> {
  const lanes = new Set<string>();
  if (!protectsWithdrawal(game, side, mode)) return lanes;
  for (const unit of game.units.filter((other) => other.side !== side && other.retreating)) {
    const homes = game.tiles.filter(
      (tile) => tile.owner === unit.side && ["keep", "village"].includes(tile.terrain),
    );
    const fallback = game.tiles.filter((tile) => tile.col === game.cols - 1);
    const routes = game.routeDistances(unit, homes.length ? homes : fallback);
    const before = routes.get(unit.tile) ?? Infinity;
    for (const tile of game.neighbors(unit.tile))
      if ((routes.get(tile.id) ?? Infinity) < before && Number.isFinite(game.cost(unit, tile)))
        lanes.add(tile.id);
  }
  return lanes;
}
