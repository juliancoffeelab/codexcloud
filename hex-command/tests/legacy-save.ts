/** Fixtures for published version-one saves; never used by the browser. */
import { Game } from "./game-fixture";
import type { LiveGame } from "../src/game/live-game";
import { migrateSave } from "../src/save/migrations";
import { saveBattle as writeCurrent } from "../src/save/write";
export { importWaveMarker } from "../src/save/migrations/legacy/snapshot";
export { SAVE_KEY } from "../src/save/storage";
import { SAVE_KEY } from "../src/save/storage";
export const SAVE_VERSION = 1;
export function saveBattle(game: Game | LiveGame, pendingEnemy = false): void {
  if (!(game instanceof Game)) {
    writeCurrent(game, pendingEnemy);
    return;
  }
  const { tiles, cols: _cols, rows: _rows, lastCombat: _combat, ...state } = game.captureState();
  const villages = tiles
    .filter((tile) => tile.terrain === "village")
    .map(({ id, owner }) => ({ id, owner }));
  const sites = tiles
    .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
    .map(({ id, owner }) => ({ id, owner }));
  localStorage.setItem(
    SAVE_KEY,
    JSON.stringify({
      version: 1,
      campaignDeploymentFormat: "named-waves-v1",
      pendingEnemy,
      state: { ...state, villages, ...(state.campaign || sites.length ? { sites } : {}) },
    }),
  );
}
export function loadBattle() {
  try {
    const { save } = migrateSave(JSON.parse(localStorage.getItem(SAVE_KEY) ?? "null"));
    const game = Game.fromCheckpoint(save.checkpoint);
    return { game, pendingEnemy: save.pendingEnemy };
  } catch {
    return { game: new Game(), pendingEnemy: false };
  }
}
