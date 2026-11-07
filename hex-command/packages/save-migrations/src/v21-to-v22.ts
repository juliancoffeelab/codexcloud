import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Frozen revision-seven settlement reduction; retain the ongoing battle. */
export function migrateV21ToV22(input: unknown): Save<22> {
  const value = readSave(input, 21);
  if (value.version !== 21 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-one save");
  const old = value.checkpoint.state;
  if (old.mapId !== "lantern-villages" || old.mapRevision !== 7) return { ...value, version: 22 };
  if (!Array.isArray(old.tiles) || !old.tiles.every(isRecord))
    throw new Error("Invalid historical roadside map");
  const tiles = old.tiles.map((tile) => {
    if (tile.id !== "13,7" && tile.id !== "16,10") return tile;
    const { connections: _oldConnections, ...rest } = tile;
    return {
      ...rest,
      terrain: tile.id === "13,7" ? "forest" : "marsh",
      floor: tile.id === "13,7" ? "green-forest" : "marsh",
    };
  });
  return {
    ...value,
    version: 22,
    checkpoint: { ...value.checkpoint, state: { ...old, tiles, mapRevision: 8 } },
  };
}
