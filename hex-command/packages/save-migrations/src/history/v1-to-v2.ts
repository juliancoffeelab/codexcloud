import type { History } from "../formats/history";
import { readHistory } from "../formats/read";
import { entryInfo } from "../formats/history-info";
/** Retire parked positions, retaining actual turn starts and their identities. */
export function migrateHistoryV1ToV2(input: unknown): { value: History<2>; removedIds: string[] } {
  const value = readHistory(input, 1),
    removedIds: string[] = [];
  const entries: History<2>["entries"] = [];
  for (const entry of value.entries) {
    if (entry.kind !== "turn") {
      removedIds.push(entry.id);
      continue;
    }
    entries.push({
      ...entryInfo(entry),
      kind: "turn",
      ...(entry.parentId === undefined ? {} : { parentId: entry.parentId }),
    });
  }
  return {
    value: { version: 2, entries },
    removedIds: removedIds.filter((id) => !entries.some((entry) => entry.id === id)),
  };
}
