/** Historical serialized fixtures only; production migration runs in OCaml. */
import { Game } from "./game-fixture";
import { Catalog, catalog } from "./native-fixture";
import * as Save from "@frontend/Save";
import * as Legacy from "@frontend/Save_legacy";
import type { SavedGame } from "../rust/pkg/hex_command";
export const SAVE_KEY = Save.key;
export const SAVE_VERSION = 1;
export const importWaveMarker = Legacy.wave_marker;
export function saveBattle(game: Game, pendingEnemy = false): void {
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
    const [save] = Save.migrate(
      Catalog.raw(catalog),
      JSON.parse(localStorage.getItem(SAVE_KEY) ?? "null"),
    );
    const game = Game.fromCheckpoint(save.checkpoint as SavedGame);
    return { game, pendingEnemy: save.pending_enemy as boolean };
  } catch {
    return { game: new Game(), pendingEnemy: false };
  }
}
