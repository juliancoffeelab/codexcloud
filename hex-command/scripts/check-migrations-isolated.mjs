import { cpSync, mkdtempSync, rmSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve, dirname } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
import { checkMigrationBoundary } from "./check-migrations.mjs";

// Opt-in: application sources and workspace links are absent from this directory.
const temporary = mkdtempSync(resolve(tmpdir(), "hex-migrations-"));
try {
  const source = fileURLToPath(new URL("../packages/save-migrations", import.meta.url));
  const isolated = resolve(temporary, "save-migrations");
  checkMigrationBoundary(source);
  cpSync(source, isolated, { recursive: true });
  checkMigrationBoundary(isolated);
  // Reuse the prepared compiler, while compiling only the copied package.
  // A second Dune project would bootstrap another OCaml compiler unnecessarily.
  const project = fileURLToPath(new URL("../", import.meta.url));
  const env = toolchainEnvironment();
  const generated = spawnSync(
    "python3",
    [
      resolve(project, "scripts/generate-migration-data.py"),
      isolated,
      resolve(isolated, "ocaml/frozen_data.ml"),
    ],
    { env, stdio: "inherit" },
  );
  if (generated.error || generated.status !== 0)
    throw generated.error ?? new Error("Frozen migration data generation failed");
  checkMigrationBoundary(isolated);
  const located = spawnSync("dune", ["exec", "--", "which", "melc", "ocamldep"], {
    cwd: project,
    env,
    encoding: "utf8",
  });
  if (located.status !== 0) throw new Error(located.stderr || "Cannot locate migration compiler");
  const [compiler, dependencyCompiler] = located.stdout.trim().split("\n");
  const compilerRoot = resolve(dirname(compiler), "../lib/melange");
  const includes = [
    resolve(compilerRoot, "melange"),
    resolve(compilerRoot, "js/melange"),
    resolve(compilerRoot, "__private__/melange_mini_stdlib/melange/.public_cmi_melange"),
  ];
  const sources = resolve(isolated, "ocaml");
  const files = readdirSync(sources).filter((file) => /\.mli?$/.test(file));
  const ordered = spawnSync(dependencyCompiler, ["-sort", ...files], {
    cwd: sources,
    env,
    encoding: "utf8",
  });
  if (ordered.status !== 0) throw new Error(ordered.stderr);
  for (const file of ordered.stdout.trim().split(/\s+/)) {
    const compiled = spawnSync(
      compiler,
      [
        ...includes.flatMap((path) => ["-I", path]),
        "-I",
        sources,
        "-ppx",
        resolve(compilerRoot, "ppx/ppx.exe") + " --as-ppx",
        "--mel-stop-after-cmj",
        "-c",
        file,
      ],
      { cwd: sources, env, stdio: "inherit" },
    );
    if (compiled.error || compiled.status !== 0)
      throw compiled.error ?? new Error(`Isolated OCaml compilation failed: ${file}`);
  }
  console.log("Isolated OCaml migration package build passed");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
