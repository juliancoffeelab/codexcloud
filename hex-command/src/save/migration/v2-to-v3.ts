import { MAPS } from "../../game/maps";
import type { MapId } from "../../game/types";
import { isRecord, type BattleSave } from "../schema";

/** Old native saves retained historical layouts in the live catalog. Their owned
 * tiles already preserve that geometry; upgrade only the retired revision tag. */
export function migrateV2ToV3(value: Record<string, unknown>): BattleSave {
  const checkpoint = value.checkpoint;
  if (!isRecord(checkpoint) || !isRecord(checkpoint.state))
    throw new Error("Invalid version-two native save");
  const state = checkpoint.state;
  const map = typeof state.mapId === "string" ? MAPS[state.mapId as MapId] : undefined;
  const retired =
    state.mapRevision === 1 && ["river", "ashen-refuge"].includes(String(state.mapId));
  if (retired && !map) throw new Error("Unknown historical map");
  return {
    ...value,
    version: 3,
    checkpoint: {
      ...checkpoint,
      state: { ...state, ...(retired ? { mapRevision: map!.revision } : {}) },
    },
  } as unknown as BattleSave;
}
