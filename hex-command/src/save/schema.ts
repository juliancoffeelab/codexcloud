import { validSetup, type SessionSetup } from "../game/setup";
import type { SavedGame } from "../../rust/pkg/hex_command";

/** Single version for browser persistence; native checkpoints are current data only. */
export const SAVE_VERSION = 31;
export interface BattleSave {
  version: typeof SAVE_VERSION;
  checkpoint: SavedGame;
  pendingEnemy: boolean;
  setup?: SessionSetup;
  undo?: import("../game/undo").SavedUndoEntry[];
  progress?: { entry: import("./progress").ProgressEntry };
}
export const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** Validate the current envelope; Rust validates the complete checkpoint and battle. */
export function currentSave(value: unknown): BattleSave {
  if (
    !isRecord(value) ||
    value.version !== SAVE_VERSION ||
    Object.keys(value).some(
      (key) =>
        !["version", "checkpoint", "pendingEnemy", "setup", "undo", "progress"].includes(key),
    ) ||
    (value.undo !== undefined &&
      (!Array.isArray(value.undo) ||
        value.undo.some(
          (entry) =>
            !isRecord(entry) ||
            !isRecord(entry.state) ||
            typeof entry.unitId !== "string" ||
            !Number.isInteger(entry.round),
        ))) ||
    (value.progress !== undefined &&
      (!isRecord(value.progress) ||
        !isRecord(value.progress.entry) ||
        !["id", "runId", "branchId", "checkpointId", "run", "mission", "side"].every(
          (key) =>
            typeof (value.progress as { entry: Record<string, unknown> }).entry[key] === "string",
        ) ||
        !["turn", "recovery"].includes(String(value.progress.entry.kind)) ||
        typeof value.progress.entry.savedAt !== "number" ||
        typeof value.progress.entry.round !== "number" ||
        !Number.isSafeInteger(value.progress.entry.ordinal) ||
        typeof value.progress.entry.missionStart !== "boolean")) ||
    typeof value.pendingEnemy !== "boolean" ||
    (value.setup !== undefined && !validSetup(value.setup)) ||
    !isRecord(value.checkpoint) ||
    !isRecord(value.checkpoint.state) ||
    !Array.isArray(value.checkpoint.state.factions) ||
    !Array.isArray(value.checkpoint.state.groundItems) ||
    Object.keys(value.checkpoint).some((key) => !["seed", "state"].includes(key)) ||
    (value.pendingEnemy &&
      (value.checkpoint.state.turn !== "player" || value.checkpoint.state.winner !== null))
  )
    throw new Error("Invalid current save envelope");
  return value as unknown as BattleSave;
}
