import { spawn, type ChildProcess } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "../.agents/skills/rust-toolchain/scripts/toolchain-env";

// Keep the duration target visible without interrupting slow runs.
const budget = 20_000;
const started = performance.now();
const cwd = fileURLToPath(new URL("../", import.meta.url));
const pythonSuites: [name: string, path: string, pattern?: string][] = [
  ["Pixel artwork", "scripts", "test_pixel_*.py"],
  ["Pixel generation", ".agents/skills/pinkbox-generation/tests"],
  ["Pixel cleaning", ".agents/skills/pixel-cleaning/tests"],
  ["Combat analysis", ".agents/skills/combat-simulation/tests"],
  ["Exact gameplay comparison", "scripts", "test_simulation_reports.py"],
];
const suites = [
  {
    name: "Rust",
    command: "cargo",
    args: [
      "test",
      "--locked",
      "--manifest-path",
      "rust/Cargo.toml",
      "--workspace",
      "--features",
      "wasm",
    ],
  },
  ...pythonSuites.map(([name, path, pattern]) => ({
    name,
    command: "python3",
    args: ["-m", "unittest", "discover", "-s", path, ...(pattern ? ["-p", pattern] : [])],
  })),
  {
    name: "Vitest",
    command: process.execPath,
    args: [
      fileURLToPath(new URL("../node_modules/vitest/vitest.mjs", import.meta.url)),
      "run",
      ...process.argv.slice(2),
    ],
  },
];
const active = new Set<ChildProcess>();
const processGroups = process.platform !== "win32";
function stopChildren() {
  for (const child of active) {
    if (processGroups && child.pid) {
      try {
        process.kill(-child.pid, "SIGKILL");
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error;
      }
    } else child.kill("SIGKILL");
  }
}
const warning = setTimeout(() => {
  console.warn("WARNING: test suites exceeded the 20-second target; continuing to completion.");
}, budget);
for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.once(signal, () => {
    stopChildren();
    process.exit(signal === "SIGINT" ? 130 : 143);
  });
}

async function run(suite: (typeof suites)[number]): Promise<boolean> {
  console.log(`RUN: ${suite.name} tests`);
  const suiteStarted = performance.now();
  const child = spawn(suite.command, suite.args, {
    cwd,
    env: toolchainEnvironment(),
    stdio: "inherit",
    detached: processGroups,
  });
  active.add(child);
  return new Promise((resolve) => {
    let settled = false;
    const finish = (passed: boolean) => {
      if (settled) return;
      settled = true;
      active.delete(child);
      console.log(
        `${passed ? "PASS" : "FAIL"}: ${suite.name} in ${((performance.now() - suiteStarted) / 1000).toFixed(2)}s`,
      );
      resolve(passed);
    };
    child.once("error", (error) => {
      console.error(error.message);
      finish(false);
    });
    child.once("close", (code) => finish(code === 0));
  });
}

// Rust builds native targets; Python and Vitest use independent inputs and fixtures.
const results = await Promise.all([
  run(suites[0]!),
  (async () => {
    let passed = true;
    for (const suite of suites.slice(1)) {
      if (!(await run(suite))) passed = false;
    }
    return passed;
  })(),
]);
clearTimeout(warning);
const elapsed = performance.now() - started;
const failed = results.some((passed) => !passed);
console.log(`${failed ? "FAIL" : "PASS"}: all test suites in ${(elapsed / 1000).toFixed(2)}s`);
process.exit(failed ? 1 : 0);
