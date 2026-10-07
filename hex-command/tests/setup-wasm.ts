import { readFileSync } from "node:fs";
import { initSync } from "../rust/pkg/hex_command";

// Node has no browser URL loader. Initialize the same WASM before importing
// application metadata; browser startup uses wasm-bindgen's async loader.
initSync({
  module: new Uint8Array(readFileSync(new URL("../rust/pkg/hex_command_bg.wasm", import.meta.url))),
});
