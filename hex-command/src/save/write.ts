import { nativeValue, type LiveGame } from "../game/live-game";
import { currentSave, SAVE_VERSION } from "./schema";
import { storeSave } from "./storage";

/** All writes use the latest envelope; this path contains no compatibility handling. */
export function saveBattle(game: LiveGame, pendingEnemy = false): void {
  if (game.turn === "enemy" && !game.winner && !(game.round === 1 && game.setup.first === "enemy"))
    return;
  const checkpoint = nativeValue(game.checkpoint());
  const save = currentSave({ version: SAVE_VERSION, checkpoint, pendingEnemy, setup: game.setup });
  storeSave(JSON.stringify(save));
}
