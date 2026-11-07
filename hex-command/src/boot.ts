import { prepareHudSound, disposeHudSound } from "./presentation/sound";
import wasmUrl from "../rust/pkg/hex_command_bg.wasm?url";
import { createNativeEngineFactory, type GameEngineFactory } from "./game/native-engine";
import { startEngine } from "./game/engine-startup";
import type { createBattleApplication } from "./battle-app";

type Application = Awaited<ReturnType<typeof createBattleApplication>>;

/** Critical downloads begin together; the HUD and renderer have separate readiness. */
export function startBoot() {
  const abort = new AbortController();
  const hudAudio = prepareHudSound();
  let disposed = false;
  let factory: GameEngineFactory | undefined;
  let application: Application | undefined;
  let renderer: import("./presentation/phaser-battlefield").PreparedBattlefield | undefined;
  const alive = () => {
    if (disposed) throw new DOMException("Application boot was abandoned.", "AbortError");
  };
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    abort.abort();
    disposeHudSound();
    try {
      renderer?.dispose();
      application?.controller.dispose();
      application?.previews.dispose();
    } finally {
      factory?.dispose();
    }
  };
  // Only critical engine/HUD bytes compete initially. The renderer starts as soon
  // as the engine is ready, before catalog evaluation, save restoration or mount.
  const engine = startEngine(async () => {
    const response = fetch(wasmUrl, { signal: abort.signal });
    const bindings = import("../rust/pkg/hex_command");
    const [module, bytes] = await Promise.all([bindings, response]);
    if (!bytes.ok) throw new Error(`WASM download failed: ${bytes.status} ${bytes.url}`);
    await module.default({ module_or_path: bytes });
    alive();
    try {
      factory = createNativeEngineFactory(new module.NativeEngineFactory(), {
        responseOwnership: "transferred",
      });
    } catch (cause) {
      throw new Error("Cannot create the game engine.", { cause });
    }
    return factory;
  });
  const battlefield = engine.then(() => {
    alive();
    return import("./presentation/phaser-battlefield").then((module) => {
      alive();
      renderer = module.preparePhaserBattlefield();
      return renderer;
    });
  });
  // A renderer failure has its own recovery UI and must not dispose a mounted HUD.
  void battlefield.catch(() => {});
  const module = import("./battle-app");
  const ready = Promise.all([engine, module, hudAudio])
    .then(async ([native, loaded]) => {
      alive();
      const initialized = await loaded.createBattleApplication(native);
      if (disposed) {
        initialized.controller.dispose();
        initialized.previews.dispose();
        alive();
      }
      application = initialized;
      return { ...initialized, battlefield };
    })
    .catch((error) => {
      dispose();
      throw error;
    });
  void ready.catch(() => {});
  return { ready, dispose };
}
export type ApplicationBoot = ReturnType<typeof startBoot>;
