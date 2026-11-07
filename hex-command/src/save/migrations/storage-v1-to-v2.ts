import {
  migrateStorageV1ToV2 as convert,
  inspectStorageV1ToV2 as inspect,
} from "@hex-command/save-migrations/storage";
import { currentSave, currentFlashSave, currentTurn } from "../schema";
export {
  SAVE_KEY,
  HISTORY_KEY,
  NEW_GAME_KEY,
  historyPayloadKey,
  sourceKey,
  repairStorageV1ToV2,
  type LocalSource,
} from "@hex-command/save-migrations/storage";
import type { BattleSave } from "../schema";
import type { RepairRecord } from "../repair-types";
import type { LocalSource } from "@hex-command/save-migrations/storage";
export function migrateStorageV1ToV2(source: LocalSource) {
  const result = convert(source);
  return {
    ...result,
    current: result.current && currentFlashSave(result.current),
    turns: result.turns.map(currentTurn),
  };
}
export function inspectStorageV1ToV2(
  source: LocalSource,
): (RepairRecord & { save?: BattleSave })[] {
  return inspect(source).map((record) => {
    const { save, ...metadata } = record;
    if (!save) return metadata;
    try {
      return { ...metadata, save: currentSave(save) };
    } catch (error) {
      return {
        ...metadata,
        issue: metadata.issue ?? "format",
        error: [metadata.error, String(error)].filter(Boolean).join("\n"),
      };
    }
  });
}
