import { nativeValue, type LiveGame } from "../game/live-game";
import { SAVE_VERSION } from "./schema";
import { validSetup } from "../game/setup";
import { storeSave } from "./storage";

/** All writes use the latest envelope; this path contains no compatibility handling. */
export function saveBattle(game: LiveGame, pendingEnemy = false): void {
  if (game.turn === "enemy" && !game.winner && !(game.round === 1 && game.setup.first === "enemy"))
    return;
  if (
    typeof pendingEnemy !== "boolean" ||
    !validSetup(game.setup) ||
    (pendingEnemy && (game.turn !== "player" || game.winner !== null))
  )
    throw new Error("Invalid current save envelope");
  // Rust serializes its complete canonical SavedGame. Do not materialize,
  // clone/freeze and stringify the full map again just to write those bytes.
  const checkpoint = nativeValue(game.serializedCheckpoint());
  const metadata = JSON.stringify({ version: SAVE_VERSION, pendingEnemy, setup: game.setup });
  storeSave(`${metadata.slice(0, -1)},"checkpoint":${checkpoint}}`);
}
