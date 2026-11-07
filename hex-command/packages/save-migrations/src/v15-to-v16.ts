import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Correct the published village floors without resetting captures or battle progress. */
export function migrateV15ToV16(input: unknown): Save<16> {
  const value = readSave(input, 15);
  if (value.version !== 15 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-fifteen save");
  const old = value.checkpoint.state;
  if (old.mapId !== "lantern-bell" || old.mapRevision !== 3) return { ...value, version: 16 };
  if (!Array.isArray(old.tiles) || !old.tiles.every(isRecord))
    throw new Error("Invalid historical valley map");
  // Frozen coordinates and prior floor from the v13 village installation.
  const sites = new Set(["9,11", "13,12", "11,15"]);
  const tiles = old.tiles.map((tile) =>
    typeof tile.id === "string" &&
    sites.has(tile.id) &&
    tile.terrain === "village" &&
    tile.floor === "green-plain"
      ? { ...tile, floor: "mountain-rock" }
      : tile,
  );
  return {
    ...value,
    version: 16,
    checkpoint: { ...value.checkpoint, state: { ...old, tiles } },
  };
}
