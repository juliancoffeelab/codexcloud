import { flushSync, mount } from "svelte";
import { prepareOffline } from "./offline";
import Startup from "./Startup.svelte";
import "./style.css";
const target = document.getElementById("app");
if (!target) throw new Error("Missing application mount.");
flushSync(() => mount(Startup, { target }));
window.dispatchEvent(new Event("hex-command-shell-ready"));

void prepareOffline();
