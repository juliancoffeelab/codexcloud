import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import historicalLeaderTypes from "./legacy/data/type-leaders-v5.json";

/** Convert the old implicit archetype rule into explicit unit roles. */
export function migrateV5ToV6(input: unknown): Save<6> {
  const value = readSave(input, 5);
  const checkpoint = value.checkpoint;
  if (
    !isRecord(checkpoint) ||
    !isRecord(checkpoint.state) ||
    !Array.isArray(checkpoint.state.units)
  )
    throw new Error("Invalid version-five native save");
  const units: import("./formats/history").Unit<6>[] = checkpoint.state.units.map((unit) =>
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
  };
}
