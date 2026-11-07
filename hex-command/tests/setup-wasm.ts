import { readFileSync } from "node:fs";
import { initSync } from "../rust/pkg/hex_command";

// Node has no browser URL loader. Initialize the same WASM before importing
// application metadata; browser startup uses wasm-bindgen's async loader.
initSync({
  module: new Uint8Array(readFileSync(new URL("../rust/pkg/hex_command_bg.wasm", import.meta.url))),
});

// Tests are another explicit boot owner; metadata imports cannot start an engine.
const { startEngine } = await import("../src/game/engine-startup");
const { loadNativeEngineFactory } = await import("../src/game/native-engine");
await startEngine(async () => {
  const result = await loadNativeEngineFactory(() => import("../rust/pkg/hex_command"));
  if (!result.ok) throw new Error(JSON.stringify(result.error));
  return result.value;
});

// A fresh database namespace per case; persistence tests use the real asynchronous API.
import { IDBFactory } from "fake-indexeddb";
import { beforeEach, vi } from "vitest";
import { initializeStorage } from "../src/save/storage";
beforeEach(async () => {
  vi.stubGlobal("indexedDB", new IDBFactory());
  await initializeStorage({ turns: [], chooseNewGame: false });
});
