/** Rust defines the contract; Prettier only formats its generated declarations. */
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { format, resolveConfig } from "prettier";

const args = process.argv.slice(2);
if (args.some((arg) => arg !== "--write")) throw new Error("Usage: rust-bindings.ts [--write]");
const cwd = fileURLToPath(new URL("../", import.meta.url));
const destination = join(cwd, "src/game/native-types.ts");
const temporary = mkdtempSync(join(tmpdir(), "hex-command-bindings-"));
try {
  const generated = join(temporary, "native-types.ts");
  execFileSync(
    "cargo",
    [
      "run",
      "--locked",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "--features",
      "bindings",
      "--bin",
      "export-types",
      "--",
      generated,
    ],
    { cwd, stdio: "inherit" },
  );
  const source = await format(readFileSync(generated, "utf8"), {
    ...(await resolveConfig(destination)),
    parser: "typescript",
  });
  if (args.includes("--write")) writeFileSync(destination, source);
  else if (readFileSync(destination, "utf8") !== source) {
    throw new Error("Rust transfer types changed. Run npm run bindings:generate.");
  }
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
