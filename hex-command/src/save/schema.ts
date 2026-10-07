import type { SavedGame } from "../../rust/pkg/hex_command";

/** Single version for browser persistence; native checkpoints are current data only. */
export const SAVE_VERSION = 2;
export interface BattleSave {
  version: typeof SAVE_VERSION;
  checkpoint: SavedGame;
  pendingEnemy: boolean;
}
export const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** Validate the current envelope; Rust validates the complete checkpoint and battle. */
export function currentSave(value: unknown): BattleSave {
  if (
    !isRecord(value) ||
    value.version !== SAVE_VERSION ||
    Object.keys(value).some((key) => !["version", "checkpoint", "pendingEnemy"].includes(key)) ||
    typeof value.pendingEnemy !== "boolean" ||
    !isRecord(value.checkpoint) ||
    !isRecord(value.checkpoint.state) ||
    "version" in value.checkpoint ||
    (value.pendingEnemy &&
      (value.checkpoint.state.turn !== "player" || value.checkpoint.state.winner !== null))
  )
    throw new Error("Invalid current save envelope");
  return value as unknown as BattleSave;
}
