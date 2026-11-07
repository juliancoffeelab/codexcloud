import type { History } from "../formats/history";
import { readHistory } from "../formats/read";
/** Index v3 allows manual checkpoints alongside turn starts. */
export function migrateHistoryV2ToV3(input: unknown): History<3> {
  const value = readHistory(input, 2);
  return { version: 3, entries: value.entries };
}
