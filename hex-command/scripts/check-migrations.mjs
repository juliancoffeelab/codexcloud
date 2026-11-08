import { readFileSync, readdirSync, existsSync, realpathSync, statSync } from "node:fs";
import { relative, resolve, isAbsolute } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";

const project = fileURLToPath(new URL("../", import.meta.url));

function checkImports(sources) {
  const checker = resolve(project, "_build/default/scripts/migration_import_check.exe");
  const inputs = ["scripts/migration_import_check.ml", "scripts/dune", "dune-project"];
  const env = toolchainEnvironment();
  if (
    !existsSync(checker) ||
    inputs.some((file) => statSync(resolve(project, file)).mtimeMs > statSync(checker).mtimeMs)
  ) {
    const build = spawnSync("dune", ["build", "scripts/migration_import_check.exe"], {
      cwd: project,
      env,
      encoding: "utf8",
    });
    if (build.error || build.status !== 0)
      throw build.error ?? new Error(build.stderr || "Cannot build the OCaml import checker");
  }
  const args = [...sources];
  if (sources.some((file) => /\.rei?$/.test(file))) {
    const refmt = spawnSync("dune", ["exec", "--", "which", "refmt"], {
      cwd: project,
      env,
      encoding: "utf8",
    });
    if (refmt.error || refmt.status !== 0)
      throw refmt.error ?? new Error(refmt.stderr || "Cannot locate the Reason parser");
    args.unshift("--refmt", refmt.stdout.trim());
  }
  const checked = spawnSync(checker, args, { cwd: project, env, encoding: "utf8" });
  if (checked.error) throw checked.error;
  return checked;
}

/** Every source in the frozen library is checked, not just the application's imports. */
export function checkMigrationBoundary(directory) {
  const root = realpathSync(directory),
    failures = [],
    ocamlSources = [];
  const inside = (path) => {
    const rel = relative(root, path);
    return rel !== ".." && !rel.startsWith("../") && !isAbsolute(rel);
  };
  const fail = (file, message) => failures.push(`${relative(root, file)}: ${message}`);
  function scan(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const file = resolve(directory, entry.name);
      if (entry.isSymbolicLink()) {
        fail(file, "symlinks are not allowed");
        continue;
      }
      if (entry.isDirectory()) {
        scan(file);
        continue;
      }
      if (/\.(?:[cm]?ts|tsx|[cm]?js|jsx)$/.test(file)) fail(file, "migration runtime is OCaml");
      if (/\.(?:mli?|rei?)$/.test(file)) ocamlSources.push(file);
    }
  }
  scan(root);
  const imports = checkImports(ocamlSources);
  if (imports.status !== 0) failures.push(imports.stderr.trim() || "OCaml import audit failed");
  const ocaml = resolve(root, "ocaml");
  const dune = readFileSync(resolve(ocaml, "dune"), "utf8");
  if (/\(libraries\b/.test(dune))
    fail(resolve(ocaml, "dune"), "frozen library must not depend on application libraries");
  const contractsFile = resolve(ocaml, "contracts.json");
  const contracts = JSON.parse(readFileSync(contractsFile, "utf8"));
  for (const [name, expected] of Object.entries({ ...contracts.inputs, ...contracts.outputs })) {
    const path = resolve(root, name);
    if (
      !inside(path) ||
      !existsSync(path) ||
      createHash("sha256").update(readFileSync(path)).digest("hex") !== expected
    )
      fail(contractsFile, `stale frozen OCaml contract ${name}; run migrations:generate`);
  }
  const doc = JSON.parse(readFileSync(resolve(root, "src/formats/readers.json"), "utf8"));
  for (const [name, index] of Object.entries(doc.entries))
    if (!doc.shapes[index]) fail(contractsFile, `missing historical reader ${name}`);
  if (failures.length) throw new Error(`Migration isolation failed:\n${failures.join("\n")}`);
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  checkMigrationBoundary(fileURLToPath(new URL("../packages/save-migrations", import.meta.url)));
  console.log("Migration boundary passed");
}
