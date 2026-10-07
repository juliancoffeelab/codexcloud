import { flushSync, mount } from "svelte";
import App from "./App.svelte";
import { BattleController } from "./controller/battle";
import { GAME_ENGINE_FACTORY } from "./game/authored-content";
import "./style.css";
const target = document.getElementById("app");
if (!target) throw new Error("Missing application mount.");
async function start() {
  const factory = GAME_ENGINE_FACTORY;
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
