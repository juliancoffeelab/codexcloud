import { nativeValue, type LiveGame } from "../game/live-game";
import { currentSave, SAVE_VERSION } from "./schema";
import { storeSave } from "./storage";

const suspended = new WeakSet<LiveGame>();
/** Keep a rejected original untouched if its backup could not be stored. */
export function suspendSaves(game: LiveGame): void {
  suspended.add(game);
}

/** All writes use the latest envelope; this path contains no compatibility handling. */
export function saveBattle(game: LiveGame, pendingEnemy = false): void {
  if (suspended.has(game) || (game.turn === "enemy" && !game.winner)) return;
  const checkpoint = nativeValue(game.checkpoint());
  const save = currentSave({ version: SAVE_VERSION, checkpoint, pendingEnemy });
  storeSave(JSON.stringify(save));
}
