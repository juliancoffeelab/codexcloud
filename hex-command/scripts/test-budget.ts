import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "../.agents/skills/rust-toolchain/scripts/toolchain-env";

// Leave two seconds for npm/tsx startup within the combined 20-second budget.
const budget = 18_000;
const started = performance.now();
const cwd = fileURLToPath(new URL("../", import.meta.url));
const pythonSuites: [name: string, path: string, pattern?: string][] = [
  ["Pixel artwork", "scripts", "test_pixel_*.py"],
  ["Pixel generation", ".agents/skills/pinkbox-generation/tests"],
  ["Pixel cleaning", ".agents/skills/pixel-cleaning/tests"],
  ["Combat analysis", ".agents/skills/combat-simulation/tests"],
];
const suites = [
  {
    name: "Rust",
    command: "cargo",
    args: ["test", "--locked", "--manifest-path", "rust/Cargo.toml", "--features", "wasm"],
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
let failed = false;
for (const suite of suites) {
  const remaining = budget - (performance.now() - started);
  if (remaining <= 0) {
    console.error("All test suites exceeded their shared 20-second budget.");
    process.exit(1);
  }
  console.log(`RUN: ${suite.name} tests`);
  const result = spawnSync(suite.command, suite.args, {
    cwd,
    env: toolchainEnvironment(),
    stdio: "inherit",
    timeout: Math.ceil(remaining),
    killSignal: "SIGKILL",
  });
  if (result.status !== 0 || result.error) {
    failed = true;
    if (result.error) console.error(result.error.message);
    if (result.error && "code" in result.error && result.error.code === "ETIMEDOUT") {
      console.error("All test suites exceeded their shared 20-second budget.");
      process.exit(1);
    }
  }
}
const elapsed = performance.now() - started;
if (elapsed > budget) {
  console.error("All test suites exceeded their shared 20-second budget.");
  process.exit(1);
}
console.log(`${failed ? "FAIL" : "PASS"}: all test suites in ${(elapsed / 1000).toFixed(2)}s`);
process.exit(failed ? 1 : 0);
