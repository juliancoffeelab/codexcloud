import { isRecord, type BattleSave } from "../schema";

/** Keep resolved state, never deserialize historical planner implementations. */
export function migrateV4ToV5(value: Record<string, unknown>): BattleSave {
  const checkpoint = value.checkpoint;
  if (!isRecord(checkpoint) || !isRecord(checkpoint.state))
    throw new Error("Invalid version-four native save");
  return {
    version: 5,
    pendingEnemy: value.pendingEnemy as boolean,
    checkpoint: {
      seed: checkpoint.seed,
      state: checkpoint.state,
    } as unknown as BattleSave["checkpoint"],
  };
}
