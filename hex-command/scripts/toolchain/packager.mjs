import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "./toolchain-env.mjs";

// Cargo checks source freshness; an installed CLI from another revision is never used.
// Keep generation optimized, matching the former cargo-installed CLI.
const result = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--locked",
    "--release",
    "--manifest-path",
    fileURLToPath(new URL("../../../ocaml-js-bindgen/Cargo.toml", import.meta.url)),
    "--package",
    "ocamljs_pack",
    "--target-dir",
    fileURLToPath(new URL("../../../ocaml-js-bindgen/target/", import.meta.url)),
    "--",
    ...process.argv.slice(2),
  ],
  { env: toolchainEnvironment(), stdio: "inherit" },
);
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
