import type { Save } from "./formats/history";
import { readSave, readFragment } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import changes from "./legacy/data/lantern-villages-v19.json";

/** Reroute the eastern road using frozen terrain; preserve the ongoing company. */
export function migrateV19ToV20(input: unknown): Save<20> {
  const value = readSave(input, 19);
  if (value.version !== 19 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-nineteen save");
  const state = value.checkpoint.state;
  if (state.mapId !== "lantern-villages" || state.mapRevision !== 5)
    return { ...value, version: 20 };
  if (!Array.isArray(state.tiles) || !state.tiles.every(isRecord))
    throw new Error("Invalid historical roadside map");
  const tiles = state.tiles.map((tile) => {
    const change = readFragment(changes, "delta20").find((candidate) => candidate.id === tile.id);
    if (!change) return tile;
    const { connections: _oldConnections, ...rest } = tile;
    return { ...rest, ...change };
  });
  return {
    ...value,
    version: 20,
    checkpoint: { ...value.checkpoint, state: { ...state, mapRevision: 6, tiles } },
  };
}
