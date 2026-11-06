import { migrateHistoryV2ToV3 } from "./v2-to-v3";
import { currentHistoryIndex, HISTORY_VERSION, isRecord } from "../../schema";
import { migrateHistoryV1ToV2 } from "./v1-to-v2";
export function migrateHistoryIndex(input: unknown) {
  if (!isRecord(input)) throw new Error("Invalid saved history index");
  let value: unknown = input;
  let removedIds: string[] = [];
  const version = input.version;
  if (version === 1) ({ value, removedIds } = migrateHistoryV1ToV2(input));
  if (isRecord(value) && value.version === 2) value = migrateHistoryV2ToV3(value);
  return { index: currentHistoryIndex(value), migrated: version !== HISTORY_VERSION, removedIds };
}
