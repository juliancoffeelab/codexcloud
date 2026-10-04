import { Game } from "./engine";
import { immutable as freeze, type Immutable } from "./immutable";
import type { Tile, Unit } from "./types";

/** Published data cannot expose mutable engine state or engine commands. */
export type { Immutable } from "./immutable";

export type UnitView = Immutable<Unit>;
export type TileView = Immutable<Tile>;

type ReadableGame = Pick<
  Game,
  | "campaign"
  | "recruitmentOptions"
  | "promotionOptions"
  | "mapId"
  | "mapRevision"
  | "cols"
  | "rows"
  | "round"
  | "turn"
  | "gold"
  | "winner"
  | "log"
  | "tiles"
  | "units"
  | "tile"
  | "unit"
  | "at"
  | "neighbors"
  | "cost"
  | "reachable"
  | "targets"
  | "recruitTiles"
  | "canAttackFrom"
  | "previewDetails"
  | "income"
  | "snapshot"
>;

export type GameView = Immutable<ReadableGame>;

/** One detached, immutable revision. Only pure rule queries cross this boundary. */
export function createGameView(source: Game): GameView {
  const game = new Game(source.mapId, source.mapRevision);
  game.restoreState(source.captureState());
  freeze(game.gold);
  freeze(game.log);
  freeze(game.tiles);
  freeze(game.units);
  freeze(game.lastCombat);
  freeze(game.campaign);
  Object.freeze(game);

  return Object.freeze({
    campaign: game.campaign,
    recruitmentOptions: (side) => freeze(game.recruitmentOptions(side)),
    promotionOptions: (id) => freeze(game.promotionOptions(id)),
    mapId: game.mapId,
    mapRevision: game.mapRevision,
    cols: game.cols,
    rows: game.rows,
    round: game.round,
    turn: game.turn,
    gold: freeze(game.gold),
    winner: game.winner,
    log: freeze(game.log),
    tiles: freeze(game.tiles),
    units: freeze(game.units),
    tile: (id) => game.tile(id),
    unit: (id) => game.unit(id),
    at: (id) => game.at(id),
    neighbors: (id) => freeze(game.neighbors(id)),
    cost: (unit, tile) => game.cost(unit, tile),
    // Each result is independent; changing it cannot alter the published revision.
    reachable: (id) => game.reachable(id),
    targets: (id) => freeze(game.targets(id)),
    recruitTiles: (id, type) => freeze(game.recruitTiles(id, type)),
    canAttackFrom: (unit, from, target, attackId) =>
      game.canAttackFrom(unit, from, target, attackId),
    // Weapons are shared rule definitions inside the engine; detach them as well.
    previewDetails: (attacker, target, attackId, from) =>
      freeze(structuredClone(game.previewDetails(attacker, target, attackId, from))),
    income: (side) => game.income(side),
    snapshot: () => freeze(game.snapshot()),
  } satisfies GameView);
}
