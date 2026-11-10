import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
const env = toolchainEnvironment();
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
function run(name, command, args, watch = false) {
  if (stopping) return Promise.resolve();
  console.log(`[dev] ${name}`);
  const started = performance.now();
  const progress = watch
    ? void 0
    : setInterval(() => {
        console.log(
          `[dev] ${name} still running (${Math.floor((performance.now() - started) / 1e3)}s).`,
        );
      }, 15e3);
  const child = spawn(command, args, { cwd, env, stdio: "inherit", detached: true });
  children.add(child);
  return new Promise((resolve, reject) => {
    let launchError;
    child.on("error", (error) => {
      launchError = error;
    });
    child.on("close", (code, signal) => {
      if (progress) clearInterval(progress);
      signalGroup(child, "SIGKILL");
      children.delete(child);
      if (!children.size && escalation) clearTimeout(escalation);
      if (stopping) resolve();
      else if (launchError || code !== 0 || signal || watch)
        reject(launchError ?? new Error(`${name} stopped (${signal ?? `exit ${code}`}).`));
      else resolve();
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
  try {
    await run("Checking Dune", "dune", ["--version"]);
  } catch {
    throw new Error(
      "Dune could not run after tool discovery. Check the error above and scripts/toolchain/toolchain-env.mjs; existing cloud tools need no shell installation.",
    );
  }
  await run("Preparing Rust/WASM", "npm", ["run", "wasm:build"]);
  await run("Preparing artwork", "npm", ["run", "art:build"]);
  console.log(
    "[dev] First OCaml build can take several minutes: Dune downloads and compiles the locked compiler and libraries. Later builds reuse them.",
  );
  await run("Preparing OCaml", "dune", ["build", "--root", cwd, "@melange", "--display", "short"]);
  if (!stopping) {
    console.log(
      "[dev] OCaml edits rebuild automatically; diagnostics stay in this terminal. CSS updates through Vite. Ctrl-C stops both.",
    );
    const services = [
      run(
        "Watching OCaml",
        "dune",
        [
          "build",
          "--root",
          cwd,
          "@melange",
          "--watch",
          "--display",
          "short",
          "--terminal-persistence",
          "preserve",
        ],
        true,
      ),
      run(
        "Starting Vite",
        process.execPath,
        [
          "node_modules/vite/bin/vite.js",
          "--host",
          "0.0.0.0",
          "--clearScreen",
          "false",
          ...process.argv.slice(2),
        ],
        true,
      ),
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
