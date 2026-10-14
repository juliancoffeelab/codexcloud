import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const checks = {
  format: "format",
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
const requested = process.argv.slice(2);
if (requested.some((name) => !Object.hasOwn(checks, name))) {
  console.error(`Choose checks from: ${Object.keys(checks).join(", ")}`);
  process.exit(1);
}
const defaults = Object.keys(checks).filter((name) => name !== "rust");
const unique = [...new Set(requested.length ? requested : defaults)] as Check[];
// The shared test runner already includes Rust; retain the focused Rust group.
const selected = unique.filter((name) => name !== "rust" || !unique.includes("tests"));
// Build bindings once before dependent checks; suppress their duplicate pre-hooks.
const firstDependent = selected.findIndex((name) => name === "ui" || name === "types");
const bindings = selected.indexOf("wasm");
if (firstDependent >= 0 && (bindings < 0 || bindings > firstDependent)) {
  if (bindings >= 0) selected.splice(bindings, 1);
  selected.splice(firstDependent, 0, "wasm");
}
const failed: Check[] = [];
const started = performance.now();
for (const check of selected) {
  const stepStarted = performance.now();
  console.log(`RUN: ${check} (npm run ${checks[check]})`);
  // Independent checks continue after failures. Run tests without competing build/type jobs.
  const result = spawnSync(
    "npm",
    [
      "run",
      "--silent",
      ...(check === "ui" || check === "types" ? ["--ignore-scripts"] : []),
      checks[check],
    ],
    {
      cwd: fileURLToPath(new URL("../", import.meta.url)),
      encoding: "utf8",
      maxBuffer: 16 * 1024 * 1024,
    },
  );
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
