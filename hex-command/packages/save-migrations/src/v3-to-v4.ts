import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { MAPS } from "./legacy/frozen";
import { isRecord } from "./formats/envelope-v39";

/** Grow the two published developer layouts, preserving owned tiles and battle progress. */
export function migrateV3ToV4(input: unknown): Save<4> {
  const value = readSave(input, 3);
  const checkpoint = value.checkpoint;
  if (!isRecord(checkpoint) || !isRecord(checkpoint.state))
    throw new Error("Invalid version-three native save");
  const state = checkpoint.state;
  if (state.mapId !== "developer" || state.mapRevision !== 1) return { ...value, version: 4 };

  const { cols, rows, tiles } = state;
  if (!((cols === 6 && rows === 20) || (cols === 13 && rows === 23)) || !Array.isArray(tiles))
    throw new Error("Unknown historical developer geometry");
  const oldTiles = new Map<string, import("./formats/history").Tile<3>>();
  for (const tile of tiles) {
    if (
      !isRecord(tile) ||
      !Number.isInteger(tile.col) ||
      !Number.isInteger(tile.row) ||
      Number(tile.col) < 0 ||
      Number(tile.col) >= Number(cols) ||
      Number(tile.row) < 0 ||
      Number(tile.row) >= Number(rows) ||
      tile.id !== `${tile.col},${tile.row}` ||
      oldTiles.has(String(tile.id))
    )
      throw new Error("Invalid historical developer tile");
    oldTiles.set(String(tile.id), tile);
  }
  if (oldTiles.size !== Number(cols) * Number(rows))
    throw new Error("Incomplete historical developer map");

  const map = MAPS.developer;
  const terrain = new Map(map.terrain.map((tile) => [`${tile.col},${tile.row}`, tile]));
  const expanded = Array.from({ length: map.cols * map.rows }, (_, index) => {
    const col = index % map.cols,
      row = Math.floor(index / map.cols),
      id = `${col},${row}`;
    return oldTiles.get(id) ?? { id, col, row, terrain: "plains", owner: null, ...terrain.get(id) };
  });
  return {
    ...value,
    version: 4,
    checkpoint: {
      ...checkpoint,
      state: {
        ...state,
        mapRevision: map.revision ?? 1,
        cols: map.cols,
        rows: map.rows,
        tiles: expanded,
      },
    },
  };
}
