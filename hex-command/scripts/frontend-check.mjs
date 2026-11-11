import { spawnSync } from "node:child_process";
import { readFileSync, mkdirSync, rmSync } from "node:fs";
import { resolve } from "node:path";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
const args = process.argv.slice(2);
const prepare = !args.includes("--prepared");
const groups = args.filter((arg) => arg !== "--prepared");
if (groups.some((g) => !["ui", "wasm", "types"].includes(g)))
  throw new Error("Choose ui, wasm, or types");
const selected = new Set(groups.length ? groups : ["ui", "wasm", "types"]);
const root = resolve(import.meta.dirname, ".."),
  out = resolve(root, "tmp/check-bindings");
const env = toolchainEnvironment();
try {
  if (prepare) {
    const target = selected.has("ui") || selected.has("types") ? "frontend" : "wasm";
    const result = spawnSync("make", [target], { cwd: root, env, stdio: "inherit" });
    if (result.error || result.status !== 0)
      throw result.error ?? new Error(`${target} preparation failed`);
  }
  if (selected.has("types")) {
    mkdirSync(out, { recursive: true });
    const result = spawnSync(
      "ocamljs_pack",
      [
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
  }
  if (selected.has("wasm")) {
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
  }
  if (selected.has("types")) {
    const migrationContract = spawnSync(
      process.execPath,
      ["scripts/check-migration-contract.mjs"],
      {
        cwd: root,
        env,
        stdio: "inherit",
      },
    );
    if (migrationContract.status !== 0) throw new Error("Migration contract verification failed");
    const contracts = spawnSync(process.execPath, ["scripts/check-native-clients.mjs"], {
      cwd: root,
      env,
      stdio: "inherit",
    });
    if (contracts.status !== 0) throw new Error("Native client interface verification failed");
  }
  console.log(`PASS: ${[...selected].join(", ")}`);
} finally {
  if (selected.has("types")) rmSync(out, { recursive: true, force: true });
}
