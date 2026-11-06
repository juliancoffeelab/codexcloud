import { validUndoStep } from "./undo";
import { validSetup, type SessionSetup } from "../game/setup";
import type { SavedGame, UndoStep } from "../../rust/pkg/hex_command";
import type { CurrentEntry, TurnEntry } from "./metadata";

export const SAVE_VERSION = 35;
export const HISTORY_VERSION = 2;
interface SaveBase {
  version: typeof SAVE_VERSION;
  checkpoint: SavedGame;
  setup: SessionSetup;
}
export interface CurrentSave extends SaveBase {
  pendingEnemy: boolean;
  undo: UndoStep[];
  progress: { entry: CurrentEntry };
}
export interface TurnSave extends SaveBase {
  pendingEnemy: false;
  undo?: never;
  progress: { entry: TurnEntry };
}
export type BattleSave = CurrentSave | TurnSave;
export interface HistoryIndex {
  version: typeof HISTORY_VERSION;
  entries: TurnEntry[];
}
export const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);
function validEntry(value: unknown): value is CurrentEntry | TurnEntry {
  if (!isRecord(value) || !["turn", "recovery"].includes(String(value.kind))) return false;
  const keys = [
    "id",
    "runId",
    "branchId",
    "checkpointId",
    "ordinal",
    "savedAt",
    "run",
    "mission",
    "round",
    "side",
    "kind",
    "missionStart",
  ];
  if (value.kind === "turn") keys.push("parentId");
  return (
    Object.keys(value).every((key) => keys.includes(key)) &&
    ["id", "runId", "branchId", "checkpointId", "run", "mission", "side"].every(
      (key) => typeof value[key] === "string",
    ) &&
    Number.isFinite(value.savedAt) &&
    Number.isFinite(value.round) &&
    Number.isSafeInteger(value.ordinal) &&
    typeof value.missionStart === "boolean" &&
    (value.kind === "turn"
      ? value.id === value.checkpointId &&
        (value.parentId === undefined || typeof value.parentId === "string")
      : value.id === value.branchId && value.missionStart === false)
  );
}
/** Current contracts only. Historical forms are converted before validation. */
export function currentSave(value: unknown): BattleSave {
  if (
    !isRecord(value) ||
    value.version !== SAVE_VERSION ||
    Object.keys(value).some(
      (key) =>
        !["version", "checkpoint", "pendingEnemy", "setup", "undo", "progress"].includes(key),
    ) ||
    !validSetup(value.setup) ||
    !isRecord(value.progress) ||
    Object.keys(value.progress).some((key) => key !== "entry") ||
    !validEntry(value.progress.entry) ||
    typeof value.pendingEnemy !== "boolean" ||
    !isRecord(value.checkpoint) ||
    !isRecord(value.checkpoint.state) ||
    !Array.isArray(value.checkpoint.state.factions) ||
    !Array.isArray(value.checkpoint.state.groundItems) ||
    Object.keys(value.checkpoint).some((key) => !["seed", "state"].includes(key)) ||
    (value.pendingEnemy &&
      (value.checkpoint.state.turn !== "player" || value.checkpoint.state.winner !== null)) ||
    (value.progress.entry.kind === "turn"
      ? value.pendingEnemy !== false || value.undo !== undefined
      : !Array.isArray(value.undo) || value.undo.some((entry) => !validUndoStep(entry)))
  )
    throw new Error("Invalid current save envelope");
  return value as unknown as BattleSave;
}
export function currentRecovery(value: unknown): CurrentSave {
  const save = currentSave(value);
  if (save.progress.entry.kind !== "recovery") throw new Error("Expected the current recovery");
  return save as CurrentSave;
}
export function currentTurn(value: unknown): TurnSave {
  const save = currentSave(value);
  if (save.progress.entry.kind !== "turn") throw new Error("Expected a turn-start checkpoint");
  return save as TurnSave;
}
export function currentHistoryIndex(value: unknown): HistoryIndex {
  if (
    !isRecord(value) ||
    value.version !== HISTORY_VERSION ||
    Object.keys(value).some((key) => key !== "version" && key !== "entries") ||
    !Array.isArray(value.entries) ||
    value.entries.some((entry) => !validEntry(entry) || entry.kind !== "turn") ||
    new Set(value.entries.map((entry) => entry.id)).size !== value.entries.length
  )
    throw new Error(
      "Saved progress index could not be read. Browser storage has not been changed.",
    );
  return value as unknown as HistoryIndex;
}
