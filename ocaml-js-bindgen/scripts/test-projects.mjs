import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { root } from "./test-inputs.mjs";

// Preparation compiles each independent Rust project; the bounded test run
// executes its test binaries directly, without rebuilding.
for (const project of readdirSync(resolve(root, "demos"))) {
  const artifacts = readFileSync(
    resolve(root, "tmp", `${project}-tests.jsonl`),
    "utf8",
  )
    .trim()
    .split("\n")
    .filter(Boolean)
    .map(JSON.parse)
    .filter(
      (entry) =>
        entry.reason === "compiler-artifact" &&
        entry.profile.test &&
        entry.executable,
    );
  assert(artifacts.length, `${project}: no prepared Rust tests`);
  for (const artifact of artifacts) {
    const result = spawnSync(artifact.executable, [], {
      cwd: resolve(root, "demos", project),
      stdio: "inherit",
    });
    assert.equal(
      result.status,
      0,
      `${project}: ${result.error ?? "Rust test failed"}`,
    );
  }
}
