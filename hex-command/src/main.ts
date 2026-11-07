import { flushSync, mount } from "svelte";
import Startup from "./Startup.svelte";
import "./style.css";
import { startBoot } from "./boot";
const target = document.getElementById("app");
if (!target) throw new Error("Missing application mount.");
const boot = startBoot();
try {
  flushSync(() => mount(Startup, { target, props: { boot } }));
} catch (error) {
  boot.dispose();
  throw error;
}
window.dispatchEvent(new Event("hex-command-shell-ready"));
