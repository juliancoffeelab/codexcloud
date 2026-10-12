/** Own preparation, the OCaml watcher and Vite for one development invocation.
 * Compiler errors remain visible and recover on the next edit. A stopped watcher
 * or server terminates its sibling; interruption releases their process groups.
 * Extra command-line arguments are forwarded to Vite unchanged.
 */
import { spawn, type ChildProcess } from "node:child_process";
import { fileURLToPath } from "node:url";

const cwd = fileURLToPath(new URL("../", import.meta.url));
const children = new Set<ChildProcess>();
let stopping = false;
let escalation: NodeJS.Timeout | undefined;

function signalGroup(child: ChildProcess, signal: NodeJS.Signals): void {
  if (!child.pid) return;
  try {
    process.kill(-child.pid, signal);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ESRCH") console.error(error);
  }
}

function stop(code: number): void {
  if (stopping) return;
  stopping = true;
  process.exitCode = code;
  for (const child of children) signalGroup(child, "SIGTERM");
  escalation = setTimeout(() => {
    for (const child of children) signalGroup(child, "SIGKILL");
  }, 2_000);
  escalation.unref();
}

function run(name: string, command: string, args: string[], watch = false): Promise<void> {
  if (stopping) return Promise.resolve();
  console.log(`[dev] ${name}`);
  const started = performance.now();
  const progress = watch
    ? undefined
    : setInterval(() => {
        console.log(
          `[dev] ${name} still running (${Math.floor((performance.now() - started) / 1000)}s).`,
        );
      }, 15_000);
  const child = spawn(command, args, { cwd, stdio: "inherit", detached: true });
  children.add(child);
  return new Promise((resolve, reject) => {
    let launchError: Error | undefined;
    child.on("error", (error) => {
      launchError = error;
    });
    child.on("close", (code, signal) => {
      if (progress) clearInterval(progress);
      // A child may exit while its helpers are still alive.
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
      "Dune is unavailable. Follow the Dune bootstrap commands in README.md, then retry npm run dev.",
    );
  }
  await run("Preparing Rust/WASM", "npm", ["run", "wasm:build"]);
  await run("Generating OCaml ABI bindings", "npm", ["run", "bindings:frontend"]);
  await run("Preparing artwork", "npm", ["run", "art:build"]);
  console.log(
    "[dev] First OCaml build can take several minutes: Dune downloads and compiles the locked compiler and libraries. Later builds reuse them.",
  );
  await run("Preparing OCaml", "dune", ["build", "@melange", "--display", "short"]);
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
