import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Keep resolved state, never deserialize historical planner implementations. */
export function migrateV4ToV5(input: unknown): Save<5> {
  const value = readSave(input, 4);
  const checkpoint = value.checkpoint;
  if (!isRecord(checkpoint) || !isRecord(checkpoint.state))
    throw new Error("Invalid version-four native save");
  return {
    version: 5,
    pendingEnemy: value.pendingEnemy as boolean,
    checkpoint: {
      seed: checkpoint.seed,
      state: checkpoint.state,
    },
  };
}
