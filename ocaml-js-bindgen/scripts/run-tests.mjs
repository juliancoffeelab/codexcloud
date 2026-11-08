import { spawn } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";
import { pathToFileURL } from "node:url";
import {
  root,
  rustEnv,
  preparation,
  sourceFingerprint,
} from "./test-inputs.mjs";

// One advisory clock covers the complete verification command, including builds.
export async function runChecks(
  checks,
  { budgetMs = 20_000, started = performance.now() } = {},
) {
  let active;
  let abort;
  const kill = () => {
    if (!active?.pid) return;
    try {
      if (process.platform === "win32") active.kill("SIGKILL");
      else process.kill(-active.pid, "SIGKILL");
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
    }
  };
  const stop = (reason) => {
    abort = reason;
    kill();
  };
  const interrupt = () => stop(new Error("test run interrupted"));
  const timer = setTimeout(
    () =>
      console.warn(
        `WARN: verification exceeded the advisory ${budgetMs / 1000}s budget; continuing.`,
      ),
    Math.max(0, budgetMs - (performance.now() - started)),
  );
  process.once("SIGINT", interrupt);
  process.once("SIGTERM", interrupt);
  try {
    for (const [name, command, args] of checks) {
      if (abort) throw abort;
      console.log(`Checking ${name}`);
      const status = await new Promise((resolve, reject) => {
        active = spawn(command, args, {
          cwd: root,
          env: rustEnv,
          stdio: "inherit",
          detached: process.platform !== "win32",
        });
        active.once("error", reject);
        active.once("exit", (code, signal) => resolve({ code, signal }));
      });
      // Also remove stray background descendants of an otherwise finished check.
      kill();
      active = undefined;
      if (abort) throw abort;
      if (status.code !== 0)
        throw new Error(`${name} failed (${status.signal ?? status.code})`);
    }
    console.log(
      `Passed ${checks.length} checks in ${((performance.now() - started) / 1000).toFixed(2)}s (shared ${budgetMs / 1000}s advisory budget)`,
    );
  } finally {
    clearTimeout(timer);
    process.removeListener("SIGINT", interrupt);
    process.removeListener("SIGTERM", interrupt);
    kill();
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const started = performance.now();
  try {
    let prepared;
    try {
      prepared = JSON.parse(readFileSync(preparation, "utf8"));
    } catch {
      /* reported below */
    }
    const needsPreparation =
      prepared?.fingerprint !== sourceFingerprint() ||
      !Array.isArray(prepared?.artifacts) ||
      prepared.artifacts.some((path) => !existsSync(path));
    const checks = [
      [
        "Rust workspace",
        "cargo",
        ["test", "--locked", "--offline", "--workspace"],
      ],
      ...[
        "projects",
        "runner",
        "packaging",
        "codegen",
        "collections",
        "projection",
        "json",
        "lowering",
        "references",
        "compiled",
        "async",
        "worker",
      ].map((name) => [
        name,
        process.execPath,
        [name === "compiled" ? "scripts/test.mjs" : `scripts/test-${name}.mjs`],
      ]),
    ];
    if (needsPreparation)
      checks.unshift([
        "Build changed or missing verification inputs",
        "make",
        ["prepare"],
      ]);
    await runChecks(checks, { started });
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
