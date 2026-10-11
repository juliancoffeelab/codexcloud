import type { Game } from "./game-fixture";
/** Presentation receives the actual detached OCaml publication used in production. */
export function createGameView(source: Game) {
  return source.publishedView();
}
