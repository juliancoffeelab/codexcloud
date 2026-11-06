import {
  currentHistoryIndex,
  currentRecovery,
  currentTurn,
  HISTORY_VERSION,
  type HistoryIndex,
  type TurnSave,
} from "./schema";
import { readSave, HISTORY_KEY, historyPayloadKey } from "./storage";
import { TurnStartCheckpoint } from "./write";
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
/** Only turn-boundary snapshots can create historical records. Caller holds the storage lock. */
export function addTurn(checkpoint: TurnStartCheckpoint): void {
  const text = TurnStartCheckpoint.text(checkpoint);
  const entry = checkpoint.entry;
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
function removeWhere(predicate: (entry: TurnEntry) => boolean): void {
  const value = index(),
    removed = value.entries.filter(predicate),
    ids = new Set(removed.map((entry) => entry.id));
  commit({
    version: HISTORY_VERSION,
    entries: value.entries.filter((entry) => !ids.has(entry.id)),
  });
  for (const entry of removed) localStorage.removeItem(historyPayloadKey(entry.id));
}
export function deleteProgress(id: string): void {
  if (!index().entries.some((entry) => entry.id === id))
    throw new Error("Saved turn is unavailable.");
  removeWhere((entry) => entry.id === id);
}
export function deleteRun(runId: string): void {
  removeWhere((entry) => entry.runId === runId);
}
/** Existing dense/exponential retention policy, applied exclusively to turn starts. */
export function pruneProgress(runId?: string): void {
  const entries = index().entries,
    keep = new Set<string>(),
    branches = new Map<string, TurnEntry[]>();
  for (const entry of entries) {
    if (runId && entry.runId !== runId) continue;
    const branch = branches.get(entry.branchId) ?? [];
    branch.push(entry);
    branches.set(entry.branchId, branch);
  }
  for (const branch of branches.values()) {
    const newest = branch.reduce((maximum, entry) => Math.max(maximum, entry.ordinal), 0);
    for (const entry of branch) {
      const age = newest - entry.ordinal,
        interval = age < 24 ? 1 : 2 ** (1 + Math.floor(Math.log2(age / 24)));
      if (entry.missionStart || entry.ordinal % interval === 0) keep.add(entry.id);
    }
  }
  removeWhere((entry) => (!runId || entry.runId === runId) && !keep.has(entry.id));
}
/** Migration may replace an existing turn payload, but cannot add records or recoveries. */
export function updateProgress(save: TurnSave): void {
  currentTurn(save);
  const id = save.progress.entry.id;
  if (!index().entries.some((entry) => entry.id === id))
    throw new Error("Saved turn is unavailable.");
  localStorage.setItem(historyPayloadKey(id), JSON.stringify(save));
}
