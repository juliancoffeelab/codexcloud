// OCaml owns rendering; Dune owns the incremental build; Node owns file watches and runtime lifetime.
import { build } from "esbuild";
import { spawn } from "node:child_process";
import { readdirSync, watchFile, unwatchFile } from "node:fs";
import { mkdir, writeFile, rename } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Worker } from "node:worker_threads";
import { frontendAliases, frontendModulePath } from "./frontend-paths.mjs";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
const root = fileURLToPath(new URL("../", import.meta.url));
const watching = process.argv.includes("--watch") && !process.argv.includes("--help");
const args = process.argv.slice(2).filter((arg) => arg !== "--watch");
const output = resolve(root, "tmp/map-draft/runner.mjs");
const inputPaths = [
  "--map",
  "--roster",
  "--scenario",
  "--content",
  "--overlay",
  "--artwork",
].flatMap((flag) => {
  const index = args.indexOf(flag);
  return index >= 0 && args[index + 1] ? [resolve(args[index + 1])] : [];
});
const outputIndex = args.indexOf("--output");
const destination = outputIndex >= 0 ? resolve(args[outputIndex + 1] ?? "") : "Preview";

function sourceFiles() {
  const files = [
    "frontend/dune",
    "dune-project",
    "dune-workspace",
    "dune.lock/lock.dune",
    "package.json",
    "package-lock.json",
    "scripts/render-map.mjs",
    "scripts/render-map-worker.mjs",
    "scripts/frontend-paths.mjs",
    "scripts/generate-json-codecs.mjs",
    "scripts/generate-resource-visitors.mjs",
    frontendModulePath("Domain"),
    frontendModulePath("Domain_resource_walk"),
    "_build/default/artwork/pixel-art.mjs",
    "_build/default/wasm/dev/backend.wasm",
    "_build/default/wasm/dev/schema.json",
    "_build/default/wasm/dev/bindings.runtime.mjs",
  ];
  const walk = (dir) => {
    for (const entry of readdirSync(resolve(root, dir), { withFileTypes: true })) {
      const path = `${dir}/${entry.name}`;
      if (entry.isDirectory()) walk(path);
      else if (
        (/\.(ml|mli|re|rei)$/.test(entry.name) && !entry.name.startsWith("Fixture_json.")) ||
        entry.name === "dune"
      )
        files.push(path);
    }
  };
  for (const dir of ["frontend", "packages/save-migrations/ocaml"]) walk(dir);
  return [...new Set(files)].map((file) => resolve(root, file)).sort();
}
async function prepare() {
  for (const [command, args] of [
    ["bash", ["scripts/build-wasm.sh", "dev"]],
    ["dune", ["build", "--root=.", "@melange", "@wasm-dev", "@artwork"]],
  ])
    await new Promise((ok, fail) => {
      const child = spawn(command, args, {
        cwd: root,
        env: toolchainEnvironment(),
        stdio: "inherit",
      });
      child.once("error", fail);
      child.once("exit", (code) =>
        code === 0 ? ok() : fail(new Error(`OCaml compilation exited ${code}`)),
      );
    });
  await mkdir(resolve(root, "tmp/map-draft"), { recursive: true });
  const compiled = await build({
    stdin: {
      contents: `export { main, dependencies } from './_build/default/frontend/output/frontend/devtools/Map_draft.js'; export { initialize } from './_build/default/wasm/dev/bindings.runtime.mjs';`,
      resolveDir: root,
    },
    alias: frontendAliases,
    bundle: true,
    platform: "node",
    format: "esm",
    target: "es2022",
    packages: "external",
    write: false,
    outfile: output,
  });
  // Readers only see complete bundles, including concurrent one-shot invocations.
  const temporary = `${output}.${process.pid}.tmp`;
  await writeFile(temporary, compiled.outputFiles[0].contents);
  await rename(temporary, output);
  console.log("Prepared OCaml map renderer.");
}

let worker,
  closed = false,
  busy = false,
  pending = false,
  codeDirty = true,
  debounce;
let watched = new Map(),
  dataPaths = new Set(inputPaths);
function receive(expected) {
  return new Promise((ok, fail) => {
    const current = worker;
    const cleanup = () => {
      current.off("message", message);
      current.off("error", error);
      current.off("exit", exit);
    };
    const message = (value) => {
      if (value.type === expected || value.type.endsWith("-error")) {
        cleanup();
        ok(value);
      }
    };
    const error = (reason) => {
      cleanup();
      fail(reason);
    };
    const exit = (code) => {
      cleanup();
      fail(new Error(`Renderer worker exited ${code}`));
    };
    current.on("message", message);
    current.once("error", error);
    current.once("exit", exit);
  });
}
async function restart() {
  if (worker) await worker.terminate();
  worker = new Worker(new URL("./render-map-worker.mjs", import.meta.url), {
    workerData: { root, output },
    argv: args,
  });
  const ready = await receive("ready");
  if (ready.type !== "ready") throw new Error(ready.error);
  console.log("Renderer runtime ready.");
}
const knownCodePaths = new Set();

function watches() {
  if (!watching || closed) return;
  const desired = new Map([...dataPaths].map((path) => [path, false]));
  for (const path of sourceFiles()) knownCodePaths.add(path);
  for (const path of knownCodePaths) desired.set(path, true);
  for (const [path, listener] of watched)
    if (!desired.has(path)) {
      unwatchFile(path, listener);
      watched.delete(path);
    }
  for (const [path, code] of desired) {
    if (watched.has(path)) continue;
    const listener = (now, before) => {
      if (
        now.mtimeMs !== before.mtimeMs ||
        now.ctimeMs !== before.ctimeMs ||
        now.size !== before.size ||
        now.ino !== before.ino
      )
        schedule(code);
    };
    watchFile(path, { interval: 80 }, listener);
    watched.set(path, listener);
  }
}
function schedule(code = false) {
  if (closed) return;
  pending = true;
  codeDirty ||= code;
  clearTimeout(debounce);
  debounce = setTimeout(drain, 35);
}
async function drain() {
  if (busy || closed) return;
  busy = true;
  try {
    while (pending && !closed) {
      pending = false;
      try {
        if (codeDirty || !worker) {
          await prepare();
          if (closed) break;
          await restart();
          codeDirty = false;
        }
        if (closed) break;
        const done = receive("rendered");
        worker.postMessage({ type: "render" });
        const result = await done;
        const dependencies = result.dependencies ?? [];
        dataPaths = new Set([
          ...inputPaths,
          ...dependencies,
          ...(result.type === "render-error" ? dataPaths : []),
        ]);
        watches();
        if (result.type === "render-error") throw new Error(result.error);
        if (!args.includes("--help")) console.log(`${destination} (${result.ms.toFixed(0)} ms)`);
      } catch (error) {
        if (closed) break;
        console.error(error.stack ?? error);
        if (!watching) process.exitCode = 1;
        else console.error("Waiting for the next edit; the previous PNG is unchanged.");
      }
    }
  } finally {
    busy = false;
    if (!watching) await stop();
    else {
      watches();
      if (pending) schedule();
    }
  }
}
async function stop() {
  closed = true;
  clearTimeout(debounce);
  for (const [path, listener] of watched) unwatchFile(path, listener);
  watched.clear();
  if (worker) {
    await worker.terminate();
    worker = undefined;
  }
}
for (const signal of ["SIGINT", "SIGTERM"])
  process.once(signal, () => {
    void stop();
  });
watches();
if (watching)
  console.log("Watching draft files, used artwork and renderer sources. Ctrl+C to stop.");
pending = true;
await drain();
