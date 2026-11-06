import { readImportSource, withImportLock } from "./import";
import {
  initialized,
  transaction,
  request,
  STATE,
  ENTRIES,
  TURNS,
  CURRENT,
  INITIALIZED,
  NEW_GAME,
  STORAGE_VERSION,
} from "./storage";
import {
  inspectStorageV1,
  repairStorageV1,
  SAVE_KEY,
  HISTORY_KEY,
  historyPayloadKey,
  type LocalSource,
} from "./migrations/storage-v1-to-v2";
import type { BattleSave } from "./schema";
import type { ImportFailure, RepairInventory, RepairTarget } from "./repair-types";

/** Raw target-store inspection bypasses normal repository validation. */
async function databaseSource(tx: IDBTransaction): Promise<LocalSource> {
  const source: LocalSource = {};
  source[SAVE_KEY] = (await request(tx.objectStore(STATE).get(CURRENT))) ?? null;
  const entries = await request(tx.objectStore(ENTRIES).getAll());
  source[HISTORY_KEY] = JSON.stringify({ version: 4, entries });
  const store = tx.objectStore(TURNS);
  const [keys, values] = await Promise.all([request(store.getAllKeys()), request(store.getAll())]);
  keys.forEach((key, i) => {
    source[historyPayloadKey(String(key))] = values[i];
  });
  return source;
}
export function inspectImport(
  failure: ImportFailure,
  validate: (save: BattleSave) => void,
): Promise<RepairInventory> {
  return withImportLock(async () => {
    const errors: string[] = [];
    let authoritative = false;
    if (failure.source === "localStorage") {
      try {
        authoritative = await initialized();
      } catch (error) {
        errors.push(`Save storage is unavailable: ${String(error)}`);
      }
      if (authoritative) return { records: [], errors, authoritative };
    }
    let source: LocalSource;
    try {
      source =
        failure.source === "localStorage"
          ? readImportSource()
          : await transaction([STATE, ENTRIES, TURNS], "readonly", databaseSource);
    } catch (error) {
      return { records: [], errors: [...errors, String(error)], authoritative };
    }
    const records = inspectStorageV1(source);
    for (const record of records) {
      if (record.save) {
        try {
          validate(record.save);
        } catch (error) {
          record.error = String(error);
        }
        delete record.save;
      }
    }
    return { records, errors, authoritative };
  });
}
/** Destructive repair never requires a valid battlefield or a successful full import. */
export function repairImport(failure: ImportFailure, target: RepairTarget): Promise<void> {
  return withImportLock(async () => {
    if (failure.source === "localStorage") {
      // Another tab's completed import wins. Do not mutate it through source repair.
      let authoritative = false;
      try {
        authoritative = await initialized();
      } catch {
        /* IDB failure must not block source cleanup. */
      }
      if (authoritative)
        throw new Error(
          "Import finished in another tab. Import the saved game here before deleting anything.",
        );
      const source = readImportSource();
      const plan = repairStorageV1(source, target);
      // Deletes require no additional quota; a failed index update remains repairable.
      for (const key of plan.deleted) localStorage.removeItem(key);
      for (const [key, value] of Object.entries(plan.writes))
        if (value !== null) localStorage.setItem(key, value);
      return;
    }
    await transaction([STATE, ENTRIES, TURNS], "readwrite", async (tx) => {
      if (target.kind === "all") {
        tx.objectStore(ENTRIES).clear();
        tx.objectStore(TURNS).clear();
        tx.objectStore(STATE).clear();
        tx.objectStore(STATE).put(STORAGE_VERSION, INITIALIZED);
        tx.objectStore(STATE).put(true, NEW_GAME);
        return;
      }
      const source = await databaseSource(tx);
      const plan = repairStorageV1(source, target);
      for (const key of plan.deleted) {
        if (key === SAVE_KEY) {
          tx.objectStore(STATE).delete(CURRENT);
          tx.objectStore(STATE).put(true, NEW_GAME);
        } else if (key.startsWith("hex-command-progress:")) {
          const id = key.slice("hex-command-progress:".length);
          tx.objectStore(TURNS).delete(id);
          tx.objectStore(ENTRIES).delete(id);
        }
      }
      if (plan.writes[HISTORY_KEY]) {
        const entries = JSON.parse(plan.writes[HISTORY_KEY]!).entries;
        tx.objectStore(ENTRIES).clear();
        for (const entry of entries) tx.objectStore(ENTRIES).put(entry);
      }
    });
  });
}
