import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const checks = {
  style: "style",
  content: "content:check",
  wasm: "check:wasm",
  ui: "check:ui",
  types: "check:types",
  rust: "check:rust",
  art: "art:check",
  tests: "test",
  build: "build",
} as const;
type Check = keyof typeof checks;
// Keep the former group name working for existing callers.
const requested = process.argv
  .slice(2)
  .map((name) => (name === "format" ? "style" : name === "bindings" ? "wasm" : name));
if (requested.some((name) => !Object.hasOwn(checks, name))) {
  console.error(`Choose checks from: ${Object.keys(checks).join(", ")}`);
  process.exit(1);
}
const selected = [...new Set(requested.length ? requested : Object.keys(checks))] as Check[];
const failed: Check[] = [];
const started = performance.now();
for (const check of selected) {
  const stepStarted = performance.now();
  console.log(`RUN: ${check} (npm run ${checks[check]})`);
  // Independent checks continue after failures. Run tests without competing build/type jobs.
  const result = spawnSync("npm", ["run", "--silent", checks[check]], {
    cwd: fileURLToPath(new URL("../", import.meta.url)),
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  const seconds = ((performance.now() - stepStarted) / 1000).toFixed(2);
  if (result.status === 0 && !result.error) console.log(`PASS: ${check} (${seconds}s)`);
  else {
    failed.push(check);
    console.error(`FAIL: ${check} (${seconds}s)`);
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    if (result.error) console.error(result.error.message);
  }
}
const seconds = ((performance.now() - started) / 1000).toFixed(2);
if (failed.length) {
  console.error(`FAIL: ${failed.join(", ")} — ${selected.length} checks in ${seconds}s`);
  console.error(`Rerun failed groups: npm run verify -- ${failed.join(" ")}`);
  process.exit(1);
}
console.log(`PASS: all ${selected.length} checks in ${seconds}s`);
