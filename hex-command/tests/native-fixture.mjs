import { afterEach, afterAll } from "vitest";
import * as Native from "./native-adapter.mjs";
import * as Catalog from "./catalog-adapter.mjs";
const factory = await Native.create_factory();
const catalog = Native.catalog(factory);
const handles = /* @__PURE__ */ new Set();
function createTestFactory() {
  return factory;
}
function createBattle(start, seed = 17) {
  const battle = Native.create(factory, start, seed);
  handles.add(battle);
  return battle;
}
function restoreBattle(checkpoint) {
  const battle = Native.restore(factory, checkpoint);
  handles.add(battle);
  return battle;
}
afterEach(() => {
  for (const battle of handles) Native.dispose(battle);
  handles.clear();
});
afterAll(() => Native.dispose_factory(factory));
export { Catalog, Native, catalog, createBattle, createTestFactory, factory, restoreBattle };
