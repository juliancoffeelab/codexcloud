import { currentTurn, type TurnSave } from "./schema";
import { CURRENT, NEW_GAME, STATE, ENTRIES, TURNS, repository, request } from "./storage";
import { readCurrentFlashSave, flashSaveEntry } from "./load";
import { TurnCheckpoint } from "./write";
import type { CurrentEntry, TurnEntry } from "./metadata";

export function currentProgress(): Promise<CurrentEntry | undefined> {
  return readCurrentFlashSave();
}
/** Metadata is separate from payloads: listing never deserializes battlefields. */
export async function listProgress(): Promise<TurnEntry[]> {
  return repository([ENTRIES], "readonly", async (tx) =>
    (await request<TurnEntry[]>(tx.objectStore(ENTRIES).getAll())).sort(
      (a, b) => b.savedAt - a.savedAt || a.id.localeCompare(b.id),
    ),
  );
}
/** Metadata and payload become visible in the same transaction. */
export async function addTurn(checkpoint: TurnCheckpoint): Promise<void> {
  const text = TurnCheckpoint.text(checkpoint);
  await repository([ENTRIES, TURNS], "readwrite", (tx) => {
    tx.objectStore(ENTRIES).put(checkpoint.entry);
    tx.objectStore(TURNS).put(text, checkpoint.entry.id);
  });
}
/** Raw bytes go straight to the versioned migration loader. */
export async function progressText(id: string): Promise<string> {
  return repository([TURNS], "readonly", async (tx) => {
    const text = await request<string | undefined>(tx.objectStore(TURNS).get(id));
    if (text === undefined) throw new Error("Saved turn is unavailable.");
    return text;
  });
}
export async function deleteProgress(id: string): Promise<void> {
  await repository([ENTRIES, TURNS], "readwrite", async (tx) => {
    if (!(await request(tx.objectStore(ENTRIES).get(id))))
      throw new Error("Saved turn is unavailable.");
    tx.objectStore(ENTRIES).delete(id);
    tx.objectStore(TURNS).delete(id);
  });
}
export async function deleteRun(runId: string): Promise<void> {
  await repository([ENTRIES, TURNS], "readwrite", async (tx) => {
    const state = tx.objectStore(STATE);
    const raw = await request<string | undefined>(state.get(CURRENT));
    const ownsFlashSave = raw !== undefined && flashSaveEntry(raw).runId === runId;
    const ids = await request(tx.objectStore(ENTRIES).index("runId").getAllKeys(runId));
    for (const id of ids) {
      tx.objectStore(ENTRIES).delete(id);
      tx.objectStore(TURNS).delete(id);
    }
    if (ownsFlashSave) {
      state.delete(CURRENT);
      state.put(true, NEW_GAME);
    }
  });
}
/** Recent saves are dense; old branches share their game's age-based allowance. */
function retained(entries: TurnEntry[], runId: string | undefined, now: number): Set<string> {
  const keep = new Set<string>(),
    latest = new Map<string, TurnEntry>(),
    buckets = new Map<string, TurnEntry>();
  const minute = 60_000,
    hour = 60 * minute,
    day = 24 * hour;
  const newer = (a: TurnEntry, b: TurnEntry) =>
    a.savedAt > b.savedAt || (a.savedAt === b.savedAt && a.id < b.id);
  for (const entry of entries) {
    if (runId && entry.runId !== runId) continue;
    const previous = latest.get(entry.runId);
    if (!entry.manual && (!previous || newer(entry, previous))) latest.set(entry.runId, entry);
    if (entry.manual || entry.missionStart) {
      keep.add(entry.id);
      continue;
    }
    const age = Math.max(0, now - entry.savedAt);
    if (age < 5 * minute) {
      keep.add(entry.id);
      continue;
    }
    const interval =
      age < hour
        ? 5 * minute
        : age < day
          ? hour
          : age < 7 * day
            ? day
            : age < 30 * day
              ? 7 * day
              : 0;
    if (!interval) continue;
    const key = `${entry.runId}:${interval}:${Math.floor(entry.savedAt / interval)}`;
    const retained = buckets.get(key);
    if (!retained || newer(entry, retained)) buckets.set(key, entry);
  }
  for (const entry of [...latest.values(), ...buckets.values()]) keep.add(entry.id);
  return keep;
}

/** Selection and removal share a transaction with competing inserts/deletes. */
export async function pruneProgress(runId?: string, now = Date.now()): Promise<void> {
  await repository([ENTRIES, TURNS], "readwrite", async (tx) => {
    const store = tx.objectStore(ENTRIES);
    const entries: TurnEntry[] = await request(
      runId ? store.index("runId").getAll(runId) : store.getAll(),
    );
    const keep = retained(entries, runId, now);
    for (const entry of entries)
      if (!keep.has(entry.id)) {
        store.delete(entry.id);
        tx.objectStore(TURNS).delete(entry.id);
      }
  });
}
/** Conversion replaces only an existing turn, atomically with its metadata. */
export async function updateProgress(save: TurnSave): Promise<void> {
  currentTurn(save);
  await repository([ENTRIES, TURNS], "readwrite", async (tx) => {
    const entry = save.progress.entry;
    if (!(await request(tx.objectStore(ENTRIES).get(entry.id))))
      throw new Error("Saved turn is unavailable.");
    tx.objectStore(ENTRIES).put(entry);
    tx.objectStore(TURNS).put(JSON.stringify(save), entry.id);
  });
}
