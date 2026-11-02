/** Loaded behind the mounted shell; the validated catalog owns WASM initialization. */
import { get } from "svelte/store";
import { preloadTerrainView } from "./presentation/terrain-preload";
import App from "./App.svelte";
import { BattleController } from "./controller/battle";
import { GAME_ENGINE_FACTORY } from "./game/authored-content";

export function createBattleApplication() {
  try {
    const controller = new BattleController(undefined, GAME_ENGINE_FACTORY);
    const view = get(controller.view);
    preloadTerrainView(view.game, view.artwork);
    return { component: App, controller };
  } catch (error) {
    GAME_ENGINE_FACTORY.dispose();
    throw error;
  }
}
