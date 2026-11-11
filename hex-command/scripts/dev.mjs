import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
const cwd = fileURLToPath(new URL("../", import.meta.url));
const children = /* @__PURE__ */ new Set();
let stopping = false;
let escalation;
function signalGroup(child, signal) {
  if (!child.pid) return;
  try {
    process.kill(-child.pid, signal);
  } catch (error) {
    if (error.code !== "ESRCH") console.error(error);
  }
}
function stop(code) {
  if (stopping) return;
  stopping = true;
  process.exitCode = code;
  for (const child of children) signalGroup(child, "SIGTERM");
  escalation = setTimeout(() => {
    for (const child of children) signalGroup(child, "SIGKILL");
  }, 2e3);
  escalation.unref();
}
function run(name, command, args) {
  if (stopping) return Promise.resolve();
  console.log(`[dev] ${name}`);
  const child = spawn(command, args, { cwd, stdio: "inherit", detached: true });
  children.add(child);
  return new Promise((resolve, reject) => {
    let launchError;
    child.on("error", (error) => {
      launchError = error;
    });
    child.on("close", (code, signal) => {
      signalGroup(child, "SIGKILL");
      children.delete(child);
      if (!children.size && escalation) clearTimeout(escalation);
      if (stopping) resolve();
      else reject(launchError ?? new Error(`${name} stopped (${signal ?? `exit ${code}`}).`));
    });
  });
}
const onInterrupt = () => stop(130);
const onTerminate = () => stop(143);
process.on("SIGINT", onInterrupt);
process.on("SIGTERM", onTerminate);
try {
  if (process.platform === "win32")
    throw new Error("Run this development command in WSL; Dune setup is documented in README.md.");
  if (!stopping) {
    console.log(
      "[dev] OCaml edits rebuild automatically; diagnostics stay in this terminal. CSS updates through Vite. Ctrl-C stops both.",
    );
    const services = [
      run("Watching OCaml", "dune", [
        "build",
        "@melange",
        "@wasm-dev",
        "--root=.",
        "--cache=enabled",
        "@artwork",
        "--watch",
        "--display",
        "short",
        "--terminal-persistence",
        "preserve",
      ]),
      run("Starting Vite", process.execPath, [
        "node_modules/vite/bin/vite.js",
        "--host",
        "0.0.0.0",
        "--clearScreen",
        "false",
        ...process.argv.slice(2),
      ]),
    ];
    try {
      await Promise.all(services);
    } catch (error) {
      console.error(`[dev] ${error instanceof Error ? error.message : error}`);
      stop(1);
    } finally {
      stop(process.exitCode ? Number(process.exitCode) : 1);
      await Promise.allSettled(services);
    }
  }
} catch (error) {
  if (!stopping) console.error(`[dev] ${error instanceof Error ? error.message : error}`);
  stop(1);
} finally {
  process.off("SIGINT", onInterrupt);
  process.off("SIGTERM", onTerminate);
}
