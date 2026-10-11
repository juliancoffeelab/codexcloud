/** File-owned OCaml factory. Tests mutate checkpoints, never application helpers. */
import { afterEach, afterAll } from "vitest";
import * as Native from "@frontend/Native";
import * as Catalog from "@frontend/Catalog";
import type { SavedGame, Start } from "../rust/pkg/hex_command";
export { Native, Catalog };
export const factory = await Native.create_factory();
export const catalog = Native.catalog(factory);
const handles = new Set<Native.battle>();
export function createTestFactory() {
  return factory;
}
export function createBattle(start: Start, seed = 17) {
  const battle = Native.create(factory, start, seed);
  handles.add(battle);
  return battle;
}
export function restoreBattle(checkpoint: SavedGame | Native.Immutable<SavedGame>) {
  const battle = Native.restore(factory, checkpoint);
  handles.add(battle);
  return battle;
}
afterEach(() => {
  for (const battle of handles) Native.dispose(battle);
  handles.clear();
});
afterAll(() => Native.dispose_factory(factory));
