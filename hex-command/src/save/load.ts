import { LiveGame } from "../game/live-game";
import type { GameEngineFactory } from "../game/native-engine";
import { migrateSave } from "./migrations";
import { readSave, wipeSave, storeSave } from "./storage";

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
      // Commit the validated data conversion directly, independently of gameplay
      // turn boundaries. Migration never waits for or executes enemy actions.
      if (migrated) storeSave(JSON.stringify(save));
      return { game, pendingEnemy: save.pendingEnemy };
    } catch {
      game?.dispose();
      wipeSave();
      const fresh = LiveGame.create(factory, { kind: "scenario", map: "mirror" });
      return {
        game: fresh,
        pendingEnemy: false,
        notice: "The saved battle could not be loaded and was cleared. A new battle is ready.",
      };
    }
  }
  return {
    game: LiveGame.create(factory, { kind: "scenario", map: "mirror" }),
    pendingEnemy: false,
  };
}
