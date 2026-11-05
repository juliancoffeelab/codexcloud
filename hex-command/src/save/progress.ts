import type { BattleSave } from "./schema";
import { readSave } from "./storage";
import type { LiveGame } from "../game/live-game";

/** Stable identities and parent links are independent of a future private sync transport. */
export interface ProgressIdentity {
  runId: string;
  branchId: string;
  checkpointId: string;
  ordinal: number;
}
export interface ProgressEntry extends ProgressIdentity {
  id: string;
  parentId?: string;
  savedAt: number;
  run: string;
  mission: string;
  round: number;
  side: string;
  kind: "turn" | "recovery";
  missionStart: boolean;
  current?: boolean;
  protected?: boolean;
}
interface Index {
  version: 1;
  entries: ProgressEntry[];
}
const INDEX_KEY = "hex-command-progress-index";
const payloadKey = (id: string) => `hex-command-progress:${id}`;
export const progressId = (): string => crypto.randomUUID();
export function newRun(): ProgressIdentity {
  return { runId: progressId(), branchId: progressId(), checkpointId: progressId(), ordinal: 0 };
}
function index(): Index {
  const text = localStorage.getItem(INDEX_KEY);
  if (!text) return { version: 1, entries: [] };
  const value = JSON.parse(text) as Index;
  if (value.version !== 1 || !Array.isArray(value.entries))
    throw new Error(
      "Saved progress index could not be read. Browser storage has not been changed.",
    );
  return value;
}
function commit(value: Index): void {
  localStorage.setItem(INDEX_KEY, JSON.stringify(value));
}
export function describeProgress(
  game: LiveGame,
  identity: ProgressIdentity,
  kind: ProgressEntry["kind"],
  missionStart = false,
): ProgressEntry {
  return {
    ...identity,
    id: kind === "recovery" ? identity.branchId : identity.checkpointId,
    savedAt: Date.now(),
    run: game.campaign?.id ?? game.mapId,
    mission: game.campaign ? `Mission ${game.campaign.mission + 1} · ${game.mapId}` : game.mapId,
    round: game.round,
    side: game.turn,
    kind,
    missionStart,
  };
}
export function currentProgress(): ProgressEntry | undefined {
  const text = readSave();
  if (!text) return;
  const save = JSON.parse(text) as BattleSave;
  return save.progress?.entry;
}
export function listProgress(): ProgressEntry[] {
  const entries = index().entries;
  const current = currentProgress();
  return [...entries.filter((entry) => entry.id !== current?.id), ...(current ? [current] : [])]
    .map((entry) => ({
      ...entry,
      current: entry.id === current?.id,
      protected: entry.id === current?.id || entry.id === current?.checkpointId,
    }))
    .sort((a, b) => b.savedAt - a.savedAt || a.id.localeCompare(b.id));
}
/** Payload first, discoverability last. A failed write never replaces a reachable checkpoint. */
function insert(entry: ProgressEntry, text: string): void {
  const value = index();
  const key = payloadKey(entry.id);
  const previous = localStorage.getItem(key);
  localStorage.setItem(key, text);
  try {
    commit({
      version: 1,
      entries: [...value.entries.filter((item) => item.id !== entry.id), entry],
    });
  } catch (error) {
    if (previous === null) localStorage.removeItem(key);
    else localStorage.setItem(key, previous);
    throw error;
  }
}
/** Preserve each inactive continuation's latest recovery, including its move undo. */
export function parkRecovery(): void {
  const text = readSave();
  if (!text) return;
  const save = JSON.parse(text) as BattleSave;
  if (save.progress) insert(save.progress.entry, text);
}
export function addTurn(entry: ProgressEntry, text: string): void {
  insert(entry, text);
}
export function progressText(id: string): string {
  const current = currentProgress();
  const text = id === current?.id ? readSave() : localStorage.getItem(payloadKey(id));
  if (!text) throw new Error("This saved checkpoint is no longer available.");
  return text;
}
function removeWhere(
  predicate: (entry: ProgressEntry) => boolean,
  protectedIds: string[] = [],
): void {
  const value = index();
  const current = currentProgress();
  const removed = value.entries.filter(
    (entry) =>
      entry.id !== current?.id &&
      entry.id !== current?.checkpointId &&
      !protectedIds.includes(entry.id) &&
      predicate(entry),
  );
  const ids = new Set(removed.map((entry) => entry.id));
  commit({ version: 1, entries: value.entries.filter((entry) => !ids.has(entry.id)) });
  // Interrupted cleanup can leave unreferenced bytes, never a dangling index entry.
  for (const entry of removed) localStorage.removeItem(payloadKey(entry.id));
}
export function deleteProgress(id: string, protectedIds: string[] = []): void {
  const current = currentProgress();
  if (id === current?.id || id === current?.checkpointId || protectedIds.includes(id))
    throw new Error("The currently played checkpoint cannot be deleted.");
  removeWhere((entry) => entry.id === id, protectedIds);
}
export function deleteRun(runId: string, protectedIds: string[] = []): void {
  removeWhere((entry) => entry.runId === runId, protectedIds);
}
/** Dense recent history, then one checkpoint per exponentially wider age bucket, per continuation. */
export function pruneProgress(runId?: string, protectedIds: string[] = []): void {
  const entries = index().entries;
  const keep = new Set<string>();
  const branches = new Map<string, ProgressEntry[]>();
  for (const entry of entries) {
    if (entry.kind !== "turn" || (runId && entry.runId !== runId)) continue;
    const branch = branches.get(entry.branchId) ?? [];
    branch.push(entry);
    branches.set(entry.branchId, branch);
  }
  for (const branch of branches.values()) {
    const newest = branch.reduce((maximum, entry) => Math.max(maximum, entry.ordinal), 0);
    for (const entry of branch) {
      const age = newest - entry.ordinal;
      const interval = age < 24 ? 1 : 2 ** (1 + Math.floor(Math.log2(age / 24)));
      if (entry.missionStart || entry.ordinal % interval === 0) keep.add(entry.id);
    }
  }
  removeWhere(
    (entry) => entry.kind === "turn" && (!runId || entry.runId === runId) && !keep.has(entry.id),
    protectedIds,
  );
}

/** Loading is the sole entry point for a validated migration of a historical payload. */
export function updateProgressText(id: string, text: string): void {
  if (!index().entries.some((entry) => entry.id === id))
    throw new Error("Saved checkpoint is unavailable.");
  localStorage.setItem(payloadKey(id), text);
}
