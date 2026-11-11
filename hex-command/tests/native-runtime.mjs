import { readFileSync } from "node:fs";
import { initialize } from "@wasm/bindings.runtime.mjs";
await initialize(readFileSync(process.env.HEX_COMMAND_WASM_PATH));
