import { beforeEach, vi } from "vitest";
import { IDBFactory } from "fake-indexeddb";
import * as Store from "./storage-adapter.mjs";
import { TestStorage } from "./helpers.mjs";
beforeEach(async () => {
  vi.stubGlobal("indexedDB", new IDBFactory());
  vi.stubGlobal("localStorage", new TestStorage());
  const queues = new Map();
  const locks = {
    request(name, work) {
      const next = (queues.get(name) ?? Promise.resolve())
        .catch(() => {})
        .then(() => work({ name }));
      queues.set(name, next);
      return next;
    },
  };
  await Store.initialize({ turns: [], chooseNewGame: false });
  vi.stubGlobal(
    "navigator",
    Object.defineProperty(Object.create(globalThis.navigator ?? null), "locks", {
      value: locks,
      configurable: true,
    }),
  );
});
