import { decodeUndo } from "./undo";
import { LiveGame, nativeValue } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migrations";
import { wipeSave, storeSave, readSave, SAVE_KEY, HISTORY_KEY, historyPayloadKey } from "./storage";
import { migrateHistoryIndex } from "./migrations/history";
import { currentRecovery, currentTurn } from "./schema";
import type { CurrentEntry } from "./metadata";
import type { BattleSave } from "./schema";

export interface LoadedBattle {
  game: LiveGame;
  pendingEnemy: boolean;
  save?: BattleSave;
  undo?: import("../game/undo").SavedUndoEntry[];
  notice?: string;
  storageFailure?: string;
  retainedRecovery?: CurrentEntry;
}
/** Canonical decoding belongs to the loading boundary, independent of committing it. */
function decodeBattleText(raw: string) {
  return migrateSave(JSON.parse(raw));
}
/** Read ownership without requiring a migration write or starting a battle. */
export function readCurrentRecovery(): CurrentEntry | undefined {
  const raw = readSave();
  return raw === null ? undefined : currentRecovery(decodeBattleText(raw).save).progress.entry;
}
/** Migration is data-only. Validate both recovery and move undo through the native engine. */
export function restoreBattleText(
  factory: GameEngineFactory,
  raw: string,
  kind: "recovery" | "turn" = "recovery",
): LoadedBattle & { save: BattleSave; migrated: boolean } {
  const { save: converted, migrated } = decodeBattleText(raw);
  const save = kind === "recovery" ? currentRecovery(converted) : currentTurn(converted);
  if (save.undo?.length) {
    if (save.pendingEnemy) throw new Error("Pending enemy turns cannot retain move undo.");
    nativeValue(factory.validateUndo(save.checkpoint.state, save.undo));
  }
  const undo = decodeUndo(save.checkpoint.state, save.undo ?? []);
  const game = LiveGame.restore(factory, save.checkpoint, save.setup);
  return { game, pendingEnemy: save.pendingEnemy, save, migrated, undo };
}
/** Parse, migrate, validate natively, then persist the successfully restored format. */
export function loadBattle(factory: GameEngineFactory): LoadedBattle {
  let raw: string | null = null;
  let storageFailure: string | undefined;
  try {
    raw = localStorage.getItem(SAVE_KEY);
  } catch (error) {
    storageFailure = String(error);
  }
  if (raw !== null) {
    try {
      const restored = restoreBattleText(factory, raw);
      // Storage errors must not turn a valid restored battle into a fresh battle.
      if (restored.migrated) {
        try {
          storeSave(JSON.stringify(restored.save));
        } catch (error) {
          restored.game.dispose();
          return {
            game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
            pendingEnemy: false,
            storageFailure: String(error),
            retainedRecovery: currentRecovery(restored.save).progress.entry,
            notice:
              "Saved progress needs an update, but storage refused the write. Free storage in Saved progress, then reload to restore it.",
          };
        }
      }
      return restored;
    } catch (error) {
      wipeSave();
      return {
        game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
        pendingEnemy: false,
        notice: `The saved battle could not be loaded and was cleared. A new battle is ready.\n${error instanceof Error ? (error.stack ?? error.message) : String(error)}`,
      };
    }
  }
  return {
    game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
    pendingEnemy: false,
    storageFailure,
  };
}

/** Called under the history lock before current-format repository access. */
export function loadProgressHistory(): void {
  const text = localStorage.getItem(HISTORY_KEY);
  if (text === null) return;
  const { index, migrated, removedIds } = migrateHistoryIndex(JSON.parse(text));
  if (!migrated) return;
  localStorage.setItem(HISTORY_KEY, JSON.stringify(index));
  for (const id of removedIds) localStorage.removeItem(historyPayloadKey(id));
}
