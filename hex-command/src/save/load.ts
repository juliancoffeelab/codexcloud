import { decodeUndo } from "./undo";
import { LiveGame, nativeValue } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migrations";
import { storeSave, readSave, readSession } from "./storage";
import { importLocalStorage, readImportSource } from "./import";
import type { ImportFailure } from "./repair-types";
import { currentFlashSave, currentTurn } from "./schema";
import type { CurrentEntry } from "./metadata";
import type { BattleSave } from "./schema";

export interface LoadedBattle {
  game: LiveGame;
  pendingEnemy: boolean;
  save?: BattleSave;
  undo?: import("../game/undo").SavedUndoEntry[];
  notice?: string;
  storageFailure?: string;
  chooseNewGame?: boolean;
  importFailure?: ImportFailure;
}
/** Canonical decoding belongs to the loading boundary, independent of committing it. */
function decodeBattleText(raw: string) {
  return migrateSave(JSON.parse(raw));
}
/** Read ownership without requiring a migration write or starting a battle. */
export function flashSaveEntry(raw: string): CurrentEntry {
  return currentFlashSave(decodeBattleText(raw).save).progress.entry;
}
export async function readCurrentFlashSave(): Promise<CurrentEntry | undefined> {
  const raw = await readSave();
  return raw === null ? undefined : flashSaveEntry(raw);
}
/** Validate without leaving a battle handle alive during the storage transaction. */
export async function initializeRepository(factory: GameEngineFactory): Promise<void> {
  await importLocalStorage((save) => {
    const restored = restoreBattleText(
      factory,
      JSON.stringify(save),
      save.progress.entry.kind === "turn" ? "turn" : "flash",
    );
    restored.game.dispose();
  });
}
/** Migration is data-only. Validate both the flash save and move undo through the native engine. */
export function restoreBattleText(
  factory: GameEngineFactory,
  raw: string,
  kind: "flash" | "turn" = "flash",
): LoadedBattle & { save: BattleSave; migrated: boolean } {
  const { save: converted, migrated } = decodeBattleText(raw);
  const save = kind === "flash" ? currentFlashSave(converted) : currentTurn(converted);
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
    const detail = `${String(error)}\n${error instanceof Error ? (error.stack ?? "") : ""}`;
    let hasSource = true;
    try {
      hasSource = Object.keys(readImportSource()).some(
        (key) => key !== "hex-command-choose-new-game",
      );
    } catch {
      /* Presence is unknown. Preserve it. */
    }
    return {
      game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
      pendingEnemy: false,
      ...(hasSource
        ? { importFailure: { source: "localStorage" as const, detail } }
        : { storageFailure: detail }),
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
            importFailure: { source: "indexedDB", detail: String(error) },
            notice:
              "Saved progress could not be updated. Review retained data in Repair saved data.",
          };
        }
      }
      return restored;
    } catch (error) {
      return {
        game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
        pendingEnemy: false,
        importFailure: {
          source: "indexedDB",
          detail: `${String(error)}\n${error instanceof Error ? (error.stack ?? "") : ""}`,
        },
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
