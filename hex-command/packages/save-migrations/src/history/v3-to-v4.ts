import type { History } from "../formats/history";
import { readHistory } from "../formats/read";
/** Manual status controls retention; every historical record is a turn snapshot. */
export function migrateHistoryV3ToV4(input: unknown): History<4> {
  const value = readHistory(input, 3);
  return {
    version: 4,
    entries: value.entries.map((old) => ({ ...old, kind: "turn", manual: old.kind === "manual" })),
  };
}
