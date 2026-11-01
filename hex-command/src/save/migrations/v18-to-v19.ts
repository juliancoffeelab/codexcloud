import { isRecord } from "../schema";
import changes from "./legacy/data/lantern-villages-v18.json";

/** Apply frozen northern hills/keep geometry while preserving an ongoing company. */
export function migrateV18ToV19(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 18 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-eighteen save");
  const state = value.checkpoint.state;
  if (state.mapId !== "lantern-villages" || state.mapRevision !== 4)
    return { ...value, version: 19 };
  if (
    !Array.isArray(state.tiles) ||
    !state.tiles.every(isRecord) ||
    !Array.isArray(state.units) ||
    !state.units.every(isRecord)
  )
    throw new Error("Invalid historical roadside map");
  const oldKeep = state.tiles.find((tile) => tile.id === "16,4");
  const occupant = state.units.find((unit) => unit.tile === "15,7");
  const owner = occupant?.side ?? (oldKeep ? oldKeep.owner : "enemy");
  const tiles = state.tiles.map((tile) => {
    const change = changes.find((candidate) => candidate.id === tile.id);
    if (!change) return tile;
    const { connections: _oldConnections, ...rest } = tile;
    return {
      ...rest,
      ...change,
      owner: change.id === "15,7" ? owner : change.id === "16,4" ? null : tile.owner,
    };
  });
  return {
    ...value,
    version: 19,
    checkpoint: { ...value.checkpoint, state: { ...state, mapRevision: 5, tiles } },
  };
}
