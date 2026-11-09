import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { availableParallelism } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
const root = fileURLToPath(new URL("../", import.meta.url));
const preparedPath = join(root, "tmp/verification-prepared.json");
const advisoryMs = 20e3;
const commandStarted = performance.now();
const cpuSlots = Math.min(6, availableParallelism());
const rustWorkers = Math.min(2, Math.max(1, Math.floor(cpuSlots / 2)));
const frontendWorkers = Math.min(3, Math.max(1, cpuSlots - rustWorkers));
function nodeTask(name, script, args = [], slots = 1, resource) {
  return {
    name,
    command: process.execPath,
    args: [script, ...args],
    slots,
    resource,
  };
}
function sourceFingerprint() {
  const files = [
    "rust/Cargo.toml",
    "rust/Cargo.lock",
    "../ocaml-js-bindgen/Cargo.toml",
    "../ocaml-js-bindgen/Cargo.lock",
    "rust/build.rs",
    ".dune-version",
    "scripts/install-dune.sh",
    "dune-project",
    "dune",
    "dune-workspace",
    "scripts/build-pixel-art.py",
    "scripts/optimize-wasm.mjs",
    "src/presentation/pixel-art.mjs",
    "Makefile",
    "scripts/toolchain/packager.mjs",
    "scripts/toolchain/toolchain-env.mjs",
    "scripts/fixture-json.mjs",
    "scripts/generate-json-codecs.mjs",
    "scripts/check-migration-contract.mjs",
    "scripts/check-migrations.mjs",
    "scripts/migration_import_check.ml",
    "scripts/dune",
    "scripts/terrain-prebuild.mjs",
    "scripts/frontend-paths.mjs",
    "package.json",
    "package-lock.json",
    "index.html",
    "src/style.css",
    "vite.config.mjs",
  ];
  function walk(directory) {
    for (const entry of readdirSync(join(root, directory), { withFileTypes: true })) {
      const path = `${directory}/${entry.name}`;
      if (entry.isDirectory()) walk(path);
      else if (entry.isFile()) files.push(path);
    }
  }
  for (const directory of [
    "rust/src",
    "rust/catalog",
    "rust/battle",
    "rust/freeze-derive",
    "../ocaml-js-bindgen/crates",
    "rust/data",
    "frontend",
    "tests/ocaml",
    "pixel/art",
    "src/save",
    "packages/save-migrations",
    "dune.lock",
    "public",
  ]) {
    if (existsSync(join(root, directory))) walk(directory);
  }
  const hash = createHash("sha256");
  for (const path of files.sort()) {
    hash
      .update(path)
      .update("\0")
      .update(readFileSync(join(root, path)))
      .update("\0");
  }
  return hash.digest("hex");
}
function artifactFingerprint(binaries) {
  const files = [...binaries];
  function walk(directory, matches) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) walk(path, matches);
      else if (entry.isFile() && matches(path)) files.push(path);
    }
  }
  walk(join(root, "_build/default/frontend/output"), (path) => path.endsWith(".js"));
  walk(join(root, "rust/pkg"), (path) => /\.(?:js|mjs|wasm|json)$/.test(path));
  const sprites = join(root, "public/assets/sprites");
  if (!existsSync(sprites) || !readdirSync(sprites).some((name) => name.endsWith(".png")))
    throw new Error("Prepared artwork is missing.");
  walk(sprites, (path) => path.endsWith(".png"));
  if (!existsSync(join(root, "dist/index.html")))
    throw new Error("Prepared production bundle is missing.");
  walk(join(root, "dist"), () => true);
  const hash = createHash("sha256");
  for (const path of [...new Set(files)].sort()) {
    hash.update(path).update("\0").update(readFileSync(path)).update("\0");
  }
  return hash.digest("hex");
}
let currentPrepared;
function prepared() {
  if (currentPrepared) return currentPrepared;
  if (!existsSync(preparedPath)) throw new Error("Verification inputs have not been prepared.");
  const result = JSON.parse(readFileSync(preparedPath, "utf8"));
  if (typeof result.artifactFingerprint !== "string" || !Array.isArray(result.binaries)) {
    throw new Error("Preparation manifest is outdated.");
  }
  if (result.fingerprint !== sourceFingerprint()) {
    throw new Error("Prepared artifacts are stale after source changes.");
  }
  if (
    !result.binaries.length ||
    result.binaries.some((path) => !existsSync(path)) ||
    !existsSync(join(root, "rust/pkg/backend.wasm")) ||
    !existsSync(join(root, "_build/default/frontend/output/frontend/Main.js"))
  ) {
    throw new Error("Prepared artifacts are missing.");
  }
  if (result.artifactFingerprint !== artifactFingerprint(result.binaries)) {
    throw new Error("Prepared artifacts have changed.");
  }
  currentPrepared = result;
  return result;
}
function pythonTasks() {
  return [
    ["python-art", "scripts", "test_pixel_*.py"],
    ["python-reports", "scripts", "test_simulation_reports.py"],
    ["python-generation", ".agents/skills/pinkbox-generation/tests", "test*.py"],
    ["python-cleaning", ".agents/skills/pixel-cleaning/tests", "test*.py"],
    ["python-analysis", ".agents/skills/combat-simulation/tests", "test*.py"],
  ].map(([name, directory, pattern]) => ({
    name,
    command: "python3",
    args: ["-m", "unittest", "discover", "-s", directory, "-p", pattern],
  }));
}
function rustTasks() {
  prepared();
  return [
    {
      // Cargo supplies the toolchain library path required by proc-macro tests.
      // Preparation above still guarantees these executables are already built.
      name: "rust",
      command: "cargo",
      args: [
        "test",
        "--offline",
        "--locked",
        "--manifest-path",
        "rust/Cargo.toml",
        "--workspace",
        "--features",
        "wasm",
        "--lib",
        "--bins",
        "--tests",
        "--",
        `--test-threads=${rustWorkers}`,
      ],
      slots: rustWorkers,
      resource: "cargo-tests",
    },
    // Rustdoc owns discovery and compilation of documentation examples. This is
    // retained even when there are currently no examples, so coverage cannot drift.
    {
      name: "rust-doc",
      resource: "cargo-tests",
      command: "cargo",
      args: [
        "test",
        "--offline",
        "--locked",
        "--manifest-path",
        "rust/Cargo.toml",
        "--features",
        "wasm",
        "--doc",
        "--workspace",
      ],
    },
  ];
}
function frontendTask(args = []) {
  return {
    name: "frontend",
    command: process.execPath,
    args: [
      join(root, "node_modules/vitest/vitest.mjs"),
      "run",
      ...args,
      "--maxWorkers",
      String(frontendWorkers),
    ],
    slots: frontendWorkers,
  };
}
function allTestTasks(args = []) {
  return [
    frontendTask(args),
    nodeTask("migration-contract", "scripts/check-migration-contract.mjs"),
    ...rustTasks(),
    ...pythonTasks(),
  ];
}
async function runTasks(tasks) {
  const queue = tasks.slice();
  const running = /* @__PURE__ */ new Map();
  const failures = [];
  const env = toolchainEnvironment();
  let occupied = 0;
  let stopped = false;
  function kill(child) {
    if (!child.pid) return;
    try {
      if (process.platform === "win32") child.kill("SIGKILL");
      else process.kill(-child.pid, "SIGKILL");
    } catch (error) {
      if (error.code !== "ESRCH") console.error(error);
    }
  }
  function stop(reason) {
    if (stopped) return;
    stopped = true;
    failures.push(reason);
    for (const task of queue.splice(0)) failures.push(`${task.name} (not run)`);
    for (const child of running.keys()) kill(child);
  }
  const interrupt = () => stop("Interrupted");
  process.on("SIGINT", interrupt);
  process.on("SIGTERM", interrupt);
  const result = await new Promise((complete) => {
    function schedule() {
      if (!stopped) {
        for (let index = 0; index < queue.length && running.size < 3;) {
          const task = queue[index];
          const weight = Math.min(cpuSlots, task.slots ?? 1);
          const resourceBusy =
            task.resource &&
            [...running.values()].some((active) => active.resource === task.resource);
          if (occupied + weight > cpuSlots || resourceBusy) {
            index++;
            continue;
          }
          queue.splice(index, 1);
          occupied += weight;
          const started = performance.now();
          console.log(`RUN: ${task.name}`);
          const child = spawn(task.command, task.args, {
            cwd: root,
            env,
            stdio: "inherit",
            detached: process.platform !== "win32",
          });
          running.set(child, task);
          let launchError;
          child.on("error", (error) => {
            launchError = error;
            console.error(error.message);
          });
          child.on("close", (code, signal) => {
            running.delete(child);
            occupied -= weight;
            kill(child);
            const passed = code === 0 && !signal && !launchError && !stopped;
            console.log(
              `${passed ? "PASS" : "FAIL"}: ${task.name} (${((performance.now() - started) / 1e3).toFixed(2)}s)`,
            );
            if (!passed) failures.push(task.name);
            schedule();
          });
        }
      }
      if (!queue.length && !running.size) complete(failures.length ? 1 : 0);
    }
    schedule();
  });
  process.off("SIGINT", interrupt);
  process.off("SIGTERM", interrupt);
  if (failures.length) console.error(`FAIL: ${[...new Set(failures)].join(", ")}`);
  console.log(
    `Finished ${tasks.length} checks in ${((performance.now() - commandStarted) / 1e3).toFixed(2)}s`,
  );
  return result;
}
async function verifyTasks(makeTasks, needsPreparation = true) {
  const timer = setTimeout(
    () => console.warn("WARN: verification exceeded the advisory 20-second budget; continuing."),
    Math.max(0, advisoryMs - (performance.now() - commandStarted)),
  );
  try {
    if (needsPreparation) {
      try {
        prepared();
      } catch {
        console.log("Preparing changed or missing verification inputs...");
        const code = await runTasks([
          {
            name: "Build verification inputs",
            command: "make",
            args: ["prepare"],
            slots: cpuSlots,
          },
        ]);
        if (code !== 0) throw new Error("Verification input build failed");
        currentPrepared = undefined;
        prepared();
      }
    }
    return await runTasks(makeTasks());
  } finally {
    clearTimeout(timer);
  }
}
function beginPreparation() {
  mkdirSync(dirname(preparedPath), { recursive: true });
  rmSync(preparedPath, { force: true });
  writeFileSync(
    join(root, "tmp/verification-start.json"),
    JSON.stringify({ fingerprint: sourceFingerprint() }) + "\n",
  );
}
function recordPreparation() {
  const fingerprint = sourceFingerprint();
  const startedPath = join(root, "tmp/verification-start.json");
  if (JSON.parse(readFileSync(startedPath, "utf8")).fingerprint !== fingerprint)
    throw new Error("Sources changed during preparation. Rerun verification.");
  const binaries = [];
  for (const line of readFileSync(join(root, "tmp/native-test-artifacts.jsonl"), "utf8")
    .split("\n")
    .filter(Boolean)) {
    const artifact = JSON.parse(line);
    if (artifact.reason === "compiler-artifact" && artifact.profile?.test && artifact.executable)
      binaries.push(artifact.executable);
  }
  if (!binaries.length) throw new Error("Cargo did not publish any test executables");
  const uniqueBinaries = [...new Set(binaries)];
  writeFileSync(
    preparedPath,
    JSON.stringify({
      fingerprint,
      artifactFingerprint: artifactFingerprint(uniqueBinaries),
      binaries: uniqueBinaries,
    }) + "\n",
  );
  rmSync(startedPath);
  console.log("Prepared current artifacts. Run make test or make verify.");
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length === 1 && args[0] === "--begin-preparation") beginPreparation();
    else if (args.length === 1 && args[0] === "--record-preparation") recordPreparation();
    else
      process.exitCode = await verifyTasks(() =>
        args[0] === "--rust-only" ? rustTasks() : allTestTasks(args),
      );
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
export {
  allTestTasks,
  cpuSlots,
  frontendTask,
  nodeTask,
  prepared,
  pythonTasks,
  root,
  runTasks,
  verifyTasks,
  rustTasks,
};
