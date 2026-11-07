import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Frozen Stage 1 assignment: preserve the battle while replacing the permanent leader post. */
export function migrateV14ToV15(input: unknown): Save<15> {
  const value = readSave(input, 14);
  if (value.version !== 14 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-fourteen save");
  const old = value.checkpoint.state;
  if (old.mapId !== "lantern-villages") return { ...value, version: 15 };
  if (!Array.isArray(old.units) || !old.units.every(isRecord))
    throw new Error("Invalid historical roadside roster");
  const units = old.units.map((unit) =>
    unit.name === "The Tollkeeper" && unit.side === "enemy" && unit.role === "leader"
      ? { ...unit, aiGroup: "tollkeeper" }
      : unit,
  );
  return {
    ...value,
    version: 15,
    checkpoint: { ...value.checkpoint, state: { ...old, units } },
  };
}
