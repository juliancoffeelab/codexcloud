import { isRecord } from "../../schema";
/** Index version 3 permits manual checkpoints alongside existing turn starts. */
export function migrateHistoryV2ToV3(value: unknown): Record<string, unknown> {
  if (
    typeof value !== "object" ||
    value === null ||
    !("version" in value) ||
    value.version !== 2 ||
    !("entries" in value) ||
    !Array.isArray(value.entries)
  )
    throw new Error("Invalid version-two history index");
  if (value.entries.some((entry: unknown) => !isRecord(entry) || entry.kind !== "turn"))
    throw new Error("Invalid version-two history entry");
  return { version: 3, entries: value.entries };
}
