import assert from "node:assert/strict";
import { test } from "vitest";
import * as Startup from "@frontend/Startup";

function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (error: unknown) => void;
  const promise = new Promise<T>((accept, fail) => {
    resolve = accept;
    reject = fail;
  });
  return { promise, resolve, reject };
}

/** Controlled foreign scheduling edge; all acquisition policy runs in OCaml. */
function queued() {
  let callback: (() => void) | undefined,
    cancellations = 0;
  return {
    schedule: (acquire: () => void) => {
      callback = acquire;
      return () => {
        cancellations++;
      };
    },
    fire: () => {
      assert(callback);
      callback();
    },
    cancellations: () => cancellations,
  };
}

async function settle() {
  // Drain nested OCaml Promise callbacks without introducing clock-dependent waits.
  for (let i = 0; i < 6; i++) await Promise.resolve();
}

function failed(phase: Startup.phase | undefined): string {
  assert(phase !== undefined && typeof phase === "object");
  return phase._0;
}

test("Startup publishes the loading shell before scheduled application acquisition", async () => {
  const queue = queued(),
    pending = deferred<Startup.cleanup>(),
    phases: Startup.phase[] = [];
  let acquisitions = 0,
    current!: () => boolean,
    releases = 0;
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    (isCurrent) => {
      acquisitions++;
      current = isCurrent;
      return pending.promise;
    },
  );
  assert.deepEqual(phases, [0]);
  assert.equal(acquisitions, 0);
  queue.fire();
  assert.equal(acquisitions, 1);
  assert(current());
  assert.deepEqual(phases, [0]);
  pending.resolve(() => {
    releases++;
  });
  await settle();
  assert.deepEqual(phases, [0, 1]);
  assert(current());
  Startup.close(owner);
  Startup.close(owner);
  assert(!current());
  assert.equal(releases, 1);
});

test("Closing a queued startup cancels acquisition even if a stale frame callback runs", async () => {
  const queue = queued(),
    phases: Startup.phase[] = [];
  let acquisitions = 0;
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    async () => {
      acquisitions++;
      return () => {};
    },
  );
  Startup.close(owner);
  Startup.close(owner);
  assert.equal(queue.cancellations(), 1);
  queue.fire();
  await settle();
  assert.equal(acquisitions, 0);
  assert.deepEqual(phases, [0]);
});

test("Current acquisition failures publish a readable error after the loading shell", async () => {
  const queue = queued(),
    pending = deferred<Startup.cleanup>(),
    phases: Startup.phase[] = [];
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    () => pending.promise,
  );
  queue.fire();
  pending.reject(new Error("WASM could not be loaded"));
  await settle();
  assert.equal(phases.length, 2);
  assert.match(failed(phases[1]), /WASM could not be loaded/);
  Startup.close(owner);
  Startup.close(owner);
});

test("Closing during acquisition invalidates its predicate and ignores stale failure", async () => {
  const queue = queued(),
    pending = deferred<Startup.cleanup>(),
    phases: Startup.phase[] = [];
  let current!: () => boolean;
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    (isCurrent) => {
      current = isCurrent;
      return pending.promise;
    },
  );
  queue.fire();
  assert(current());
  Startup.close(owner);
  assert(!current());
  pending.reject(new Error("Late network failure"));
  await settle();
  assert.deepEqual(phases, [0]);
});

test("A stale successful acquisition releases its application without publishing Ready", async () => {
  const queue = queued(),
    pending = deferred<Startup.cleanup>(),
    phases: Startup.phase[] = [];
  let releases = 0;
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    () => pending.promise,
  );
  queue.fire();
  Startup.close(owner);
  pending.resolve(() => {
    releases++;
    Startup.close(owner);
  });
  await settle();
  Startup.close(owner);
  assert.equal(releases, 1);
  assert.deepEqual(phases, [0]);
});

test("A Ready listener may close immediately because application ownership is already installed", async () => {
  const queue = queued(),
    pending = deferred<Startup.cleanup>(),
    phases: Startup.phase[] = [];
  let owner!: Startup.t,
    releases = 0,
    current!: () => boolean;
  owner = Startup.create(
    (phase) => {
      phases.push(phase);
      if (phase === 1) Startup.close(owner);
    },
    queue.schedule,
    (isCurrent) => {
      current = isCurrent;
      return pending.promise;
    },
  );
  queue.fire();
  pending.resolve(() => {
    releases++;
    Startup.close(owner);
  });
  await settle();
  assert.deepEqual(phases, [0, 1]);
  assert(!current());
  assert.equal(releases, 1);
  Startup.close(owner);
  assert.equal(releases, 1);
});

test("Synchronous acquisition exceptions become a readable failed shell", async () => {
  const queue = queued(),
    phases: Startup.phase[] = [];
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    () => {
      throw new Error("Dynamic module acquisition failed");
    },
  );
  assert.doesNotThrow(queue.fire);
  await settle();
  assert.equal(phases.length, 2);
  assert.match(failed(phases[1]), /Dynamic module acquisition failed/);
  Startup.close(owner);
});

test("Cancelling a queued frame may reenter close without cancelling twice", () => {
  let owner!: Startup.t,
    cancellations = 0,
    acquisitions = 0;
  owner = Startup.create(
    () => {},
    () => () => {
      cancellations++;
      Startup.close(owner);
    },
    async () => {
      acquisitions++;
      return () => {};
    },
  );
  Startup.close(owner);
  Startup.close(owner);
  assert.equal(cancellations, 1);
  assert.equal(acquisitions, 0);
});

test("Late render failures release the application once and cannot be overwritten by ready work", async () => {
  const queue = queued(),
    phases: Startup.phase[] = [];
  let releases = 0,
    current!: () => boolean;
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    async (isCurrent) => {
      current = isCurrent;
      return () => {
        releases++;
      };
    },
  );
  queue.fire();
  await settle();
  assert.deepEqual(phases, [0, 1]);
  Startup.fail(owner, "Render failed");
  assert.equal(failed(phases.at(-1)), "Render failed");
  assert.equal(current(), false);
  assert.equal(releases, 1);
  Startup.fail(owner, "Stale failure");
  Startup.close(owner);
  assert.equal(phases.length, 3);
  assert.equal(releases, 1);
});

test("A render failure while loading releases late resources without publishing ready", async () => {
  const queue = queued(),
    pending = deferred<Startup.cleanup>(),
    phases: Startup.phase[] = [];
  let releases = 0;
  const owner = Startup.create(
    (phase) => phases.push(phase),
    queue.schedule,
    () => pending.promise,
  );
  queue.fire();
  Startup.fail(owner, "Shell failed");
  pending.resolve(() => {
    releases++;
  });
  await settle();
  assert.equal(failed(phases.at(-1)), "Shell failed");
  assert.equal(phases.length, 2);
  assert.equal(releases, 1);
  Startup.close(owner);
  assert.equal(releases, 1);
});
