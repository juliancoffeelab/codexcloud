import { isRecord, type BattleSave } from "../schema";

/** Rust supplies missing per-attack budgets from the old attacked flag. Keep the
 * pending command and recruitment allowance; recompute the old tactical bound. */
export function migrateV4ToV5(value: Record<string, unknown>): BattleSave {
  const checkpoint = value.checkpoint;
  if (!isRecord(checkpoint) || !isRecord(checkpoint.state))
    throw new Error("Invalid version-four native save");
  const planner = checkpoint.planner;
  if (planner !== undefined && (!isRecord(planner) || !isRecord(planner.workspace)))
    throw new Error("Invalid version-four planner");
  return {
    ...value,
    version: 5,
    checkpoint: {
      ...checkpoint,
      ...(isRecord(planner) && isRecord(planner.workspace)
        ? { planner: { ...planner, workspace: { ...planner.workspace, tacticalRemaining: null } } }
        : {}),
    },
  } as unknown as BattleSave;
}
