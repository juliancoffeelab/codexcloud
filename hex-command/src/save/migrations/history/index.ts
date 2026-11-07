import { migrateHistoryIndex as convert } from "@hex-command/save-migrations/history";
import { currentHistoryIndex } from "../../schema";
export function migrateHistoryIndex(input: unknown) {
  const result = convert(input);
  return { ...result, index: currentHistoryIndex(result.index) };
}
