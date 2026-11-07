/** Construct only after the boot owner has initialized the validated Rust catalog. */
import App from "./App.svelte";
import { BattleController } from "./controller/battle";
import type { GameEngineFactory } from "./game/native-engine";

import { HudArtworkCache } from "./presentation/hud-artwork";

export async function createBattleApplication(factory: GameEngineFactory) {
  const controller = await BattleController.create(undefined, factory);
  const previews = new HudArtworkCache();
  return { component: App, controller, previews };
}
