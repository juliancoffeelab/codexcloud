import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
const at = process.argv.indexOf("--source");
if (at < 0 || !process.argv[at + 1])
  throw new Error("Pass --source /path/to/self-contained/hex-command");
const source = resolve(process.argv[at + 1]);
const pkg = JSON.parse(readFileSync(resolve(source, "package.json"), "utf8"));
for (const dependencies of [pkg.dependencies, pkg.devDependencies])
  for (const name of Object.keys(dependencies ?? {}))
    assert(
      !["wasm-pack", "typescript", "tsx"].includes(name),
      `retired tool dependency: ${name}`,
    );
// Skills may carry independent tooling; the retired-language check covers the game.
const ignored = new Set([
  ".agents",
  "node_modules",
  "_build",
  "target",
  "dist",
  "tmp",
  ".git",
]);
function walk(folder) {
  for (const entry of readdirSync(folder, { withFileTypes: true })) {
    if (ignored.has(entry.name)) continue;
    const path = resolve(folder, entry.name);
    if (entry.isDirectory()) walk(path);
    else assert(!path.endsWith(".ts"), `authored TypeScript remains: ${path}`);
  }
}
walk(source);
for (const task of ["verify:prepare", "verify"]) {
  const result = spawnSync("npm", ["run", task], {
    cwd: source,
    env: process.env,
    stdio: "inherit",
  });
  assert.equal(result.status, 0, `Hex Command ${task} failed`);
}
// Native-only Arrow tooling has conditional upstream dependencies for other
// targets in Cargo.lock. Check what the browser actually builds and links.
const graph = spawnSync(
  "cargo",
  [
    "tree",
    "--locked",
    "--manifest-path",
    "rust/Cargo.toml",
    "--target",
    "wasm32-unknown-unknown",
    "--features",
    "wasm",
    "--prefix",
    "none",
  ],
  { cwd: source, encoding: "utf8" },
);
assert.equal(graph.status, 0, graph.stderr);
assert(
  !/^(?:wasm-bindgen(?:-[\w-]*)?|tsify(?:-[\w-]*)?|wasm-pack) v/m.test(
    graph.stdout,
  ),
  "retired dependency in browser build graph",
);
console.log("Hex Command real Rust/OCaml site and full verification passed");
