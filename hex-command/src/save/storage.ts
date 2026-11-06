import type { CurrentSave, TurnSave } from "./schema";

export const DATABASE_NAME = "hex-command-saves";
export const DATABASE_VERSION = 1;
export const STORAGE_VERSION = 2;
export const STATE = "state",
  ENTRIES = "entries",
  TURNS = "turns";
export const CURRENT = "current",
  INITIALIZED = "initialized",
  NEW_GAME = "choose-new-game";
let owner: IDBFactory | undefined;
let opening: Promise<IDBDatabase> | undefined;

export function request<T>(value: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    value.onsuccess = () => resolve(value.result);
    value.onerror = () => reject(value.error);
  });
}
export function database(): Promise<IDBDatabase> {
  if (owner !== indexedDB || !opening) {
    owner = indexedDB;
    opening = new Promise((resolve, reject) => {
      const value = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);
      value.onupgradeneeded = () => {
        const db = value.result;
        db.createObjectStore(STATE);
        const entries = db.createObjectStore(ENTRIES, { keyPath: "id" });
        entries.createIndex("runId", "runId");
        entries.createIndex("savedAt", "savedAt");
        db.createObjectStore(TURNS);
      };
      value.onsuccess = () => {
        const db = value.result;
        db.onversionchange = () => {
          db.close();
          opening = undefined;
        };
        resolve(db);
      };
      value.onerror = () => {
        opening = undefined;
        reject(value.error);
      };
      value.onblocked = () =>
        reject(new Error("Close other Hex Command tabs to update save storage."));
    });
  }
  return opening;
}
/** Only IDB requests may be awaited inside a transaction; conversion happens beforehand. */
export async function transaction<T>(
  stores: string[],
  mode: IDBTransactionMode,
  work: (tx: IDBTransaction) => Promise<T> | T,
): Promise<T> {
  const db = await database();
  const tx = db.transaction(stores, mode);
  let failure: unknown;
  const complete = new Promise<void>((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onabort = () => reject(failure ?? tx.error ?? new Error("Save transaction aborted."));
    tx.onerror = () => {
      /* onabort reports the transaction failure */
    };
  });
  try {
    const result = await work(tx);
    await complete;
    return result;
  } catch (error) {
    failure = error;
    try {
      tx.abort();
    } catch {
      /* Already completed or aborted. */
    }
    await complete.catch(() => {});
    throw error;
  }
}
export async function initialized(): Promise<boolean> {
  return transaction(
    [STATE],
    "readonly",
    async (tx) => (await request(tx.objectStore(STATE).get(INITIALIZED))) === STORAGE_VERSION,
  );
}
export async function repository<T>(
  stores: string[],
  mode: IDBTransactionMode,
  work: (tx: IDBTransaction) => Promise<T> | T,
): Promise<T> {
  return transaction([...new Set([STATE, ...stores])], mode, async (tx) => {
    if ((await request(tx.objectStore(STATE).get(INITIALIZED))) !== STORAGE_VERSION)
      throw new Error("Saved progress needs to finish migrating before it can be written.");
    return work(tx);
  });
}
export async function readSave(): Promise<string | null> {
  return repository(
    [],
    "readonly",
    async (tx) => (await request(tx.objectStore(STATE).get(CURRENT))) ?? null,
  );
}
/** Recovery and the idle/new-game choice change together. */
export async function storeSave(text: string): Promise<void> {
  await repository([], "readwrite", (tx) => {
    tx.objectStore(STATE).put(text, CURRENT);
    tx.objectStore(STATE).delete(NEW_GAME);
  });
}
export async function wipeSave(): Promise<void> {
  await repository([], "readwrite", (tx) => {
    tx.objectStore(STATE).delete(CURRENT);
  });
}
export async function needsNewGame(): Promise<boolean> {
  return repository(
    [],
    "readonly",
    async (tx) => (await request(tx.objectStore(STATE).get(NEW_GAME))) === true,
  );
}
/** The marker and every imported record commit together; a competing initializer wins. */
export async function initializeStorage(data: {
  current?: CurrentSave;
  turns: TurnSave[];
  chooseNewGame: boolean;
}): Promise<void> {
  await transaction([STATE, ENTRIES, TURNS], "readwrite", async (tx) => {
    const state = tx.objectStore(STATE);
    if ((await request(state.get(INITIALIZED))) === STORAGE_VERSION) return;
    if (data.current) state.put(JSON.stringify(data.current), CURRENT);
    if (data.chooseNewGame) state.put(true, NEW_GAME);
    for (const save of data.turns) {
      const entry = save.progress.entry;
      tx.objectStore(ENTRIES).put(entry);
      tx.objectStore(TURNS).put(JSON.stringify(save), entry.id);
    }
    state.put(STORAGE_VERSION, INITIALIZED);
  });
}
