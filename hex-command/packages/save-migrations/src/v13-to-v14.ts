import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import villages from "./legacy/data/bell-villages-v13.json";

/** Install the three new keep-side villages without resetting an ongoing valley battle. */
export function migrateV13ToV14(input: unknown): Save<14> {
  const value = readSave(input, 13);
  if (value.version !== 13 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirteen save");
  const old = value.checkpoint.state;
  if (old.mapId !== "lantern-bell" || old.mapRevision !== 3) return { ...value, version: 14 };
  if (
    !Array.isArray(old.tiles) ||
    !old.tiles.every(isRecord) ||
    !Array.isArray(old.units) ||
    !old.units.every(isRecord)
  )
    throw new Error("Invalid historical valley map");
  const units = old.units.map((unit) => {
    if (unit.aiGroup !== "garrison") return unit;
    const { aiGroup: _obsolete, ...ordinary } = unit;
    return ordinary;
  });
  const tiles = old.tiles.map((tile) => {
    const occupied = units.some((unit) => unit.side === "player" && unit.tile === tile.id);
    // The published bell tile becomes a capturable keep in this release too.
    if (tile.id === "11,12" && tile.terrain === "castle-bell" && occupied)
      return { ...tile, owner: "player" };
    const village = villages.find((village) => village.id === tile.id);
    if (!village || tile.terrain !== "plains") return tile;
    // A company member already occupying a newly installed village has control.
    return { ...tile, ...village, owner: occupied ? "player" : village.owner };
  });
  return {
    ...value,
    version: 14,
    checkpoint: { ...value.checkpoint, state: { ...old, tiles, units } },
  };
}
