/** Loaded behind the mounted shell; the validated catalog owns WASM initialization. */
import App from "./App.svelte";
import { BattleController } from "./controller/battle";
import { GAME_ENGINE_FACTORY } from "./game/authored-content";

export function createBattleApplication() {
  try {
    return {
      component: App,
      controller: new BattleController(undefined, GAME_ENGINE_FACTORY),
    };
  } catch (error) {
    GAME_ENGINE_FACTORY.dispose();
    throw error;
  }
}
