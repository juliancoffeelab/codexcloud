import { LiveGame } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migration";
import { readSave, preserveRejectedSave } from "./storage";
import { saveBattle, suspendSaves } from "./write";

export interface LoadedBattle {
  game: LiveGame;
  pendingEnemy: boolean;
  notice?: string;
}
/** Parse, migrate, validate natively, then persist the successfully restored format. */
export function loadBattle(factory: GameEngineFactory): LoadedBattle {
  const raw = readSave();
  if (raw !== null) {
    let game: LiveGame | undefined;
    try {
      const { save, migrated } = migrateSave(JSON.parse(raw));
      game = LiveGame.restore(factory, save.checkpoint);
      if (migrated) saveBattle(game, save.pendingEnemy);
      return { game, pendingEnemy: save.pendingEnemy };
    } catch {
      game?.dispose();
      const preserved = preserveRejectedSave(raw);
      const fresh = LiveGame.create(factory, { kind: "scenario", map: "mirror" });
      if (!preserved) suspendSaves(fresh);
      return {
        game: fresh,
        pendingEnemy: false,
        notice: preserved
          ? "The saved battle could not be restored. A new battle is ready."
          : "The saved battle could not be restored. The original is retained; start a new battle to enable saving.",
      };
    }
  }
  return {
    game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
    pendingEnemy: false,
  };
}
