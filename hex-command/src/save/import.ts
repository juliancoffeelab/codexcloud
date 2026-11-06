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
/** Stage information prevents a storage read failure being mistaken for bad contents. */
export class ImportStageError extends Error {
  constructor(
    readonly stage: "access" | "data" | "commit",
    readonly original: unknown,
  ) {
    super(String(original), { cause: original });
  }
}
export function importLocalStorage(validate: (save: BattleSave) => void): Promise<void> {
  return withImportLock(async () => {
    try {
      if (await initialized()) {
        discardSource();
        return;
      }
    } catch (error) {
      throw new ImportStageError("access", error);
    }
    let source: LocalSource;
    try {
      source = readImportSource();
    } catch (error) {
      throw new ImportStageError("access", error);
    }
    let converted: ReturnType<typeof migrateStorageV1ToV2>;
    try {
      converted = migrateStorageV1ToV2(source);
      if (converted.current) validate(converted.current);
      for (const save of converted.turns) validate(save);
    } catch (error) {
      throw new ImportStageError("data", error);
    }
    try {
      await initializeStorage(converted);
    } catch (error) {
      throw new ImportStageError("commit", error);
    }
    discardSource();
  });
}
export function captureImportSource(): LocalSource {
  return readImportSource();
}
