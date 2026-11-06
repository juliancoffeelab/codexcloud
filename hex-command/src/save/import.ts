import { initialized, initializeStorage } from "./storage";
import {
  migrateStorageV1ToV2,
  sourceKey,
  SAVE_KEY,
  HISTORY_KEY,
  NEW_GAME_KEY,
  historyPayloadKey,
  type LocalSource,
} from "./migrations/storage-v1-to-v2";
import { HISTORY_VERSION, type BattleSave } from "./schema";

function source(): LocalSource {
  const result: LocalSource = {};
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && sourceKey(key)) result[key] = localStorage.getItem(key);
  }
  return result;
}
function discardSource(): void {
  // Cleanup after a committed import is best effort; IDB remains authoritative.
  try {
    for (const key of Object.keys(source())) localStorage.removeItem(key);
  } catch {
    /* Next startup can retry. */
  }
}
export async function importLocalStorage(validate: (save: BattleSave) => void): Promise<void> {
  if (await initialized()) {
    discardSource();
    return;
  }
  const converted = migrateStorageV1ToV2(source());
  if (converted.current) validate(converted.current);
  for (const save of converted.turns) validate(save);
  await initializeStorage(converted);
  discardSource();
}
/** Migration recovery tools operate on the uncommitted source, never as a gameplay backend. */
export function pendingImport() {
  return migrateStorageV1ToV2(source());
}
export function deleteImportSaves(target: { runId: string } | { id: string }): void {
  const data = pendingImport();
  const removed = data.turns.filter((save) =>
    "runId" in target
      ? save.progress.entry.runId === target.runId
      : save.progress.entry.id === target.id,
  );
  const ids = new Set(removed.map((save) => save.progress.entry.id));
  const entries = data.turns
    .filter((save) => !ids.has(save.progress.entry.id))
    .map((save) => save.progress.entry);
  // Shrink the index before removing payloads, retaining a valid source on interruption.
  localStorage.setItem(HISTORY_KEY, JSON.stringify({ version: HISTORY_VERSION, entries }));
  for (const id of ids) localStorage.removeItem(historyPayloadKey(id));
  if ("runId" in target && data.current?.progress.entry.runId === target.runId) {
    localStorage.removeItem(SAVE_KEY);
    localStorage.setItem(NEW_GAME_KEY, "true");
  }
}
export function captureImportSource(): LocalSource {
  return source();
}
