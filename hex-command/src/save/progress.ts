import {
  currentHistoryIndex,
  currentRecovery,
  currentTurn,
  HISTORY_VERSION,
  type HistoryIndex,
  type TurnSave,
} from "./schema";
import { readSave, SAVE_KEY, HISTORY_KEY, historyPayloadKey } from "./storage";
import { TurnCheckpoint } from "./write";
import type { CurrentEntry, TurnEntry } from "./metadata";

function index(): HistoryIndex {
  const text = localStorage.getItem(HISTORY_KEY);
  return text === null
    ? { version: HISTORY_VERSION, entries: [] }
    : currentHistoryIndex(JSON.parse(text));
}
function commit(value: HistoryIndex): void {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(value));
}
export function currentProgress(): CurrentEntry | undefined {
  const text = readSave();
  return text === null ? undefined : currentRecovery(JSON.parse(text)).progress.entry;
}
/** History never includes the current recovery. */
export function listProgress(): TurnEntry[] {
  return index().entries.sort((a, b) => b.savedAt - a.savedAt || a.id.localeCompare(b.id));
}
/** History accepts only snapshot-only values, never a recovery or arbitrary bytes. */
export function addTurn(checkpoint: TurnCheckpoint): void {
  insertCheckpoint(checkpoint.entry, TurnCheckpoint.text(checkpoint));
}
function insertCheckpoint(entry: TurnEntry, text: string): void {
  const value = index(),
    key = historyPayloadKey(entry.id);
  const previous = localStorage.getItem(key);
  localStorage.setItem(key, text);
  try {
    commit({
      version: HISTORY_VERSION,
      entries: [...value.entries.filter((item) => item.id !== entry.id), entry],
    });
  } catch (error) {
    if (previous === null) localStorage.removeItem(key);
    else localStorage.setItem(key, previous);
    throw error;
  }
}
/** Raw bytes go straight to the versioned migration loader, not a runtime format adapter. */
export function progressText(id: string): string {
  if (!index().entries.some((entry) => entry.id === id))
    throw new Error("Saved turn is unavailable.");
  const text = localStorage.getItem(historyPayloadKey(id));
  if (text === null) throw new Error("This saved checkpoint is no longer available.");
  return text;
}
function removeWhere(value: HistoryIndex, predicate: (entry: TurnEntry) => boolean): void {
  const removed = value.entries.filter(predicate),
    ids = new Set(removed.map((entry) => entry.id));
  if (!removed.length) return;
  commit({
    version: HISTORY_VERSION,
    entries: value.entries.filter((entry) => !ids.has(entry.id)),
  });
  for (const entry of removed) localStorage.removeItem(historyPayloadKey(entry.id));
}
export function deleteProgress(id: string): void {
  if (!index().entries.some((entry) => entry.id === id))
    throw new Error("Saved turn is unavailable.");
  removeWhere(index(), (entry) => entry.id === id);
}
export function deleteRun(runId: string): void {
  removeWhere(index(), (entry) => entry.runId === runId);
  if (currentProgress()?.runId === runId) localStorage.removeItem(SAVE_KEY);
}
/** Recent saves are dense; old branches share their game's age-based allowance. */
export function pruneProgress(runId?: string, now = Date.now()): void {
  const value = index(),
    keep = new Set<string>(),
    latest = new Map<string, TurnEntry>(),
    buckets = new Map<string, TurnEntry>();
  const minute = 60_000,
    hour = 60 * minute,
    day = 24 * hour;
  const newer = (a: TurnEntry, b: TurnEntry) =>
    a.savedAt > b.savedAt || (a.savedAt === b.savedAt && a.id < b.id);
  for (const entry of value.entries) {
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
  removeWhere(value, (entry) => (!runId || entry.runId === runId) && !keep.has(entry.id));
}
/** Migration may replace an existing turn payload, but cannot add records or recoveries. */
export function updateProgress(save: TurnSave): void {
  currentTurn(save);
  const id = save.progress.entry.id;
  if (!index().entries.some((entry) => entry.id === id))
    throw new Error("Saved turn is unavailable.");
  localStorage.setItem(historyPayloadKey(id), JSON.stringify(save));
}
