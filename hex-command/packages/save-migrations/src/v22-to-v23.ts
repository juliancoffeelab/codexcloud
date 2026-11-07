import type { Save } from "./formats/history";
import { readSave, readFragment } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import frozen from "./legacy/data/tile-lights-v22.json";

/** Move frozen map emitters onto saved tiles and retain remembered activation. */
export function migrateV22ToV23(input: unknown): Save<23> {
  const value = readSave(input, 22);
  if (value.version !== 22 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-two save");
  const old = value.checkpoint.state;
  if (!Array.isArray(old.tiles) || !old.tiles.every(isRecord))
    throw new Error("Invalid historical terrain");
  const map = frozen[old.mapId as keyof typeof frozen];
  const lights = map && old.mapRevision === map.revision ? map.lights : [];
  const tiles = old.tiles.map((tile) => {
    const entry = lights.find((entry) => entry.tile === tile.id);
    return entry ? { ...tile, light: readFragment(structuredClone(entry.light), "light23") } : tile;
  });
  const state: import("./formats/history").State<23> = { ...old, tiles };
  if (old.conditionMemory !== undefined) {
    if (
      !Array.isArray(old.conditionMemory) ||
      !old.conditionMemory.every((key) => typeof key === "string")
    )
      throw new Error("Invalid historical condition memory");
    state.conditionMemory = old.conditionMemory.map((key) => {
      const entry = lights.find((entry) => entry.legacyKey === key);
      return entry ? `light:${entry.tile}` : key;
    });
  }
  return { ...value, version: 23, checkpoint: { ...value.checkpoint, state } };
}
