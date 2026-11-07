import { cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { checkMigrationBoundary } from "./check-migrations.mjs";

// Opt-in: application sources and workspace links are absent from this directory.
const temporary = mkdtempSync(resolve(tmpdir(), "hex-migrations-"));
try {
  const source = fileURLToPath(new URL("../packages/save-migrations", import.meta.url));
  const isolated = resolve(temporary, "save-migrations");
  checkMigrationBoundary(source);
  cpSync(source, isolated, { recursive: true });
  checkMigrationBoundary(isolated);
  const result = spawnSync(
    process.execPath,
    [
      fileURLToPath(new URL("../node_modules/typescript/bin/tsc", import.meta.url)),
      "-p",
      resolve(isolated, "tsconfig.json"),
      "--noEmit",
      "false",
      "--outDir",
      resolve(temporary, "build"),
    ],
    { cwd: isolated, stdio: "inherit" },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Isolated migration build failed (${result.status})`);
  console.log("Isolated migration package build passed");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
