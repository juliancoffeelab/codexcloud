import { flushSync, mount } from "svelte";
import App from "./App.svelte";
import "./style.css";
const target = document.getElementById("app");
if (!target) throw new Error("Missing application mount.");
flushSync(() => mount(App, { target }));
window.dispatchEvent(new Event("hex-command-ready"));
