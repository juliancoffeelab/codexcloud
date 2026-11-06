import { decodeUndo } from "./undo";
import { LiveGame, nativeValue } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migrations";
import { wipeSave, storeSave, readSave, readSession } from "./storage";
import { importLocalStorage, pendingImport } from "./import";
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
  chooseNewGame?: boolean;
  importPending?: boolean;
}
/** Canonical decoding belongs to the loading boundary, independent of committing it. */
function decodeBattleText(raw: string) {
  return migrateSave(JSON.parse(raw));
}
/** Read ownership without requiring a migration write or starting a battle. */
export function recoveryEntry(raw: string): CurrentEntry {
  return currentRecovery(decodeBattleText(raw).save).progress.entry;
}
export async function readCurrentRecovery(): Promise<CurrentEntry | undefined> {
  const raw = await readSave();
  return raw === null ? undefined : recoveryEntry(raw);
}
/** Validate without leaving a battle handle alive during the storage transaction. */
export async function initializeRepository(factory: GameEngineFactory): Promise<void> {
  await importLocalStorage((save) => {
    const restored = restoreBattleText(
      factory,
      JSON.stringify(save),
      save.progress.entry.kind === "turn" ? "turn" : "recovery",
    );
    restored.game.dispose();
  });
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
export async function loadBattle(factory: GameEngineFactory): Promise<LoadedBattle> {
  try {
    await initializeRepository(factory);
  } catch (error) {
    let retainedRecovery: CurrentEntry | undefined;
    let chooseNewGame = false;
    try {
      const data = await pendingImport();
      if (!data) return loadBattle(factory);
      retainedRecovery = data.current?.progress.entry;
      chooseNewGame = data.chooseNewGame;
    } catch {
      /* Preserve malformed source for diagnostics. */
    }
    return {
      game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
      pendingEnemy: false,
      storageFailure: `${String(error)}\n${error instanceof Error ? (error.stack ?? "") : ""}`,
      retainedRecovery,
      chooseNewGame,
      importPending: true,
    };
  }
  let raw: string | null = null;
  let storageFailure: string | undefined;
  let chooseNewGame = false;
  try {
    ({ text: raw, chooseNewGame } = await readSession());
  } catch (error) {
    storageFailure = String(error);
  }
  if (raw !== null) {
    try {
      const restored = restoreBattleText(factory, raw);
      // Storage errors must not turn a valid restored battle into a fresh battle.
      if (restored.migrated) {
        try {
          await storeSave(JSON.stringify(restored.save));
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
      await wipeSave();
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
    chooseNewGame,
  };
}
