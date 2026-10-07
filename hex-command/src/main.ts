import { flushSync, mount } from "svelte";
import App from "./App.svelte";
import { BattleController } from "./controller/battle";
import { loadNativeEngineFactory } from "./game/native-engine";
import { nativeValue } from "./game/live-game";
import "./style.css";
const target = document.getElementById("app");
if (!target) throw new Error("Missing application mount.");
async function start() {
  const factory = nativeValue(
    await loadNativeEngineFactory(() => import("../rust/pkg/hex_command")),
  );
  try {
    const controller = new BattleController(undefined, factory);
    flushSync(() => mount(App, { target: target!, props: { controller } }));
    window.dispatchEvent(new Event("hex-command-ready"));
  } catch (error) {
    factory.dispose();
    throw error;
  }
}
void start();
