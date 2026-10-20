import { MAPS } from "../../../game/maps";
import { TYPES } from "../../../game/catalog";
import { TERRAIN_ORDER } from "../../../game/authored-content";
import type { GameState, MapId, Tile, Unit } from "../../../game/types";
import { LEGACY_ASHEN_REFUGE } from "./layouts";

/** Reconstructed data plus lookup helpers; deliberately has no gameplay commands. */
export interface LegacyBoard extends GameState {
  tile(id: string): Tile | undefined;
  at(id: string): Unit | undefined;
  cost(unit: Pick<Unit, "type">, tile: Tile): number;
  addUnit(type: Unit["type"], side: Unit["side"], tile: string): Unit;
  record(message: string): void;
}

export function legacyBoard(mapId = "mirror", revision: unknown = 1): LegacyBoard {
  if (!Object.hasOwn(MAPS, mapId)) throw new Error("Unknown legacy battlefield");
  const current = MAPS[mapId as MapId];
  if (revision !== 1 && revision !== (current.revision ?? 1))
    throw new Error("Unknown legacy battlefield revision");
  const layout = mapId === "ashen-refuge" && revision === 1 ? LEGACY_ASHEN_REFUGE : current;
  const tiles: Tile[] = Array.from({ length: layout.cols * layout.rows }, (_, index) => {
    const col = index % layout.cols,
      row = Math.floor(index / layout.cols);
    return { id: `${col},${row}`, col, row, terrain: "plains", owner: null };
  });
  const byId = new Map(tiles.map((tile) => [tile.id, tile]));
  for (const { col, row, terrain, owner = null } of layout.terrain)
    Object.assign(byId.get(`${col},${row}`)!, { terrain, owner });
  const board: LegacyBoard = {
    mapId: mapId as MapId,
    mapRevision: revision as number,
    cols: layout.cols,
    rows: layout.rows,
    round: 1,
    turn: "player",
    gold: { player: 12, enemy: 12 },
    winner: null,
    nextId: 1,
    log: [],
    units: [],
    tiles,
    tile: (id) => byId.get(id),
    at: (id) => board.units.find((unit) => unit.tile === id),
    cost: (unit, tile) => TYPES[unit.type].costs[TERRAIN_ORDER.indexOf(tile.terrain)] ?? Infinity,
    addUnit(type, side, tile) {
      const unit: Unit = {
        id: `u${board.nextId++}`,
        type,
        side,
        tile,
        hp: TYPES[type].hp,
        moved: false,
        attacked: false,
        experience: 0,
      };
      board.units.push(unit);
      return unit;
    },
    record(message) {
      board.log = [message, ...board.log].slice(0, 8);
    },
  };
  return board;
}

/** Remove the importer-only helpers before crossing the native checkpoint boundary. */
export function boardState(board: LegacyBoard): GameState {
  const { tile: _tile, at: _at, cost: _cost, addUnit: _addUnit, record: _record, ...state } = board;
  return state;
}
