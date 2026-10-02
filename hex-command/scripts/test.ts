import { readdirSync } from "node:fs";
import { spawnSync } from "node:child_process";

const tests = readdirSync(new URL("../tests/", import.meta.url))
  .filter((file) => file.endsWith(".ts") && file !== "helpers.ts")
  .sort();
for (const test of tests) {
  const result = spawnSync(
    process.execPath,
    ["--import", "tsx", `tests/${test}`],
    { stdio: "inherit" },
  );
  if (result.status !== 0) process.exit(result.status ?? 1);
}
