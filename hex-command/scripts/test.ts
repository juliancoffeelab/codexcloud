import { readdirSync } from "node:fs";
import { spawnSync } from "node:child_process";

const tests = readdirSync(new URL("../tests/", import.meta.url))
  .filter((file) => file.endsWith(".ts") && file !== "helpers.ts")
  .sort();

// Leave two seconds for npm/tsx startup within the whole-command 20-second limit.
const started = performance.now();
const deadline = started + 18_000;
for (const test of tests) {
  const remaining = Math.floor(deadline - performance.now());
  if (remaining <= 0) {
    console.error("Unit suite exceeded its 20-second total budget.");
    process.exit(1);
  }
  const result = spawnSync(process.execPath, ["--import", "tsx", `tests/${test}`], {
    stdio: "inherit",
    timeout: remaining,
    killSignal: "SIGKILL",
  });
  if (result.error) {
    console.error(
      `Unit check ${test} failed or exceeded the shared time budget:`,
      result.error.message,
    );
  }
  if (result.status !== 0) process.exit(result.status ?? 1);
}
console.log(
  `PASS: ${tests.length} unit files in ${((performance.now() - started) / 1000).toFixed(2)}s`,
);
