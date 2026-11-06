import { isRecord } from "../../schema";

/** Retire parked recoveries and import-generated positions; preserve real turn starts. */
export function migrateHistoryV1ToV2(value: Record<string, unknown>): {
  value: unknown;
  removedIds: string[];
} {
  if (value.version !== 1 || !Array.isArray(value.entries))
    throw new Error("Invalid version-one history index");
  const entries: Record<string, unknown>[] = [],
    removedIds: string[] = [];
  for (const entry of value.entries) {
    if (
      !isRecord(entry) ||
      typeof entry.id !== "string" ||
      !["turn", "recovery", "resume"].includes(String(entry.kind))
    )
      throw new Error("Invalid historical index entry");
    if (entry.kind !== "turn") {
      removedIds.push(entry.id);
      continue;
    }
    entries.push(
      Object.fromEntries(
        [
          "id",
          "runId",
          "branchId",
          "checkpointId",
          "ordinal",
          "savedAt",
          "run",
          "mission",
          "round",
          "side",
          "kind",
          "missionStart",
          "parentId",
        ]
          .filter((k) => Object.hasOwn(entry, k))
          .map((k) => [k, entry[k]]),
      ),
    );
  }
  return {
    value: { version: 2, entries },
    removedIds: removedIds.filter((id) => !entries.some((entry) => entry.id === id)),
  };
}
