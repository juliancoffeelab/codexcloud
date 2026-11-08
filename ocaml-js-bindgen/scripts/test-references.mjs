import assert from "node:assert/strict";
import { readFile, mkdtemp, rm } from "node:fs/promises";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";
const root = resolve(import.meta.dirname, "..");
{
  const ordinary = new WebAssembly.Module(
    await readFile(
      resolve(root, "demos/hello-world/generated/melange/backend.wasm"),
    ),
  );
  assert(
    !WebAssembly.Module.imports(ordinary).some(
      (i) => i.module === "ocamljs_ref" || i.module === "ocamljs_host",
    ),
  );
  assert(
    !WebAssembly.Module.exports(ordinary).some((e) =>
      e.name.startsWith("__ocamljs_host_"),
    ),
  );
}
for (const target of ["melange", "js_of_ocaml", "rescript", "lwt"]) {
  const backend = target === "lwt" ? "js_of_ocaml" : target;
  const dir = resolve(root, "demos/reference-conformance/generated", target);
  const bytes = await readFile(`${dir}/backend.wasm`);
  const module = new WebAssembly.Module(bytes);
  assert(
    WebAssembly.Module.customSections(module, "ocamljs.references").length ===
      1,
  );
  const imports = WebAssembly.Module.imports(module);
  assert(!imports.some((i) => i.module === "ocamljs_ref"));
  const runtime = await readFile(`${dir}/bindings.runtime.mjs`, "utf8");
  assert(!/callbacks: new Map|callback_retain|callback_release/.test(runtime));
  const { initialize, api } = await import(`${dir}/bindings.runtime.mjs`);
  const i = await initialize(bytes);
  const decode = (packet, read) => {
    try {
      const result = read(packet);
      api.finish(packet);
      return result;
    } finally {
      api.close(packet);
    }
  };
  const result = (packet, read = api.r_host) =>
    decode(packet, (r) =>
      api.r_u8(r) === 0 ? { ok: read(r) } : { error: api.r_host(r) },
    );
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
  const task = (token) => taskAwait(api.task_map(token, (r) => result(r)));
  const clean = () => {
    assert.equal(i.e.__ocamljs_host_live(), 0, "host root leak");
    assert.equal(i.e.__ocamljs_live(), 0, "ABI allocation leak");
    for (let n = 0; n < i.e.__ocamljs_host_references.length; n++)
      assert.equal(
        i.e.__ocamljs_host_references.get(n),
        null,
        `retained table slot ${n}`,
      );
  };
  const object = { key: "OCaml λ🦀" };
  // The engine enforces the native export type, before any Rust or JS shim.
  assert.equal(i.e.__ocamljs_fn_identity(object), object);
  for (const value of [
    null,
    undefined,
    0,
    false,
    "λ🦀",
    object,
    new Blob(["λ"]),
    new Uint8Array([1, 2]),
  ])
    assert.equal(api.call_identity(value), value);
  assert.equal(api.call_alias_identity(object), object);
  clean();
  const holder = api.call___ocamljs_Holder_create(object);
  assert.equal(i.e.__ocamljs_host_live(), 1);
  assert.equal(api.call___ocamljs_Holder_value(holder), object);
  api.dispose_Holder(holder);
  clean();
  const { thrown } = await import(`${dir}/third-party.mjs`);
  for (let n = 0; n < 32; n++)
    assert.equal(result(api.call_fail()).error, thrown);
  const jsonArgs =
    backend === "js_of_ocaml"
      ? [
          1,
          (view, offset) => {
            view[offset] = 123;
          },
        ]
      : ["{"];
  assert(result(api.call_parse_json(...jsonArgs)).error instanceof SyntaxError);
  assert.equal(result(api.call_unsigned(), api.r_u64).ok, "-1");
  clean();
  const buffer = result(api.call_buffer(32)).ok;
  const view = result(api.call_bytes(buffer)).ok;
  assert(view instanceof Uint8Array);
  assert.equal(view.buffer, buffer);
  assert.equal(result(api.call_paint_bytes(view, 173)).ok, view);
  assert(view.every((v) => v === 173));
  assert.equal(result(api.call_read_byte(view, 31), api.r_u8).ok, 173);
  clean();
  const copyResult = (packet, read = api.r_host) =>
    typeof packet === "string"
      ? { ok: packet }
      : decode(packet, (r) => {
          if (api.r_u8(r) === 0) return { ok: read(r) };
          const tag = api.r_u32(r);
          return tag === 0 ? { tooLarge: true } : { error: api.r_host(r) };
        });
  for (const text of [
    "",
    "ASCII\0",
    "a".repeat(15) + "\0",
    "a".repeat(16) + "\0",
    "a".repeat(14) + "é",
    "a".repeat(15) + "é",
    "\ufeffOCaml λ🦀",
    "\ud800",
    // UTF-8 widths, combining text and a four-byte code point crossing the
    // native byte-string chunk boundary must retain their exact byte spelling.
    "éÿ中𝄞e\u0301\u0080\u009f",
    // Smallest/largest scalar of each UTF-8 width, surrogate neighbors,
    // noncharacters, BOM and the exact tiny-pool boundary are all valid.
    "\u007f\u0080\u07ff",
    "\u0800\ud7ff\ue000\uffff",
    "\u{10000}\u{10ffff}\ufeff\0",
    "é".repeat(8),
    "é".repeat(8) + "x",
    "a".repeat(4095) + "🦀\ufeffλ",
  ]) {
    const expected = new TextDecoder("utf-8", { ignoreBOM: true }).decode(
      new TextEncoder().encode(text),
    );
    const size = new TextEncoder().encode(text).length;
    const encode = TextEncoder.prototype.encode;
    let encodes = 0;
    TextEncoder.prototype.encode = function (...args) {
      encodes++;
      return encode.apply(this, args);
    };
    try {
      assert.equal(
        api.host_string(copyResult(api.call_copy_text(text, size)).ok),
        expected,
      );
      assert.equal(encodes, 0, "Reading a UTF-8 result must not re-encode it");
    } finally {
      TextEncoder.prototype.encode = encode;
    }
    if (size)
      assert.deepEqual(copyResult(api.call_copy_text(text, size - 1)), {
        tooLarge: true,
      });
    clean();
  }
  assert(copyResult(api.call_copy_text({}, 16)).error instanceof TypeError);
  for (const text of ["\0", "\ufeff\u0080\u009fλ🦀éÿ"]) {
    assert.equal(api.host_string(api.native_string(text)), text);
  }
  for (const host of ["\u0100", "🦀"]) {
    if (backend === "rescript") assert.equal(api.host_string(host), host);
    else
      assert.throws(
        () => api.host_string(host),
        /expected native UTF-8 OCaml string/,
      );
  }
  for (const malformed of [
    [0x80],
    [0xff],
    [...Array(15).fill(97), 0x80],
    [...Array(16).fill(97), 0xc2],
    [0xc0, 0xaf],
    [0xc2],
    [0xe2, 0x28, 0xa1],
    [0xed, 0xa0, 0x80],
    [0xf4, 0x90, 0x80, 0x80],
    [0xe0, 0x9f, 0xbf],
    [0xed, 0xbf, 0xbf],
    [0xf0, 0x8f, 0xbf, 0xbf],
    [0xf5, 0x80, 0x80, 0x80],
    [0xef, 0xbf],
    [0xf0, 0x90, 0x80],
    [0xf0, 0x90, 0x80, 0x41],
    [...Array(14).fill(97), 0xe0, 0x80],
    [...Array(14).fill(97), 0xe0, 0x80, 0x80],
  ]) {
    if (backend !== "rescript")
      assert.throws(
        () => api.host_string(String.fromCharCode.apply(null, malformed)),
        TypeError,
      );
    const raw = i.e;
    let frees = 0;
    i.e = {
      ...raw,
      __ocamljs_fn_copy_text: (...args) => {
        const result = raw.__ocamljs_fn_copy_text(...args);
        const words = new Uint32Array(raw.memory.buffer);
        assert.equal(words[result >>> 2], 0);
        const stringInfo = words[(result >>> 2) + 1];
        new Uint8Array(
          raw.memory.buffer,
          words[stringInfo >>> 2],
          malformed.length,
        ).set(malformed);
        return result;
      },
      __ocamljs_string_free: (...args) => {
        frees++;
        return raw.__ocamljs_string_free(...args);
      },
    };
    try {
      assert.throws(
        () =>
          api.call_copy_text("a".repeat(malformed.length), malformed.length),
        TypeError,
        "Malformed UTF-8 must be rejected before native string construction",
      );
    } finally {
      i.e = raw;
    }
    assert.equal(
      frees,
      1,
      "failed eager string construction must release its Rust allocation",
    );
    clean();
  }
  {
    const text = "\ufeffDetached λ🦀";
    const length = new TextEncoder().encode(text).length;
    const raw = i.e;
    let frees = 0;
    i.e = {
      ...raw,
      __ocamljs_string_free: (pointer, capacity) => {
        new Uint8Array(raw.memory.buffer, pointer, length).fill(0);
        frees++;
        return raw.__ocamljs_string_free(pointer, capacity);
      },
    };
    let native;
    try {
      native = api.call_copy_text(text, length);
    } finally {
      i.e = raw;
    }
    assert.equal(frees, 1);
    i.e.memory.grow(1);
    assert.equal(
      api.host_string(native),
      text,
      "Returned strings own their bytes after output overwrite, release and memory growth",
    );
    clean();
  }

  const sourcePixels = new Uint8ClampedArray([99, 0, 127, 255, 88]).subarray(
    1,
    4,
  );
  const copied = copyResult(api.call_copy_pixels(sourcePixels, 3)).ok;
  assert.deepEqual([...copied], [0, 127, 255]);
  assert.notEqual(copied.buffer, sourcePixels.buffer);
  sourcePixels[0] = 66;
  assert.equal(copied[0], 0);
  assert.deepEqual(copyResult(api.call_copy_pixels(sourcePixels, 2)), {
    tooLarge: true,
  });
  assert.equal(
    copyResult(api.call_copy_pixels(new Uint8ClampedArray(), 0)).ok.length,
    0,
  );
  const detached = new Uint8ClampedArray([1]);
  structuredClone(detached.buffer, { transfer: [detached.buffer] });
  assert(
    copyResult(api.call_copy_pixels(detached, 1)).error instanceof TypeError,
  );
  const length = 1024 * 1024;
  const input = i.e.__ocamljs_alloc(length);
  try {
    const pixels = new Uint8ClampedArray(
      i.e.memory.buffer,
      i.e.__ocamljs_input_ptr(input) >>> 0,
      length,
    );
    pixels.fill(173);
    const before = i.e.memory.buffer;
    const safe = copyResult(api.call_copy_pixels(pixels, length)).ok;
    assert.notEqual(
      i.e.memory.buffer,
      before,
      "fixture must grow Wasm memory during destination allocation",
    );
    assert.equal(safe.length, length);
    assert(
      safe.every((v) => v === 173),
      "Wasm-backed input lost contents during growth",
    );
  } finally {
    i.e.__ocamljs_input_free(input);
  }
  clean();
  let resolvePromise;
  const pending = new Promise((resolve) => {
    resolvePromise = resolve;
  });
  assert.equal(api.call_promise_identity(pending), pending);
  assert.equal(api.call_alias_promise(pending), pending);
  const eventValues = [];
  const token = api.call_notify_after(pending, (r) => {
    eventValues.push(api.r_host(r));
    assert.equal(
      api.host_string(api.r_pool_string(r, api.r_u32(r))),
      "\ufeffHost λ🦀",
    );
    assert.equal(api.r_u32(r), 2);
    api.finish(r);
  });
  const waiting = task(token);
  assert(
    i.e.__ocamljs_host_live() > 0,
    "suspended future failed to retain its inputs",
  );
  assert.deepEqual(eventValues, []);
  resolvePromise(object);
  assert.equal((await waiting).ok, object);
  assert.deepEqual(eventValues, [object]);
  clean();
  for (const error of [thrown, null, undefined, 0, false, "failure"]) {
    const rejected = Promise.reject(error);
    assert.equal((await task(api.call_await_borrowed(rejected))).error, error);
    assert.equal(
      (await task(api.call_await_delayed(Promise.reject(error)))).error,
      error,
    );
    clean();
  }
  if (api.task_cancel) {
    const neverStarted = api.call_await_object(Promise.reject(thrown));
    const neverStartedResult = task(neverStarted);
    const neverStartedCheck = assert.rejects(neverStartedResult, {
      name: "AbortError",
    });
    api.task_cancel(neverStarted);
    await neverStartedCheck;
    await new Promise((resolve) => setImmediate(resolve));
    clean();
    // Cancellation retires the host subscription before freeing its Rust pointer. Late
    // settlement must not revive it or target a subsequently allocated waiter.
    let settleLate, settleNext;
    const waitForSubscription = async () => {
      const deadline = performance.now() + 2000;
      while (!i.hostPromises.size) {
        assert(
          performance.now() < deadline,
          "Promise future did not subscribe",
        );
        await new Promise((resolve) => setImmediate(resolve));
      }
    };
    const old = api.call_await_object(
      new Promise((resolve) => {
        settleLate = resolve;
      }),
    );
    const cancelled = task(old);
    const cancelledCheck = assert.rejects(cancelled, { name: "AbortError" });
    await waitForSubscription();
    assert.equal(api.task_cancel(old), true);
    await cancelledCheck;
    clean();
    const next = task(
      api.call_await_object(
        new Promise((resolve) => {
          settleNext = resolve;
        }),
      ),
    );
    await waitForSubscription();
    const actualExports = i.e;
    let lateSettles = 0;
    i.e = {
      ...actualExports,
      __ocamljs_host_settle: (...args) => {
        lateSettles++;
        return actualExports.__ocamljs_host_settle(...args);
      },
    };
    settleLate({ stale: true });
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(
      lateSettles,
      0,
      "cancelled subscription reentered Wasm with a freed pointer",
    );
    i.e = actualExports;
    settleNext(object);
    assert.equal((await next).ok, object);
    clean();
  }
  // Malformed pooled bytes after extracting an externref must release every
  // callback output root and table slot, including the still-borrowed input.
  assert.throws(
    () =>
      api.call_notify(object, (r) => {
        assert.equal(api.r_host(r), object);
        new Uint8Array(i.e.memory.buffer, r.poolPtr, r.poolLength).fill(0xff);
        api.r_pool_string(r, api.r_u32(r));
      }),
    TypeError,
  );
  clean();
  // Packet cleanup must release a partially decoded host record too.
  assert.throws(
    () =>
      api.call_notify(object, () => {
        throw thrown;
      }),
    (error) => error === thrown,
  );
  clean();
  // No ever-growing table or root allocator under repeated round trips.
  for (let n = 0; n < 10000; n++)
    assert.equal(api.call_identity(object), object);
  assert(i.e.__ocamljs_host_references.length <= 256);
  clean();
  delete globalThis.Ocamljs_reference_conformance;
  console.log(
    `references/${backend}: native identity, imports, unsigned values, buffers, retained callbacks, pending/rejected promises, cancellation and zero residual roots`,
  );
}
for (const feature of [
  "primitive-only",
  "builtin-only",
  "error-only",
  "text-only",
  "pixels-only",
]) {
  const folder = resolve(
    root,
    "demos/reference-conformance/generated",
    feature,
  );
  const { initialize, api } = await import(`${folder}/bindings.runtime.mjs`);
  const i = await initialize(await readFile(`${folder}/backend.wasm`));
  if (feature === "primitive-only") {
    const schema = JSON.parse(await readFile(`${folder}/schema.json`, "utf8"));
    assert(!Object.keys(schema).some((name) => name.includes("disabled")));
    const packet = api.call_abs(-42);
    try {
      assert.equal(api.r_u8(packet), 0);
      assert.equal(api.r_f64(packet), 42);
      api.finish(packet);
    } finally {
      api.close(packet);
    }
  } else if (feature === "text-only" || feature === "pixels-only") {
    const value =
      feature === "text-only" ? "native λ🦀" : new Uint8ClampedArray([1, 2, 3]);
    assert.equal(api.call_identity(value), value);
  } else {
    const error = { native: true };
    assert.equal(api.call_identity(error), error);
    if (feature === "builtin-only") {
      const promise = Promise.resolve(error);
      assert.equal(api.call_pending(promise), promise);
    }
  }
  assert.equal(i.e.__ocamljs_host_live(), 0);
  assert.equal(i.e.__ocamljs_live(), 0);
  delete globalThis[`Ocamljs_${feature.replaceAll("-", "_")}`];
}
// Post-link processing must be idempotent and retain the native signatures.
const temp = await mkdtemp(resolve(root, "tmp/references-"));
try {
  const r = spawnSync(
    resolve(root, "target/debug/ocamljs_pack"),
    [
      "generate",
      "--wasm",
      resolve(
        root,
        "demos/reference-conformance/generated/melange/backend.wasm",
      ),
      "--schema",
      resolve(
        root,
        "demos/reference-conformance/generated/melange/schema.json",
      ),
      "--out-dir",
      temp,
      "--name",
      "reference_roundtrip",
    ],
    { encoding: "utf8" },
  );
  assert.equal(r.status, 0, r.stderr);
  const regenerated = await readFile(resolve(temp, "backend.wasm"));
  assert(WebAssembly.validate(regenerated));
  assert.equal(
    WebAssembly.Module.customSections(
      new WebAssembly.Module(regenerated),
      "ocamljs.references",
    ).length,
    1,
  );
} finally {
  await rm(temp, { recursive: true, force: true });
}
