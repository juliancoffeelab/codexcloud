import { spawnSync } from "node:child_process";
import { readFileSync, mkdirSync, rmSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
const groups = process.argv.slice(2);
if (groups.some((g) => !["ui", "wasm", "types"].includes(g)))
  throw new Error("Choose ui, wasm, or types");
const root = resolve(import.meta.dirname, ".."),
  out = resolve(root, "tmp/check-bindings");
const env = toolchainEnvironment();
try {
  mkdirSync(out, { recursive: true });
  const result = spawnSync(
    process.execPath,
    [
      fileURLToPath(new URL("./toolchain/packager.mjs", import.meta.url)),
      "generate",
      "--wasm",
      "rust/pkg/backend.wasm",
      "--schema",
      "rust/pkg/schema.json",
      "--name",
      "hex_command",
      "--out-dir",
      out,
    ],
    { cwd: root, env, stdio: "inherit" },
  );
  if (result.error || result.status !== 0)
    throw result.error ?? new Error("Native binding verification failed");
  for (const ext of ["ml", "mli"])
    if (
      readFileSync(`${out}/Bindings.${ext}`, "utf8") !==
      readFileSync(`frontend/engine/Domain.${ext}`, "utf8")
    )
      throw new Error("Stale OCaml bindings: run npm run verify:prepare");
  const module = await WebAssembly.compile(readFileSync("rust/pkg/backend.wasm"));
  if (
    WebAssembly.Module.imports(module).some(
      (i) =>
        i.module !== "ocamljs" ||
        ![
          "boundary_error",
          "task_wake",
          "callback_release",
          "callback_call",
          "callback_retain",
          "fetch_open",
          "fetch_close",
          "fetch_credit",
        ].includes(i.name),
    )
  )
    throw new Error("Unexpected foreign ABI import");
  const compiled = spawnSync("dune", ["build", "--root", root, "@melange"], {
    cwd: root,
    env,
    stdio: "inherit",
  });
  if (compiled.status !== 0) throw new Error("OCaml interface verification failed");
  const migrationContract = spawnSync(process.execPath, ["scripts/check-migration-contract.mjs"], {
    cwd: root,
    env,
    stdio: "inherit",
  });
  if (migrationContract.status !== 0) throw new Error("Migration contract verification failed");
  const contracts = spawnSync(process.execPath, ["scripts/check-native-clients.mjs"], {
    cwd: root,
    env,
    stdio: "inherit",
  });
  if (contracts.status !== 0) throw new Error("Native client interface verification failed");
  console.log("PASS: native Rust schema, OCaml implementation/interfaces, and WASM imports");
} finally {
  rmSync(out, { recursive: true, force: true });
}
