import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { availableParallelism } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  toolchainCommand,
  toolchainEnvironment,
} from "../.agents/skills/rust-toolchain/scripts/toolchain-env";

export const root = fileURLToPath(new URL("../", import.meta.url));
const preparedPath = join(root, "tmp/verification-prepared.json");
// One budget for the invocation, including module loading. Reserve time for npm startup.
const deadline = 18_000;
export const cpuSlots = Math.min(6, availableParallelism());
const frontendWorkers = Math.min(3, cpuSlots);
export type Task = {
  name: string;
  command: string;
  args: string[];
  slots?: number;
};
export function nodeTask(name: string, script: string, args: string[] = [], slots = 1): Task {
  return {
    name,
    command: process.execPath,
    args: [...(script.endsWith(".ts") ? ["--import", "tsx"] : []), script, ...args],
    slots,
  };
}

function sourceFingerprint(): string {
  const files = [
    "rust/Cargo.toml",
    "rust/Cargo.lock",
    "rust/build.rs",
    ".dune-version",
    "scripts/install-dune.sh",
    "dune-project",
    "dune",
    "dune-workspace",
    "scripts/build-pixel-art.py",
    "src/presentation/pixel-art.ts",
    "scripts/melange-bindings.ts",
    "scripts/frontend-boundary.ts",
    "scripts/terrain-prebuild.ts",
    "scripts/frontend-paths.ts",
    "package.json",
    "package-lock.json",
    "index.html",
    "src/style.css",
    "vite.config.ts",
  ];
  function walk(directory: string): void {
    for (const entry of readdirSync(join(root, directory), { withFileTypes: true })) {
      const path = `${directory}/${entry.name}`;
      if (entry.isDirectory()) walk(path);
      else if (entry.isFile()) files.push(path);
    }
  }
  for (const directory of [
    "rust/src",
    "rust/data",
    "frontend",
    "pixel/art",
    "src/save/migration/legacy/data",
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

function artifactFingerprint(binaries: string[]): string {
  const files = [...binaries];
  function walk(directory: string, matches: (path: string) => boolean): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) walk(path, matches);
      else if (entry.isFile() && matches(path)) files.push(path);
    }
  }
  walk(join(root, "_build/default/frontend/output"), (path) => path.endsWith(".js"));
  walk(join(root, "rust/pkg"), (path) => /\.(?:js|wasm|d\.ts)$/.test(path));
  const sprites = join(root, "public/assets/sprites");
  if (!existsSync(sprites) || !readdirSync(sprites).some((name) => name.endsWith(".png")))
    throw new Error("Prepared artwork is missing. Run npm run verify:prepare.");
  walk(sprites, (path) => path.endsWith(".png"));
  if (!existsSync(join(root, "dist/index.html")))
    throw new Error("Prepared production bundle is missing. Run npm run verify:prepare.");
  walk(join(root, "dist"), () => true);
  const hash = createHash("sha256");
  for (const path of [...new Set(files)].sort()) {
    hash.update(path).update("\0").update(readFileSync(path)).update("\0");
  }
  return hash.digest("hex");
}

type Prepared = { fingerprint: string; artifactFingerprint: string; binaries: string[] };
let currentPrepared: Prepared | undefined;
export function prepared(): Prepared {
  if (currentPrepared) return currentPrepared;
  if (!existsSync(preparedPath)) throw new Error("Run npm run verify:prepare before verification.");
  const result = JSON.parse(readFileSync(preparedPath, "utf8")) as Prepared;
  if (typeof result.artifactFingerprint !== "string" || !Array.isArray(result.binaries)) {
    throw new Error("Preparation manifest is outdated. Run npm run verify:prepare.");
  }
  if (result.fingerprint !== sourceFingerprint()) {
    throw new Error(
      "Prepared artifacts are stale. Run npm run verify:prepare after source changes.",
    );
  }
  if (
    !result.binaries.length ||
    result.binaries.some((path) => !existsSync(path)) ||
    !existsSync(join(root, "rust/pkg/hex_command_bg.wasm")) ||
    !existsSync(join(root, "_build/default/frontend/output/frontend/Main.js"))
  ) {
    throw new Error("Prepared artifacts are missing. Run npm run verify:prepare.");
  }
  if (result.artifactFingerprint !== artifactFingerprint(result.binaries)) {
    throw new Error("Prepared artifacts have changed. Run npm run verify:prepare.");
  }
  currentPrepared = result;
  return result;
}

export function pythonTasks(): Task[] {
  return [
    ["python-art", "scripts", "test_pixel_*.py"],
    ["python-generation", ".agents/skills/pinkbox-generation/tests", "test*.py"],
    ["python-cleaning", ".agents/skills/pixel-cleaning/tests", "test*.py"],
  ].map(([name, directory, pattern]) => ({
    name: name!,
    command: "python3",
    args: ["-m", "unittest", "discover", "-s", directory!, "-p", pattern!],
  }));
}
export function rustTasks(): Task[] {
  return [
    ...prepared().binaries.map((command, index) => ({
      name: `rust-${index + 1}`,
      command,
      args: ["--test-threads=1"],
    })),
    // Rustdoc owns discovery and compilation of documentation examples. This is
    // retained even when there are currently no examples, so coverage cannot drift.
    {
      name: "rust-doc",
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
      ],
    },
  ];
}
export function frontendTask(args: string[] = []): Task {
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
export function allTestTasks(args: string[] = []): Task[] {
  return [frontendTask(args), ...rustTasks(), ...pythonTasks()];
}

/** Independent checks share one wall deadline and bounded CPU/process slots.
 * Each child owns a process group; cancellation kills its descendants as well.
 */
export async function runTasks(tasks: Task[]): Promise<number> {
  const queue = tasks.slice();
  const running = new Map<ChildProcess, Task>();
  const failures: string[] = [];
  const env = toolchainEnvironment();
  let occupied = 0;
  let stopped = false;
  function kill(child: ChildProcess): void {
    if (!child.pid) return;
    try {
      if (process.platform === "win32") child.kill("SIGKILL");
      else process.kill(-child.pid, "SIGKILL");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ESRCH") console.error(error);
    }
  }
  function stop(reason: string): void {
    if (stopped) return;
    stopped = true;
    failures.push(reason);
    for (const task of queue.splice(0)) failures.push(`${task.name} (not run)`);
    for (const child of running.keys()) kill(child);
  }
  const interrupt = () => stop("Interrupted");
  process.on("SIGINT", interrupt);
  process.on("SIGTERM", interrupt);
  const timer = setTimeout(
    () => stop("Shared 20-second command budget exceeded"),
    Math.max(0, deadline - performance.now()),
  );
  const result = await new Promise<number>((complete) => {
    function schedule(): void {
      if (performance.now() >= deadline && !stopped)
        stop("Shared 20-second command budget exceeded");
      if (!stopped) {
        for (let index = 0; index < queue.length && running.size < 3;) {
          const task = queue[index]!;
          const weight = Math.min(cpuSlots, task.slots ?? 1);
          if (occupied + weight > cpuSlots) {
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
          let launchError: Error | undefined;
          child.on("error", (error) => {
            launchError = error;
            console.error(error.message);
          });
          child.on("close", (code, signal) => {
            running.delete(child);
            occupied -= weight;
            // A command must not leave workers or helper processes behind.
            kill(child);
            const passed = code === 0 && !signal && !launchError && !stopped;
            console.log(
              `${passed ? "PASS" : "FAIL"}: ${task.name} (${((performance.now() - started) / 1000).toFixed(2)}s)`,
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
  clearTimeout(timer);
  process.off("SIGINT", interrupt);
  process.off("SIGTERM", interrupt);
  if (failures.length) console.error(`FAIL: ${[...new Set(failures)].join(", ")}`);
  console.log(`Finished ${tasks.length} checks in ${(performance.now() / 1000).toFixed(2)}s`);
  return result;
}

/** Cold toolchain/build work is explicit, never hidden in a timed check. */
function prepare(): void {
  rmSync(preparedPath, { force: true });
  const env = toolchainEnvironment();
  const tasks: Task[] = [
    nodeTask("content", "scripts/content.ts", ["--check"]),
    {
      name: "WASM",
      command: toolchainCommand("wasm-pack", root),
      args: [
        "build",
        "rust",
        "--target",
        "web",
        "--release",
        "--out-dir",
        "pkg",
        "--out-name",
        "hex_command",
        "--no-pack",
        "--no-opt",
        "--locked",
        "--features",
        "wasm",
      ],
    },
    { name: "artwork", command: "python3", args: ["scripts/build-pixel-art.py"] },
    nodeTask("frontend bindings", "scripts/melange-bindings.ts"),
    { name: "frontend", command: "dune", args: ["build", "@melange"] },
    nodeTask("production bundle", "node_modules/vite/bin/vite.js", ["build"]),
    { name: "formatters", command: "dune", args: ["exec", "--", "which", "refmt", "ocp-indent"] },
  ];
  for (const task of tasks) {
    console.log(`PREPARE: ${task.name}`);
    const result = spawnSync(task.command, task.args, { cwd: root, env, stdio: "inherit" });
    if (result.status !== 0 || result.error)
      throw result.error ?? new Error(`${task.name} preparation failed`);
  }
  const fingerprint = sourceFingerprint();
  const native = spawnSync(
    "cargo",
    [
      "test",
      "--no-run",
      "--locked",
      "--manifest-path",
      "rust/Cargo.toml",
      "--features",
      "wasm",
      "--message-format=json",
    ],
    { cwd: root, env, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  if (native.stderr) process.stderr.write(native.stderr);
  if (native.status !== 0 || native.error) {
    if (native.stdout) process.stdout.write(native.stdout);
    throw native.error ?? new Error("Native test preparation failed");
  }
  const binaries: string[] = [];
  for (const line of native.stdout.split("\n").filter(Boolean)) {
    const artifact = JSON.parse(line) as {
      reason: string;
      profile?: { test: boolean };
      executable?: string;
    };
    if (artifact.reason === "compiler-artifact" && artifact.profile?.test && artifact.executable)
      binaries.push(artifact.executable);
  }
  if (!binaries.length) throw new Error("Cargo did not publish any test executables");
  if (sourceFingerprint() !== fingerprint)
    throw new Error("Sources changed during preparation. Run it again.");
  const uniqueBinaries = [...new Set(binaries)];
  const artifacts = artifactFingerprint(uniqueBinaries);
  mkdirSync(dirname(preparedPath), { recursive: true });
  writeFileSync(
    preparedPath,
    JSON.stringify({ fingerprint, artifactFingerprint: artifacts, binaries: uniqueBinaries }) +
      "\n",
  );
  console.log(
    "Prepared current WASM/frontend/artwork/production bundle/native tests. Run npm test or npm run verify.",
  );
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length === 1 && args[0] === "--prepare") prepare();
    else
      process.exitCode = await runTasks(
        args[0] === "--rust-only" ? rustTasks() : allTestTasks(args),
      );
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
