import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";

/** Include native code and authored rule exporters so stale binaries cannot run new schedules. */
export function rustFingerprint(): string {
  const hash = createHash("sha256");
  for (const directory of ["src/game", "rust/src"]) {
    const root = new URL(`../${directory}/`, import.meta.url);
    for (const file of readdirSync(root, { recursive: true })
      .filter((file): file is string => typeof file === "string" && /\.(ts|rs)$/.test(file))
      .sort()) {
      hash.update(`${directory}/${file}\0`);
      hash.update(readFileSync(new URL(file, root)));
    }
  }
  for (const file of [
    "rust/Cargo.toml",
    "rust/Cargo.lock",
    "scripts/export-ai-rules.ts",
    "scripts/ai-fingerprint.ts",
  ]) {
    hash.update(file + "\0");
    hash.update(readFileSync(new URL(`../${file}`, import.meta.url)));
  }
  return hash.digest("hex");
}
