import { decodeUndo } from "./undo";
import { LiveGame, nativeValue } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migrations";
import { storeSave, readSave, readSession } from "./storage";
import { importLocalStorage, readImportSource, ImportStageError } from "./import";
import type { LoadProblem } from "./repair-types";
import { saveFailure } from "./failure";
import { currentFlashSave, currentTurn } from "./schema";
import type { CurrentEntry } from "./metadata";
import type { BattleSave } from "./schema";

interface BattleState {
  game: LiveGame;
  pendingEnemy: boolean;
  notice?: string;
}
export interface RestoredBattle extends BattleState {
  status: "restored";
  save: BattleSave;
  undo: import("../game/undo").SavedUndoEntry[];
}
/** An unsuccessful read can never be represented as a confirmed empty repository. */
export type LoadedBattle = BattleState &
  (
    | (RestoredBattle & { problem?: never; chooseNewGame?: never })
    | { status: "empty"; chooseNewGame: boolean; save?: never; undo?: never; problem?: never }
    | { status: "blocked"; problem: LoadProblem; save?: never; undo?: never; chooseNewGame?: never }
  );
function blocked(
  factory: GameEngineFactory,
  source: LoadProblem["source"],
  kind: LoadProblem["kind"],
  error: unknown,
): LoadedBattle {
  return {
    status: "blocked",
    game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
    pendingEnemy: false,
    problem: {
      source,
      kind,
      detail: `${String(error)}\n${error instanceof Error ? (error.stack ?? "") : ""}`,
    },
  };
}
function accessKind(error: unknown): "read" | "blocked" | "full" {
  const failure = saveFailure(error);
  return failure.kind === "internal" ? "read" : failure.kind;
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
): RestoredBattle & { migrated: boolean } {
  const { save: converted, migrated } = decodeBattleText(raw);
  const save = kind === "flash" ? currentFlashSave(converted) : currentTurn(converted);
  if (save.undo?.length) {
    if (save.pendingEnemy) throw new Error("Pending enemy turns cannot retain move undo.");
    nativeValue(factory.validateUndo(save.checkpoint.state, save.undo));
  }
  const undo = decodeUndo(save.checkpoint.state, save.undo ?? []);
  const game = LiveGame.restore(factory, save.checkpoint, save.setup);
  return { status: "restored", game, pendingEnemy: save.pendingEnemy, save, migrated, undo };
}
/** Parse, migrate, validate natively, then persist the successfully restored format. */
export async function loadBattle(factory: GameEngineFactory): Promise<LoadedBattle> {
  try {
    await initializeRepository(factory);
  } catch (error) {
    const stage = error instanceof ImportStageError ? error.stage : "access";
    const reason = error instanceof ImportStageError ? error.original : error;
    let source: LoadProblem["source"] = "indexedDB";
    try {
      if (Object.keys(readImportSource()).some((key) => key !== "hex-command-choose-new-game"))
        source = "localStorage";
    } catch {
      source = "localStorage";
    }
    return blocked(factory, source, stage === "data" ? "data" : accessKind(reason), reason);
  }
  let session: Awaited<ReturnType<typeof readSession>>;
  try {
    session = await readSession();
  } catch (error) {
    return blocked(factory, "indexedDB", accessKind(error), error);
  }
  if (session.text === null)
    return {
      status: "empty",
      game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
      pendingEnemy: false,
      chooseNewGame: session.chooseNewGame,
    };
  let restored: ReturnType<typeof restoreBattleText>;
  try {
    restored = restoreBattleText(factory, session.text);
  } catch (error) {
    return blocked(factory, "indexedDB", "data", error);
  }
  if (restored.migrated) {
    try {
      await storeSave(JSON.stringify(restored.save));
    } catch (error) {
      restored.game.dispose();
      return blocked(factory, "indexedDB", accessKind(error), error);
    }
  }
  return restored;
}
