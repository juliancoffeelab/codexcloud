import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

// Leave two seconds for npm/tsx startup within the suite's 20-second budget.
// Vitest owns discovery, filters, execution and failure reporting.
const result = spawnSync(
  process.execPath,
  [
    fileURLToPath(new URL("../node_modules/vitest/vitest.mjs", import.meta.url)),
    "run",
    ...process.argv.slice(2),
  ],
  {
    cwd: fileURLToPath(new URL("../", import.meta.url)),
    stdio: "inherit",
    timeout: 18_000,
    killSignal: "SIGKILL",
  },
);
if (result.error) {
  console.error(result.error.message);
  if ("code" in result.error && result.error.code === "ETIMEDOUT") {
    console.error("Test suite exceeded its shared 20-second budget.");
  }
}
process.exit(result.status ?? 1);
