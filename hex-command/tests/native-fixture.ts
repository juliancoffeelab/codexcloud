import { readFileSync } from "node:fs";
import { afterEach, afterAll } from "vitest";
import { initSync, NativeEngineFactory } from "../rust/pkg/hex_command";
import {
  createNativeEngineFactory,
  type GameEngine,
  type GameEngineFactory,
} from "../src/game/native-engine";

initSync({
  module: new Uint8Array(readFileSync(new URL("../rust/pkg/hex_command_bg.wasm", import.meta.url))),
});
let shared: GameEngineFactory | undefined;
const handles = new Set<GameEngine>();
/** Share immutable rules within a test file, while disposing all battle handles per case. */
export function createTestFactory(): GameEngineFactory {
  const factory = (shared ??= createNativeEngineFactory(new NativeEngineFactory()));
  const track = (result: ReturnType<GameEngineFactory["create"]>) => {
    if (result.ok) handles.add(result.value);
    return result;
  };
  return {
    catalog: () => factory.catalog(),
    create: (...args) => track(factory.create(...args)),
    restore: (...args) => track(factory.restore(...args)),
    dispose: () => {
      for (const handle of handles) handle.dispose();
      handles.clear();
      return { ok: true, value: null };
    },
  };
}
afterEach(() => {
  for (const handle of handles) handle.dispose();
  handles.clear();
});
afterAll(() => {
  shared?.dispose();
  shared = undefined;
});
