// Application JSON decoding belongs to consumer code. Old SDK flags must fail.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
const root = resolve(import.meta.dirname, "..");
for (const flag of ["--json-type", "--json-default", "--json-presence", "--equal-type"]) {
  const result = spawnSync(resolve(root, "target/debug/ocamljs_pack"), ["generate", flag, "Removed"], {encoding: "utf8"});
  assert.equal(result.status, 1);
  assert.match(result.stderr, new RegExp(`unknown option ${flag}`));
}
console.log("PASS: application convenience options are rejected");
