import { initialized, initializeStorage } from "./storage";
import {
  migrateStorageV1ToV2,
  sourceKey,
  SAVE_KEY,
  type LocalSource,
} from "./migrations/storage-v1-to-v2";
import type { BattleSave } from "./schema";

export function readImportSource(): LocalSource {
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
    for (const key of Object.keys(readImportSource())) localStorage.removeItem(key);
  } catch {
    /* Next startup can retry. */
  }
}
export async function withImportLock<T>(work: () => Promise<T>): Promise<T> {
  return await navigator.locks.request("hex-command-storage-import", work);
}
export function importLocalStorage(validate: (save: BattleSave) => void): Promise<void> {
  return withImportLock(async () => {
    if (await initialized()) {
      discardSource();
      return;
    }
    const converted = migrateStorageV1ToV2(readImportSource());
    if (converted.current) validate(converted.current);
    for (const save of converted.turns) validate(save);
    await initializeStorage(converted);
    discardSource();
  });
}
export function captureImportSource(): LocalSource {
  return readImportSource();
}
