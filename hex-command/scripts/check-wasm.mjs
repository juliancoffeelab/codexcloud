import { wasmPath } from "./frontend-paths.mjs";
import { readFileSync } from "node:fs";
const module = await WebAssembly.compile(readFileSync(wasmPath("backend.wasm")));
const allowed = new Set([
  "boundary_error",
  "task_wake",
  "callback_release",
  "callback_call",
  "callback_retain",
  "fetch_open",
  "fetch_close",
  "fetch_credit",
]);
for (const entry of WebAssembly.Module.imports(module))
  if (entry.module !== "ocamljs" || !allowed.has(entry.name))
    throw new Error(`Unexpected foreign ABI import: ${entry.module}.${entry.name}`);
console.log("PASS: WASM imports");
