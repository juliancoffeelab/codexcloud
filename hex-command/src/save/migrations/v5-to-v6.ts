import { isRecord, type BattleSave } from "../schema";
import historicalLeaderTypes from "./legacy/data/type-leaders-v5.json";

/** Convert the old implicit archetype rule into explicit unit roles. */
export function migrateV5ToV6(value: Record<string, unknown>): BattleSave {
  const checkpoint = value.checkpoint;
  if (
    !isRecord(checkpoint) ||
    !isRecord(checkpoint.state) ||
    !Array.isArray(checkpoint.state.units)
  )
    throw new Error("Invalid version-five native save");
  const units = checkpoint.state.units.map((unit) =>
    isRecord(unit) &&
    unit.role === undefined &&
    typeof unit.type === "string" &&
    historicalLeaderTypes.includes(unit.type)
      ? { ...unit, role: "leader" }
      : unit,
  );
  return {
    ...value,
    version: 6,
    checkpoint: { ...checkpoint, state: { ...checkpoint.state, units } },
  } as unknown as BattleSave;
}
