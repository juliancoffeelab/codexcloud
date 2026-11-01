import { isRecord } from "../schema";

/** Freeze the pre-v10 implicit orientations into explicit historical tile data. */
export function migrateV9ToV10(value: Record<string, unknown>): Record<string, unknown> {
  if (
    value.version !== 9 ||
    !isRecord(value.checkpoint) ||
    !isRecord(value.checkpoint.state) ||
    !Array.isArray(value.checkpoint.state.tiles) ||
    !value.checkpoint.state.tiles.every(isRecord)
  )
    throw new Error("Invalid version-nine save");
  // Historical constants only; never query live terrain rules or authored maps.
  const orientations: Record<string, string[]> = {
    signpost: ["e", "w"],
    road: ["e", "w"],
    bridge: ["e", "w"],
    ford: ["e", "w"],
    village: [],
    landing: ["w"],
    ferry: ["w"],
  };
  return {
    ...value,
    version: 10,
    checkpoint: {
      ...value.checkpoint,
      state: {
        ...value.checkpoint.state,
        tiles: value.checkpoint.state.tiles.map((tile) =>
          tile.connections === undefined &&
          typeof tile.terrain === "string" &&
          Object.hasOwn(orientations, tile.terrain)
            ? { ...tile, connections: [...orientations[tile.terrain]!] }
            : { ...tile },
        ),
      },
    },
  };
}
