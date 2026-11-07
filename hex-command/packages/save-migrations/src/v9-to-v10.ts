import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Freeze the pre-v10 implicit orientations into explicit historical tile data. */
export function migrateV9ToV10(input: unknown): Save<10> {
  const value = readSave(input, 9);
  if (
    value.version !== 9 ||
    !isRecord(value.checkpoint) ||
    !isRecord(value.checkpoint.state) ||
    !Array.isArray(value.checkpoint.state.tiles) ||
    !value.checkpoint.state.tiles.every(isRecord)
  )
    throw new Error("Invalid version-nine save");
  // Historical constants only; never query live terrain rules or authored maps.
  const orientations: Record<string, import("./formats/native-v38").HexDirection[]> = {
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
