import type { Game } from "./game-fixture";
import type { LiveGame } from "../src/game/live-game";
import type { GameView } from "../src/game/view";

/** Presentation fixtures publish the same detached Rust inspection as the browser. */
export function createGameView(source: Game | LiveGame): GameView {
  return source.publishedView();
}
