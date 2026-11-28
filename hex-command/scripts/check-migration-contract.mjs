import { wasmPath } from "./frontend-paths.mjs";
// The frozen Rust wire metadata and OCaml signature are independent of the live engine.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { checkMigrationBoundary } from "./check-migrations.mjs";
checkMigrationBoundary(resolve("packages/save-migrations"));
const frozen = JSON.parse(readFileSync("packages/save-migrations/ocaml/native-v53.json", "utf8"));
const live = JSON.parse(readFileSync(wasmPath("schema.json"), "utf8"));
for (const [alias, primitive] of [
  ["Text", "String"],
  ["TextSlice", "str"],
]) {
  assert.equal(live[alias]?.kind, "alias", `${alias} must remain a transparent string alias`);
  assert.equal(live[alias]?.ty?.name, primitive, `${alias} must preserve the Rust wire type`);
  assert.equal(live[alias]?.js_value, true, `${alias} must use host text`);
  assert.equal(live[alias]?.js_selector, "string", `${alias} must use host text`);
}
// Text is a Rust String alias with a different JS transport, not a wire change.
function wireDefinition(value) {
  if (Array.isArray(value)) return value.map(wireDefinition);
  if (!value || typeof value !== "object") return value;
  const result = Object.fromEntries(
    Object.entries(value).map(([key, child]) => [key, wireDefinition(child)]),
  );
  if (result.kind === "named" && result.name === "Text") result.name = "String";
  if (result.kind === "named" && result.name === "TextSlice") result.name = "str";
  return result;
}
for (const [name, definition] of Object.entries(frozen)) {
  assert.deepStrictEqual(
    wireDefinition(live[name]),
    wireDefinition(definition),
    `Live/frozen wire contract differs: ${name}; add a migration`,
  );
}
const bridgeInputs = [
  "packages/save-migrations/ocaml/native-v53.json",
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
