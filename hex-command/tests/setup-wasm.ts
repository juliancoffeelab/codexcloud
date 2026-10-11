import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { initSync } from "../rust/pkg/hex_command";

// Node has no browser URL loader. Initialize the same WASM before importing
// application metadata; browser startup uses wasm-bindgen's async loader.
initSync({
  module: new Uint8Array(readFileSync(resolve(process.cwd(), "rust/pkg/hex_command_bg.wasm"))),
});
