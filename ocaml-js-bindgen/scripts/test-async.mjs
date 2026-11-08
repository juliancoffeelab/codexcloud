import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
for (const target of ["melange", "js_of_ocaml", "rescript", "lwt"]) {
  const backend = target === "lwt" ? "js_of_ocaml" : target;
  const dir = resolve(root, "demos/async-search/generated", target);
  // Drive the platform's finalization callback deterministically without a
  // production test hook, real GC timing, or strong references to its targets.
  const registrations = new WeakMap();
  let finalize;
  const NativeRegistry = globalThis.FinalizationRegistry;
  globalThis.FinalizationRegistry = class {
    constructor(callback) {
      finalize = callback;
    }
    register(target, heldValue, token) {
      registrations.set(token ?? target, heldValue);
    }
    unregister(token) {
      return registrations.delete(token);
    }
  };
  let initialize, api;
  try {
    ({ initialize, api } = await import(`${dir}/bindings.runtime.mjs`));
  } finally {
    globalThis.FinalizationRegistry = NativeRegistry;
  }
  const bytes = await readFile(`${dir}/backend.wasm`);
  const i = await initialize(bytes, { diagnostics: true });
  // The ordinary jsoo Promise adapter has no cancellation channel.
  // The Lwt fixture retains the four-callback cancellation protocol.
  const taskReceive =
    api.task_receive &&
    ((token, ok, cancelled, nativeError, boundaryError) =>
      api.task_cancel
        ? api.task_receive(token, ok, cancelled, nativeError, boundaryError)
        : api.task_receive(token, ok, nativeError, boundaryError));
  const taskAwait =
    api.task_await ??
    ((token) =>
      new Promise((resolve, reject) =>
        taskReceive(
          token,
          (value) => {
            resolve(value);
            return true;
          },
          () => {
            reject(new DOMException("Rust operation cancelled", "AbortError"));
            return true;
          },
          (error) => {
            reject(error);
            return true;
          },
          (reason) => {
            reject(new Error(reason));
            return true;
          },
        ),
      ));
  const text = (s) =>
    backend !== "rescript" ? Buffer.from(s, "utf8").toString("latin1") : s;
  // Raw js_of_ocaml stubs accept a byte count and compiler fill callback.
  const stringArgs = (value) => {
    if (backend !== "js_of_ocaml") return [text(value)];
    const bytes = new TextEncoder().encode(value);
    return [bytes.length, (view, offset) => view.set(bytes, offset)];
  };
  const writer = stringArgs;
  const decode = (r, read) => {
    try {
      const value = read(r);
      api.finish(r);
      return value;
    } finally {
      api.close(r);
    }
  };
  const owner = (name, _type) => api[`call_${name}`]();
  const awaitTask = (token, read) => {
    api.task_map(token, read ? (r) => decode(r, read) : (value) => value);
    return taskAwait(token).catch((error) => {
      if (error?.name === "AbortError") return "cancelled";
      throw error;
    });
  };
  const callback = (handler) => (r) => {
    const n = decode(r, api.r_u32);
    handler(n);
  };
  // Per-call roots: preflight rollback cannot exhaust callback capacity.
  const staleCorpus = owner("corpus", "corpus");
  api.dispose_Corpus(staleCorpus);
  const plain = callback(() => {});
  for (let n = 0; n < 1100; n++)
    assert.throws(
      () => api.call_callback_then_borrow(plain, staleCorpus),
      /disposed/,
    );
  assert.equal(i.e.__ocamljs_host_live(), 0);
  // A host callback can itself throw RuntimeError without trapping this Wasm.
  // Check generic calls, scalar fast wrappers and explicit owner disposal.
  const hostError = new WebAssembly.RuntimeError("host callback error");
  const hostCorpus = owner("corpus", "corpus");
  const hostCallback = callback(() => {
    throw hostError;
  });
  assert.throws(
    () => api.call_callback_then_borrow(hostCallback, hostCorpus),
    (error) => error === hostError,
  );
  assert.equal(api.module_failed(), false);
  const hostDropper = api.call_dropper(hostCallback);
  assert.throws(
    () => api.call_notify_dropper(hostDropper),
    (error) => error === hostError,
  );
  assert.equal(api.module_failed(), false);
  assert.throws(
    () => api.dispose_Dropper(hostDropper),
    (error) => error === hostError,
  );
  assert.equal(api.module_failed(), false);
  api.dispose_Corpus(hostCorpus);
  assert.equal(i.e.__ocamljs_host_live(), 0);
  // One OCaml function can back independent registrations. Dropping either
  // Rust owner must preserve the other registration and its closure.
  let dropped = 0;
  const sharedCallback = callback(() => dropped++);
  const firstDropper = api.call_dropper(sharedCallback);
  const secondDropper = api.call_dropper(sharedCallback);
  assert.equal(i.e.__ocamljs_host_live(), 2);
  api.dispose_Dropper(firstDropper);
  assert.equal(dropped, 1);
  assert.equal(i.e.__ocamljs_host_live(), 1);
  api.dispose_Dropper(secondDropper);
  assert.equal(dropped, 2);
  assert.equal(i.e.__ocamljs_host_live(), 0);
  for (const thrown of [
    new Error("drop owner callback"),
    0,
    false,
    null,
    undefined,
  ]) {
    const dropper = api.call_dropper(() => {
      throw thrown;
    });
    let caught = false;
    try {
      api.dispose_Dropper(dropper);
    } catch (error) {
      caught = true;
      assert.equal(error, thrown);
    }
    assert(caught, "destructor failure escaped its operation");
    const clean = owner("corpus", "corpus");
    api.dispose_Corpus(clean);
    assert.equal(i.e.__ocamljs_host_live(), 0);
  }
  // A poll can return an owned value after its progress callback throws.
  // Reclaim it under the same reentry guard as explicit disposal, and preserve
  // the original exception if its destructor also calls a throwing callback.
  for (const thrown of [
    new Error("progress failure"),
    0,
    false,
    null,
    undefined,
  ]) {
    let callbacks = 0;
    const task = api.call_dropper_after_progress(() => {
      callbacks++;
      assert.throws(() => api.call_corpus(), /reentr/);
      throw callbacks === 1 ? thrown : new Error("destructor failure");
    });
    await assert.rejects(
      awaitTask(task, () => assert.fail("failed callback decoded its result")),
      (error) => Object.is(error, thrown),
    );
    assert.equal(callbacks, 2);
    assert.equal(i.e.__ocamljs_host_live(), 0);
    assert.equal(i.e.__ocamljs_live(), 0);
    const clean = owner("corpus", "corpus");
    api.dispose_Corpus(clean);
  }
  const c = owner("corpus", "corpus"),
    input = owner("input", "input");
  const progress = [];
  const registration = callback((n) => progress.push(n));
  const task = api.call_build_index(c, input, registration);
  const done = awaitTask(task, (r) => {
    assert.equal(api.r_u8(r), 0);
    return api.r_handle_index(r);
  });
  assert.throws(() => api.call_revision(c), /borrow/);
  // A second ordinary Rust task completes while the first awaits its input gate.
  assert.equal(
    await awaitTask(api.call_digest(...writer("λ🦀")), (r) => {
      assert.equal(api.r_u8(r), 0);
      return api.r_u32(r);
    }),
    2,
  );
  assert.equal(i.e.__ocamljs_tasks(), 1);
  const polls = i.stats.polls;
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(i.stats.polls, polls, "idle future was busy-polled");
  const lines =
    Array.from(
      { length: 32 },
      (_, n) => JSON.stringify({ text: `OCaml ${n}` }) + "\n",
    ).join("") + "\n".repeat(256);
  decode(api.call_push(input, ...writer(lines)), (r) =>
    assert.equal(api.r_u8(r), 0),
  );
  api.call_finish(input);
  const index = await done;
  assert.equal(api.call_revision(c), 1);
  assert.deepEqual(progress, [16, 32]);
  api.dispose_Index(index);
  // A stale retained waker after cancellation cannot poll a recycled task slot.
  if (api.task_cancel) {
    const second = api.call_build_index(c, input, registration);
    const cancelled = awaitTask(second, () =>
      assert.fail("cancelled task decoded"),
    );
    assert(api.task_cancel(second));
    assert.equal(await cancelled, "cancelled");
  }
  api.dispose_Input(input);
  api.dispose_Corpus(c);
  assert.equal(i.e.__ocamljs_live(), 0);
  // Finalizer fallback must retry an owner after the future's lease is released.
  if (api.task_cancel) {
    const leased = owner("corpus", "corpus"),
      channel = owner("input", "input"),
      quiet = callback(() => {});
    const pending = api.call_build_index(leased, channel, quiet);
    api.task_map(pending, (r) => decode(r, api.r_u8));
    const heldOwner = registrations.get(leased);
    assert(heldOwner, "owner was not registered for finalization");
    registrations.delete(leased);
    finalize(heldOwner);
    assert.equal(i.finalizerPending.size, 1);
    api.task_cancel(pending);
    assert.equal(i.finalizerPending.size, 0);
    api.dispose_Input(channel);
    assert.equal(i.e.__ocamljs_live(), 0);
  }
  // Callback exceptions belong to the originating async poll and do not poison
  // the next ordinary call or leave a future/lease behind.
  const errorCorpus = owner("corpus", "corpus"),
    errorInput = owner("input", "input");
  const throwing = callback(() => {
    throw new Error("async hook failure");
  });
  const errorTask = api.call_build_index(errorCorpus, errorInput, throwing);
  const failure = awaitTask(errorTask, () =>
    assert.fail("throwing callback completed"),
  );
  decode(api.call_push(errorInput, ...writer(lines)), (r) =>
    assert.equal(api.r_u8(r), 0),
  );
  await assert.rejects(failure, /async hook failure/);
  assert.equal(api.call_revision(errorCorpus), 0);
  assert.equal(i.e.__ocamljs_tasks(), 0);
  api.dispose_Input(errorInput);
  api.dispose_Corpus(errorCorpus);
  assert.equal(i.e.__ocamljs_live(), 0);
  // Cancellation runs Rust destructors before exposing the callback exception.
  if (api.task_cancel) {
    const dc = owner("corpus", "corpus"),
      di = owner("input", "input");
    let duringDrop = false;
    const hook = callback(() => {
      duringDrop = true;
      assert.throws(() => api.call_revision(dc), /reentrant/);
      throw new Error("drop callback");
    });
    const dt = api.call_drop_hook(dc, di, hook);
    api.task_map(dt, (r) => decode(r, api.r_u32));
    while (!api.call_waiting(di))
      await new Promise((resolve) => setImmediate(resolve));
    const dropFailure = taskAwait(dt);
    assert(api.task_cancel(dt));
    await assert.rejects(dropFailure, /drop callback/);
    assert(duringDrop);
    assert.equal(api.call_revision(dc), 0);
    assert.equal(i.e.__ocamljs_tasks(), 0);
    api.dispose_Input(di);
    api.dispose_Corpus(dc);
  }
  // A cancelled native receiver declines an owned typed exception and its
  // payload is reclaimed instead of remaining rooted in a task protocol.
  if (api.task_receive) {
    const declinedError = api.call_throw_owned();
    api.task_map(declinedError, (r) => {
      const owner = decode(r, (r) => {
        assert.equal(api.r_u8(r), 1);
        return api.r_handle_index(r);
      });
      throw Object.assign(new Error("declined typed exception"), { owner });
    });
    await new Promise((resolve) =>
      taskReceive(
        declinedError,
        () => assert.fail("unexpected success"),
        () => assert.fail("unexpected cancel"),
        () => {
          queueMicrotask(resolve);
          return false;
        },
        () => assert.fail("unexpected boundary"),
      ),
    );
    assert.equal(i.e.__ocamljs_live(), 0);
    assert.throws(
      () =>
        taskReceive(
          declinedError,
          () => true,
          () => true,
          () => true,
          () => true,
        ),
      /receiver already attached/,
    );
  }
  // Borrowed text/bytes remain valid across suspension and linear-memory growth.
  const numericArgs = (values, kind) =>
    backend === "js_of_ocaml"
      ? [
          values.length,
          (view, offset) => {
            for (let j = 0; j < values.length; j++)
              api[`numeric_set_${kind}`](view, offset + j, values[j]);
          },
        ]
      : [values];
  let numericProgress = 0;
  assert.equal(
    api.call_numeric_notify(
      ...numericArgs([1, 2], 7),
      callback((n) => (numericProgress = n)),
      ...numericArgs([3, 4], 7),
    ),
    10,
  );
  assert.equal(numericProgress, 2);
  const bt = api.call_borrowed_text(
    ...stringArgs("λ🦀"),
    ...numericArgs(new Uint8Array([2, 3]), 0),
  );
  const bd = awaitTask(bt);
  i.e.memory.grow(1);
  assert.equal(await bd, 7);
  const numericTask = () =>
    api.call_numeric_sum_after_yield(
      ...numericArgs([1, 2], 7),
      ...numericArgs([3, 4], 7),
    );
  const nt = awaitTask(numericTask());
  i.e.memory.grow(1);
  assert.equal(await nt, 10);
  if (api.task_cancel) {
    const cancelledNumeric = numericTask();
    const cancelledNumericResult = awaitTask(cancelledNumeric);
    assert(api.task_cancel(cancelledNumeric));
    assert.equal(await cancelledNumericResult, "cancelled");
    assert.equal(i.e.__ocamljs_live(), 0);
  }

  // A full fetch queue sleeps on a capacity notification; cancellation also
  // releases a parked producer. Transport is controlled without wall-clock sleeps.
  const originalFetch = globalThis.fetch;
  let aborted = false;
  let chunks = Array.from({ length: 12 }, () =>
    new TextEncoder().encode("chunk λ\n"),
  );
  globalThis.fetch = async (_url, { signal }) => {
    signal.addEventListener("abort", () => {
      aborted = true;
    });
    let n = 0;
    return {
      ok: true,
      body: {
        getReader: () => ({
          read: async () =>
            n < chunks.length
              ? { done: false, value: chunks[n++] }
              : { done: true },
          cancel: async () => {},
        }),
      },
    };
  };
  try {
    const gate = owner("input", "input");
    const ft = api.call___ocamljs_Input_fetch_gate(
      gate,
      ...stringArgs("https://fixture.invalid"),
    );
    const result = awaitTask(ft, (r) => {
      assert.equal(api.r_u8(r), 0);
      return api.r_u32(r);
    });
    while (!i.stats.fetchWaits)
      await new Promise((resolve) => setImmediate(resolve));
    assert.equal(i.stats.fetchPushes, 4);
    const stalled = i.stats.fetchWaits;
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(i.stats.fetchWaits, stalled);
    decode(api.call_push(gate, ...writer("open")), (r) =>
      assert.equal(api.r_u8(r), 0),
    );
    assert.equal(
      await result,
      12 * new TextEncoder().encode("chunk λ\n").length,
    );
    assert(i.stats.fetchCredits > 0);
    api.dispose_Input(gate);
    assert.equal(i.fetches.size, 0);
    aborted = false;
    if (api.task_cancel) {
      const gate2 = owner("input", "input"),
        start = i.stats.fetchWaits;
      const blocked = api.call___ocamljs_Input_fetch_gate(
        gate2,
        ...stringArgs("https://fixture.invalid"),
      );
      api.task_map(blocked, (r) => decode(r, api.r_u8));
      while (i.stats.fetchWaits === start)
        await new Promise((resolve) => setImmediate(resolve));
      assert(api.task_cancel(blocked));
      assert(aborted);
      await new Promise((resolve) => setImmediate(resolve));
      assert.equal(i.fetches.size, 0);
      api.dispose_Input(gate2);
      assert.equal(i.e.__ocamljs_live(), 0);
    }
    // A stream's total size belongs to its Rust consumer, not the JS bridge.
    // Even one large browser chunk must pass through the same bounded queue.
    chunks = [new Uint8Array(1024 * 1024 + 1).fill(120)];
    const largeGate = owner("input", "input");
    const largeTask = api.call___ocamljs_Input_fetch_gate(
      largeGate,
      ...stringArgs("https://fixture.invalid"),
    );
    const largeResult = awaitTask(largeTask, (r) => {
      assert.equal(api.r_u8(r), 0);
      return api.r_u32(r);
    });
    decode(api.call_push(largeGate, ...writer("open")), (r) =>
      assert.equal(api.r_u8(r), 0),
    );
    assert.equal(await largeResult, chunks[0].length);
    api.dispose_Input(largeGate);
    assert.equal(i.fetches.size, 0);
    assert.equal(i.e.__ocamljs_live(), 0);
    // Simulate pointer reuse before an aborted fetch finishes, independently of
    // malloc order. The old completion must not enter Wasm or delete its replacement.
    if (api.task_cancel) {
      let finishOldFetch;
      globalThis.fetch = () =>
        new Promise((resolve) => {
          finishOldFetch = resolve;
        });
      const lateGate = owner("input", "input");
      const lateTask = api.call___ocamljs_Input_fetch_gate(
        lateGate,
        ...stringArgs("https://fixture.invalid"),
      );
      api.task_map(lateTask, (r) => decode(r, api.r_u8));
      while (!finishOldFetch)
        await new Promise((resolve) => setImmediate(resolve));
      const oldId = [...i.fetches.keys()][0];
      assert(api.task_cancel(lateTask));
      const replacement = new AbortController();
      i.fetches.set(oldId, replacement);
      const actualExports = i.e;
      let staleCalls = 0;
      i.e = {
        ...actualExports,
        __ocamljs_fetch_push: () => {
          staleCalls++;
          return 0;
        },
        __ocamljs_fetch_end: () => {
          staleCalls++;
        },
      };
      finishOldFetch({
        ok: true,
        body: {
          getReader: () => ({
            read: async () => ({ done: false, value: new Uint8Array([1]) }),
            cancel: async () => {},
          }),
        },
      });
      await new Promise((resolve) => setImmediate(resolve));
      assert.equal(staleCalls, 0);
      assert.equal(i.fetches.get(oldId), replacement);
      i.e = actualExports;
      i.fetches.delete(oldId);
      api.dispose_Input(lateGate);
      assert.equal(i.e.__ocamljs_live(), 0);
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
  // Native adapters have one receiver; a duplicate attachment cannot steal
  // ownership from the original Promise.
  const single = api.call_digest(...writer("hello"));
  api.task_map(single, (r) =>
    decode(r, (r) => {
      assert.equal(api.r_u8(r), 0);
      return api.r_u32(r);
    }),
  );
  const observed = taskAwait(single);
  assert.throws(
    () =>
      api.task_receive
        ? taskReceive(
            single,
            () => true,
            () => true,
            () => true,
            () => true,
          )
        : api.task_await(single),
    /receiver already attached/,
  );
  assert.equal(await observed, 5);
  // A committed result remains immutable when an unrelated operation traps.
  const committed = api.call_digest(...writer("ready"));
  api.task_map(committed, (r) =>
    decode(r, (r) => {
      assert.equal(api.r_u8(r), 0);
      return api.r_u32(r);
    }),
  );
  while (i.jobs.get([...i.jobs.keys()][0])?.status === "pending")
    await new Promise((resolve) => setTimeout(resolve, 0));
  const waitingCorpus = owner("corpus", "corpus"),
    waitingInput = owner("input", "input"),
    last = callback(() => {});
  const waiting = api.call_build_index(waitingCorpus, waitingInput, last);
  const poisoned = awaitTask(waiting, () =>
    assert.fail("poisoned task decoded"),
  );
  assert.throws(() => api.call_force_trap(), WebAssembly.RuntimeError);
  await assert.rejects(poisoned, /unreachable|trap/);
  assert.equal(await taskAwait(committed), 5);
  assert.equal(i.jobs.size, 0);
  assert(
    Array.from({ length: i.e.__ocamljs_host_references.length }, (_, n) =>
      i.e.__ocamljs_host_references.get(n),
    ).every((v) => v === null),
  );
  console.log(
    `async ownership/${target}: suspension, wake, cancellation, cleanup, errors and terminal races passed`,
  );
  for (const [name, value] of Object.entries(globalThis))
    if (name.startsWith("Ocamljs_") && value === api) delete globalThis[name];
}
