import { isRecord } from "../../schema";
/** Manual status controls retention, while all historical records use turn snapshots. */
export function migrateHistoryV3ToV4(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 3 || !Array.isArray(value.entries))
    throw new Error("Invalid version-three history index");
  return {
    version: 4,
    entries: value.entries.map((old: unknown) => {
      if (!isRecord(old) || (old.kind !== "turn" && old.kind !== "manual"))
        throw new Error("Invalid historical index entry");
      return { ...old, kind: "turn", manual: old.kind === "manual" };
    }),
  };
}
