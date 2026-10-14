import { nativeValue, type LiveGame } from "../game/live-game";
import { currentSave, SAVE_VERSION } from "./schema";
import { storeSave } from "./storage";

/** All writes use the latest envelope; this path contains no compatibility handling. */
export function saveBattle(game: LiveGame, pendingEnemy = false): void {
  if (game.turn === "enemy" && !game.winner) return;
  const checkpoint = nativeValue(game.checkpoint());
  const save = currentSave({ version: SAVE_VERSION, checkpoint, pendingEnemy });
  storeSave(JSON.stringify(save));
}
