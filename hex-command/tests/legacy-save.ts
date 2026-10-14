/** Fixtures for published version-one saves; never used by the browser. */
import { Game } from "./game-fixture";
import { nativeValue, type LiveGame } from "../src/game/live-game";
import { loadBattle as loadCurrent } from "../src/save/load";
import { saveBattle as writeCurrent } from "../src/save/write";
import { createTestFactory } from "./native-fixture";
import type { SavedGame } from "../rust/pkg/hex_command";
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
  const loaded = loadCurrent(createTestFactory());
  try {
    return {
      game: Game.fromCheckpoint(
        structuredClone(nativeValue(loaded.game.checkpoint())) as SavedGame,
      ),
      pendingEnemy: loaded.pendingEnemy,
    };
  } finally {
    loaded.game.dispose();
  }
}
