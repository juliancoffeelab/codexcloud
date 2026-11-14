import { wasmPath } from "./frontend-paths.mjs";
// The frozen Rust wire metadata and OCaml signature are independent of the live engine.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { checkMigrationBoundary } from "./check-migrations.mjs";
checkMigrationBoundary(resolve("packages/save-migrations"));
const frozen = JSON.parse(readFileSync("packages/save-migrations/ocaml/native-v48.json", "utf8"));
const live = JSON.parse(readFileSync(wasmPath("schema.json"), "utf8"));
for (const [name, definition] of Object.entries(frozen)) {
  assert.deepStrictEqual(
    live[name],
    definition,
    `Live/frozen wire contract differs: ${name}; add a migration`,
  );
}
const bridgeInputs = [
  "packages/save-migrations/ocaml/native-v48.json",
  "packages/save-migrations/src/formats/readers.json",
  "scripts/generate-migration-native-contract.py",
  "scripts/historical_shapes.py",
];
const digest = createHash("sha256");
for (const path of bridgeInputs.sort()) {
  digest.update(path + "\0");
  digest.update(readFileSync(path));
  digest.update("\0");
}
const bridge = readFileSync("frontend/persistence/Migration_native_contract.ml", "utf8");
assert.equal(
  bridge.match(/Sources SHA256: ([0-9a-f]{64})/)?.[1],
  digest.digest("hex"),
  "Stale typed native migration contract; run migrations:generate",
);
console.log(
  "PASS: complete frozen/live wire metadata and typed native bridge; OCaml contracts checked by Dune",
);
