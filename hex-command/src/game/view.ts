import type { LiveGame } from "./live-game";
import type { Immutable } from "./immutable";
import type { Tile, Unit } from "./types";

/** Published data cannot expose mutable engine state or engine commands. */
export type { Immutable } from "./immutable";

export type UnitView = Immutable<Unit>;
export type TileView = Immutable<Tile>;

type ReadableGame = Pick<
  LiveGame,
  | "presentation"
  | "faction"
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
  | "canAttack"
  | "previewDetails"
  | "income"
  | "snapshot"
>;

export type GameView = Immutable<ReadableGame>;
