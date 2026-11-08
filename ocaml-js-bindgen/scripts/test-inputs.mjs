import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve, relative } from "node:path";

export const root = resolve(import.meta.dirname, "..");
export const preparation = resolve(root, "tmp/test-preparation.json");
// Invalid-input conformance runs in the explicitly checked build.
export const rustEnv = {
  ...process.env,
  OCAMLJS_PARANOID: "1",
  OCAMLJS_RESOURCE_DIAGNOSTICS: "1",
};
rustEnv.DUNE_CACHE ??= "enabled";
for (const [name, path] of [
  ["CARGO_HOME", "/workspace/.cargo"],
  ["RUSTUP_HOME", "/workspace/.rustup"],
  ["XDG_CACHE_HOME", "/workspace/.cache"],
])
  if (existsSync(path)) rustEnv[name] ??= path;
if (rustEnv.CARGO_HOME)
  rustEnv.PATH = `${rustEnv.CARGO_HOME}/bin:${rustEnv.PATH}`;
if (
  existsSync("/workspace/.local/bin/dune") &&
  !rustEnv.PATH.split(":").some((directory) =>
    existsSync(resolve(directory, "dune")),
  )
)
  rustEnv.PATH = `${rustEnv.PATH}:/workspace/.local/bin`;

export function sourceFingerprint(base = root) {
  const hash = createHash("sha256");
  function visit(path) {
    for (const e of readdirSync(path, { withFileTypes: true }).sort((a, b) =>
      a.name.localeCompare(b.name),
    )) {
      if (
        ["generated", "dist", "target", "_build", "node_modules"].includes(
          e.name,
        ) ||
        (e.name === "lib" && existsSync(resolve(path, "rescript.json")))
      )
        continue;
      const file = resolve(path, e.name);
      if (e.isDirectory()) visit(file);
      else if (
        /\.(rs|mjs|ml|mli|res|resi|toml|json|html|css|ndjson|jsonl|txt|png|pkg|dune|mk|in)$/.test(
          e.name,
        ) ||
        e.name === "Cargo.lock" ||
        e.name === "dune" ||
        e.name === "dune-project" ||
        e.name === "dune-workspace" ||
        e.name === "Makefile"
      ) {
        hash.update(relative(base, file));
        hash.update("\0");
        hash.update(readFileSync(file));
        hash.update("\0");
      }
    }
  }
  for (const name of [
    "Makefile",
    "examples/rust-to-ocaml/Makefile",
    "examples/rust-to-ocaml/dune",
    "examples/rust-to-ocaml/dune-project",
    "examples/rust-to-ocaml/dune-workspace",
    "examples/rust-to-ocaml/Frontend.ml",
    "examples/rust-to-ocaml/index.html",
    "examples/rust-to-ocaml/test.mjs",
    "examples/rust-to-ocaml/package.json",
    "examples/rust-to-ocaml/package-lock.json",
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "dune-project",
    "dune-workspace",
    "dune",
  ]) {
    if (!existsSync(resolve(base, name))) continue;
    hash.update(name);
    hash.update(readFileSync(resolve(base, name)));
  }
  for (const name of ["crates", "tests", "demos", "scripts"])
    visit(resolve(base, name));
  if (existsSync(resolve(base, "dune.lock"))) visit(resolve(base, "dune.lock"));
  // Hash the standalone example's sources and compiler lock, excluding generated bindings.
  visit(resolve(base, "examples/rust-to-ocaml/rust"));
  visit(resolve(base, "examples/rust-to-ocaml/dune.lock"));
  return hash.digest("hex");
}

// Record outputs once after preparation; normal verification only checks that
// the listed artifacts still exist, without walking compiler caches.
export function verificationArtifacts() {
  const files = [resolve(root, "target/debug/ocamljs_pack")];
  function walk(folder) {
    if (!existsSync(folder)) return;
    for (const entry of readdirSync(folder, { withFileTypes: true })) {
      const path = resolve(folder, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (entry.isFile() && entry.name !== "Test.cjs") files.push(path);
    }
  }
  for (const entry of readdirSync(resolve(root, "demos"), {
    withFileTypes: true,
  })) {
    if (!entry.isDirectory()) continue;
    const project = resolve(root, "demos", entry.name);
    for (const directory of [
      "generated",
      "lib/es6",
      "_build/default/test-output",
    ])
      walk(resolve(project, directory));
    for (const path of [
      "_build/default/adapters/js_of_ocaml/Test.bc.js",
      "_build/default/tests/lwt/Test.bc.js",
    ])
      if (existsSync(resolve(project, path)))
        files.push(resolve(project, path));
    const manifest = resolve(root, "tmp", `${entry.name}-tests.jsonl`);
    files.push(manifest);
    for (const line of readFileSync(manifest, "utf8")
      .trim()
      .split("\n")
      .filter(Boolean)) {
      const artifact = JSON.parse(line);
      if (
        artifact.reason === "compiler-artifact" &&
        artifact.profile?.test &&
        artifact.executable
      )
        files.push(artifact.executable);
    }
  }
  return [...new Set(files)];
}
