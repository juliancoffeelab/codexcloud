import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { MAPS } from "./legacy/frozen";
import type { MapId } from "./legacy/game-types";
import { isRecord } from "./formats/envelope-v39";

/** Old native saves retained historical layouts in the live catalog. Their owned
 * tiles already preserve that geometry; upgrade only the retired revision tag. */
export function migrateV2ToV3(input: unknown): Save<3> {
  const value = readSave(input, 2);
  const checkpoint = value.checkpoint;
  if (!isRecord(checkpoint) || !isRecord(checkpoint.state))
    throw new Error("Invalid version-two native save");
  const state = checkpoint.state;
  const map = typeof state.mapId === "string" ? MAPS[state.mapId as MapId] : undefined;
  const retired = state.mapRevision === 1 && state.mapId === "ashen-refuge";
  if (retired && !map) throw new Error("Unknown historical map");
  return {
    ...value,
    version: 3,
    checkpoint: {
      ...checkpoint,
      state: { ...state, ...(retired ? { mapRevision: map!.revision } : {}) },
    },
  };
}
