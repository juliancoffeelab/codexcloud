import { LiveGame } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migrations";
import { wipeSave, storeSave, SAVE_KEY } from "./storage";
import type { BattleSave } from "./schema";

export interface LoadedBattle {
  game: LiveGame;
  pendingEnemy: boolean;
  save?: BattleSave;
  notice?: string;
  storageFailure?: string;
}
/** Migration is data-only. Validate both recovery and move undo through the native engine. */
export function restoreBattleText(
  factory: GameEngineFactory,
  raw: string,
): LoadedBattle & { save: BattleSave; migrated: boolean } {
  const { save, migrated } = migrateSave(JSON.parse(raw));
  const game = LiveGame.restore(factory, save.checkpoint, save.setup);
  try {
    if (migrated) delete save.undo;
    for (const entry of save.undo ?? []) {
      if (
        entry.round !== game.round ||
        entry.state.round !== game.round ||
        entry.state.turn !== "player" ||
        game.turn !== "player" ||
        game.winner ||
        !entry.state.units.some((unit) => unit.id === entry.unitId)
      )
        throw new Error("Invalid saved move history");
      const previous = LiveGame.restore(
        factory,
        { ...save.checkpoint, state: entry.state as unknown as BattleSave["checkpoint"]["state"] },
        save.setup,
      );
      previous.dispose();
    }
    return { game, pendingEnemy: save.pendingEnemy, save, migrated };
  } catch (error) {
    game.dispose();
    throw error;
  }
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
            notice:
              "Saved progress needs an update, but storage refused the write. Free storage in Saved progress, then reload to restore it.",
          };
        }
      }
      return restored;
    } catch {
      wipeSave();
      return {
        game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
        pendingEnemy: false,
        notice: "The saved battle could not be loaded and was cleared. A new battle is ready.",
      };
    }
  }
  return {
    game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
    pendingEnemy: false,
    storageFailure,
  };
}
