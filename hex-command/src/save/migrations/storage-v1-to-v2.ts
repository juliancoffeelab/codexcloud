import { migrateSave } from "./index";
import { migrateHistoryIndex } from "./history";
import { currentFlashSave, currentTurn, isRecord } from "../schema";

/** Storage v1: localStorage's envelope and index. Storage v2: transactional IDB records. */
export const SAVE_KEY = "hex-command-battle";
export const HISTORY_KEY = "hex-command-progress-index";
export const NEW_GAME_KEY = "hex-command-choose-new-game";
export const historyPayloadKey = (id: string) => `hex-command-progress:${id}`;
export const sourceKey = (key: string) =>
  [SAVE_KEY, HISTORY_KEY, NEW_GAME_KEY, `${SAVE_KEY}-unrestored`].includes(key) ||
  key.startsWith("hex-command-progress:");
export type LocalSource = Record<string, string | null>;

/** Version 37 was deployed in localStorage with the former current-save label. */
function importedSave(raw: string) {
  let value: unknown = JSON.parse(raw);
  if (
    isRecord(value) &&
    value.version === 37 &&
    isRecord(value.progress) &&
    isRecord(value.progress.entry) &&
    value.progress.entry.kind === "recovery"
  ) {
    value = {
      ...value,
      progress: { ...value.progress, entry: { ...value.progress.entry, kind: "flash" } },
    };
  }
  return migrateSave(value).save;
}

export function migrateStorageV1ToV2(source: LocalSource) {
  const raw = source[SAVE_KEY];
  const current = raw ? currentFlashSave(importedSave(raw)) : undefined;
  const text = source[HISTORY_KEY];
  const entries = text ? migrateHistoryIndex(JSON.parse(text)).index.entries : [];
  const turns = entries.map((entry) => {
    const payload = source[historyPayloadKey(entry.id)];
    if (payload === null || payload === undefined)
      throw new Error(`Missing saved turn ${entry.id}.`);
    const save = currentTurn(importedSave(payload));
    if (save.progress.entry.id !== entry.id || save.progress.entry.runId !== entry.runId)
      throw new Error(`Saved turn metadata does not match ${entry.id}.`);
    return save;
  });
  return { current, turns, chooseNewGame: source[NEW_GAME_KEY] === "true" };
}
