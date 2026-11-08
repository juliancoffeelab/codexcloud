// Input hashes only; the Makefiles own the build commands.
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  root,
  preparation,
  sourceFingerprint,
  verificationArtifacts,
} from "./test-inputs.mjs";
const started = resolve(root, "tmp/test-preparation-start.json");
const fingerprint = sourceFingerprint();
if (process.argv[2] === "begin") {
  mkdirSync(resolve(root, "tmp"), { recursive: true });
  rmSync(preparation, { force: true });
  writeFileSync(started, JSON.stringify({ fingerprint }) + "\n");
} else if (process.argv[2] === "finish") {
  if (JSON.parse(readFileSync(started, "utf8")).fingerprint !== fingerprint)
    throw new Error(
      "Sources changed during preparation; run make prepare again.",
    );
  writeFileSync(
    preparation,
    JSON.stringify({ fingerprint, artifacts: verificationArtifacts() }) + "\n",
  );
  rmSync(started);
} else {
  throw new Error(
    "Use preparation.mjs begin|finish (normally through make prepare)",
  );
}
