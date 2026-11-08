// First-principles OCamlJS ABI runtime. Generated native codecs own OCaml values.
// #if references
__HOST_MODULES__;
// #endif
const stamp = "__STAMP__";
const byteStrings = __BYTE_STRINGS__;
const paranoid = __PARANOID__;
let nativeByteDecoder;
function asciiBytes(bytes, start = 0, end = bytes.length) {
  const n = end - start,
    data = new DataView(bytes.buffer, bytes.byteOffset + start, n);
  let j = 0;
  for (; j + 4 <= n; j += 4)
    if (data.getUint32(j, true) & 0x80808080) return false;
  for (; j < n; j++) if (bytes[start + j] & 128) return false;
  return true;
}
function shortAscii(bytes, start, end) {
  if (end - start > 16 || bytes[start] >= 128) return false;
  for (let at = start + 1; at < end; at++) if (bytes[at] >= 128) return false;
  return true;
}
function nativeBytes(bytes, start = 0, end = bytes.length, nonAscii = false) {
  // Short owned ranges need only the result string, without a temporary view.
  switch (end - start) {
    case 1:
      return String.fromCharCode(bytes[start]);
    case 2:
      return String.fromCharCode(bytes[start], bytes[start + 1]);
    case 3:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
      );
    case 4:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
      );
    case 5:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
      );
    case 6:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
      );
    case 7:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
      );
    case 8:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
      );
    case 9:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
      );
    case 10:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
      );
    case 11:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
        bytes[start + 10],
      );
    case 12:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
        bytes[start + 10],
        bytes[start + 11],
      );
    case 13:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
        bytes[start + 10],
        bytes[start + 11],
        bytes[start + 12],
      );
    case 14:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
        bytes[start + 10],
        bytes[start + 11],
        bytes[start + 12],
        bytes[start + 13],
      );
    case 15:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
        bytes[start + 10],
        bytes[start + 11],
        bytes[start + 12],
        bytes[start + 13],
        bytes[start + 14],
      );
    case 16:
      return String.fromCharCode(
        bytes[start],
        bytes[start + 1],
        bytes[start + 2],
        bytes[start + 3],
        bytes[start + 4],
        bytes[start + 5],
        bytes[start + 6],
        bytes[start + 7],
        bytes[start + 8],
        bytes[start + 9],
        bytes[start + 10],
        bytes[start + 11],
        bytes[start + 12],
        bytes[start + 13],
        bytes[start + 14],
        bytes[start + 15],
      );
  }
  if (end - start <= 512)
    return String.fromCharCode.apply(null, bytes.subarray(start, end));
  if (!nonAscii && asciiBytes(bytes, start, end))
    return decoder.decode(bytes.subarray(start, end));
  // Bulk widening preserves all byte values, including C1 controls; latin1
  // TextDecoder labels would instead apply the Windows-1252 character mapping.
  const units = new Uint16Array(end - start);
  units.set(bytes.subarray(start, end));
  nativeByteDecoder ??= new TextDecoder(
    littleEndian ? "utf-16le" : "utf-16be",
    { ignoreBOM: true },
  );
  return nativeByteDecoder.decode(units);
}
function nativeString(value) {
  return !byteStrings || /^[\x00-\x7f]*$/.test(value)
    ? value
    : nativeBytes(encoder.encode(value));
}
function hostString(value) {
  if (!byteStrings) return value;
  // #if native_string_host
  const i = defaultInstance;
  if (value.length > 256 && i && !i.poisoned) {
    let p;
    try {
      p = prepareNativeByteString(i, value, value.length);
      return decoder.decode(stringView(i).subarray(p, p + value.length));
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) poison(i, error);
      throw error;
    } finally {
      if (p !== undefined) freeStringInput(i, p, value.length);
    }
  }
  // #endif
  if (/^[\x00-\x7f]*$/.test(value)) return value;
  const bytes = new Uint8Array(value.length);
  for (let at = 0; at < value.length; at++) {
    const code = value.charCodeAt(at);
    if (code > 255) fail("expected native UTF-8 OCaml string");
    bytes[at] = code;
  }
  return decoder.decode(bytes);
}
// Compiler-native promises may box thenable OCaml values. Observe them through
// the compiler API so Rust receives the actual value without re-assimilation.
// #if jsoo_promise
const promiseObservers = new WeakMap();
// #endif
// #if host_promises
function observePromise(promise, ok, error) {
  // #if jsoo_promise
  const observer = promiseObservers.get(promise);
  if (observer) {
    observer(ok, error);
    return;
  }
  // #endif
  Promise.resolve(promise).then(ok, error);
}
// #endif
const MAX_BYTES = 64 * 1024 * 1024,
  MAX_ITEMS = 1000000,
  MAX_DEPTH = 128;
// #if paranoid
const instances = new WeakSet();
// #endif
// #if owners
class Owner {
  constructor(ptr) {
    this.ptr = ptr;
  }
}
// #endif
// #if async
// #if paranoid
const tasks = new WeakSet();
// #endif
// #endif
// #if owners
const finalizers =
  typeof FinalizationRegistry === "function"
    ? new FinalizationRegistry((id) => {
        finalizeOwner(defaultInstance, id);
        flushFinalizers(defaultInstance);
      })
    : null;
// #endif
const encoder = new TextEncoder(),
  decoder = new TextDecoder("utf-8", { fatal: __PARANOID__, ignoreBOM: true });
// #if unicode_decoder
const validStringDecoder = paranoid
  ? decoder
  : new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
// #endif
// #if wide_string_scan
const stringWideCharacter = /[\u0100-\uffff]/;
// #endif
let defaultInstance, initialization;
function fail(message) {
  throw new Error(`OCamlJS boundary: ${message}`);
}
class InitializationError extends Error {
  constructor(kind, fields = {}) {
    const message = {
      fetch: `Wasm fetch failed at ${fields.url}`,
      http: `Wasm download failed: HTTP ${fields.status} at ${fields.url}`,
      compile: "Wasm compilation failed",
      instantiate: "Wasm instantiation failed",
      schema: "schema fingerprint mismatch",
      abi: "ABI version mismatch",
      mode: "paranoid mode mismatch: rebuild Rust and generated bindings together",
      poisoned: fields.actual,
      unexpected: "Unexpected Wasm initialization failure",
    }[kind];
    super(`OCamlJS boundary: ${message}`);
    this.kind = kind;
    Object.assign(this, fields);
  }
}
function foreignDetail(value) {
  try {
    if (typeof value?.stack === "string") return value.stack;
    if (typeof value?.message === "string") return value.message;
    return String(value);
  } catch {
    return "Unprintable foreign error";
  }
}
async function initializationStage(kind, action, fields = {}) {
  try {
    return await action();
  } catch (cause) {
    throw new InitializationError(kind, { ...fields, cause });
  }
}
export function default_source() {
  return new URL("./backend.wasm", import.meta.url).href;
}
export function initialize(
  source = new URL("./backend.wasm", import.meta.url),
  { diagnostics = false } = {},
) {
  if (defaultInstance?.poisoned)
    return Promise.reject(
      new InitializationError("poisoned", {
        actual: defaultInstance.failure,
        cause: defaultInstance.failureCause,
      }),
    );
  if (initialization) return initialization;
  const pending = Promise.resolve().then(() =>
    instantiate(source, diagnostics),
  );
  initialization = pending;
  pending.catch(() => {
    if (initialization === pending) initialization = undefined;
  });
  return pending;
}
// #if native_string_input
let nativeCopyModule;
function prepareNativeCopyModule() {
  return (nativeCopyModule ??= WebAssembly.compile(
    new Uint8Array(__NATIVE_STRING_COPY__),
    { builtins: ["js-string"] },
  ).catch(() => null));
}
// #endif
// #if native_string_output
let nativeReadModule;
function prepareNativeReadModule() {
  return (nativeReadModule ??= WebAssembly.compile(
    new Uint8Array(__NATIVE_STRING_READ__),
    { builtins: ["js-string"] },
  ).catch(() => null));
}
// #endif
async function instantiate(source, diagnostics) {
  // #if native_string_input
  const nativeModule = prepareNativeCopyModule();
  // #endif
  // #if native_string_output
  const nativeOutputModule = prepareNativeReadModule();
  // #endif
  let module;
  if (Object.prototype.toString.call(source) === "[object WebAssembly.Module]")
    module = source;
  else if (ArrayBuffer.isView(source))
    module = await initializationStage("compile", () =>
      WebAssembly.compile(
        new Uint8Array(source.buffer, source.byteOffset, source.byteLength),
      ),
    );
  else if (Object.prototype.toString.call(source) === "[object ArrayBuffer]")
    module = await initializationStage("compile", () =>
      WebAssembly.compile(source),
    );
  else {
    const url = String(source);
    const response = await initializationStage("fetch", () => fetch(source), {
      url,
    });
    if (!response.ok)
      throw new InitializationError("http", { url, status: response.status });
    module = await initializationStage("compile", async () =>
      typeof WebAssembly.compileStreaming === "function" &&
      response.headers.get("Content-Type") === "application/wasm"
        ? WebAssembly.compileStreaming(response)
        : WebAssembly.compile(await response.arrayBuffer()),
    );
  }
  const stamps = WebAssembly.Module.customSections(module, "ocamljs.abi");
  if (
    stamps.length !== 1 ||
    stamps[0].byteLength !== stamp.length ||
    !new Uint8Array(stamps[0]).every(
      (byte, index) => byte === stamp.charCodeAt(index),
    )
  )
    throw new InitializationError("schema", {
      expected: stamp,
      actual: stamps.map((s) => new TextDecoder().decode(s)).join(", "),
    });
  let e, value;
  const instance = await initializationStage("instantiate", () =>
    WebAssembly.instantiate(module, {
      // #if references
      ocamljs_host: { __HOST_IMPORTS__ },
      // #endif
      ocamljs: {
        boundary_error: (p, n) =>
          fail(decoder.decode(new Uint8Array(e.memory.buffer, p >>> 0, n))),
        // #if async
        task_wake: (id) => {
          if (value) queueTask(value, id >>> 0);
        },
        // #endif
        // #if sync
        task_wake: () => {},
        // #endif
        // #if host_promises
        host_promise_subscribe: (promise, channel) => {
          const subscription = { active: true };
          value.hostPromises.set(channel >>> 0, subscription);
          const settle = (result, ok) => {
            if (!subscription.active) return;
            subscription.active = false;
            value.hostPromises.delete(channel >>> 0);
            settleHostPromise(value, channel, result, ok);
          };
          observePromise(
            promise,
            (result) => settle(result, true),
            (error) => settle(error, false),
          );
        },
        host_promise_unsubscribe: (channel) => {
          const subscription = value.hostPromises.get(channel >>> 0);
          if (subscription) subscription.active = false;
          value.hostPromises.delete(channel >>> 0);
        },
        // #endif
        // #if host_text
        host_string_length: (text, errorPtr) => {
          try {
            if (typeof text !== "string")
              throw new TypeError("expected host string");
            return text.length;
          } catch (error) {
            e.__ocamljs_host_error(errorPtr, error);
            return 0;
          }
        },
        host_string_copy: (text, ptr, len, errorPtr) => {
          try {
            if (typeof text !== "string")
              throw new TypeError("expected host string");
            const { read, written } = encoder.encodeInto(
              text,
              new Uint8Array(e.memory.buffer, ptr >>> 0, len >>> 0),
            );
            return read === text.length ? written : -1;
          } catch (error) {
            e.__ocamljs_host_error(errorPtr, error);
            return 0;
          }
        },
        // #endif
        // #if host_bytes
        host_bytes_new: (ptr, len, errorPtr) => {
          try {
            return new Uint8ClampedArray(
              new Uint8Array(e.memory.buffer, ptr >>> 0, len >>> 0),
            );
          } catch (error) {
            e.__ocamljs_host_error(errorPtr, error);
            return null;
          }
        },
        host_bytes_prepare: (bytes, limit, lenPtr, errorPtr) => {
          try {
            if (!ArrayBuffer.isView(bytes) || bytes.BYTES_PER_ELEMENT !== 1)
              throw new TypeError("expected byte array");
            // Snapshot a Wasm view before Rust allocation can grow, detach or reuse
            // its backing memory. Ordinary external arrays require no extra copy.
            if (bytes.byteLength > limit >>> 0) {
              new DataView(e.memory.buffer).setUint32(
                lenPtr >>> 0,
                Math.min(bytes.byteLength, 4294967295),
                true,
              );
              return null;
            }
            const source = new Uint8Array(
              bytes.buffer,
              bytes.byteOffset,
              bytes.byteLength,
            );
            if (bytes.buffer === e.memory.buffer)
              bytes = new Uint8ClampedArray(source);
            new DataView(e.memory.buffer).setUint32(
              lenPtr >>> 0,
              bytes.byteLength,
              true,
            );
            return bytes;
          } catch (error) {
            e.__ocamljs_host_error(errorPtr, error);
            return null;
          }
        },
        host_bytes_copy: (bytes, ptr, len, errorPtr) => {
          try {
            if (
              !ArrayBuffer.isView(bytes) ||
              bytes.BYTES_PER_ELEMENT !== 1 ||
              bytes.byteLength !== len >>> 0
            )
              throw new TypeError("byte array changed or detached");
            new Uint8Array(e.memory.buffer, ptr >>> 0, len >>> 0).set(
              new Uint8Array(bytes.buffer, bytes.byteOffset, bytes.byteLength),
            );
          } catch (error) {
            e.__ocamljs_host_error(errorPtr, error);
          }
        },
        // #endif
        // #if callbacks
        callback_call: (handler, output) =>
          invokeCallback(value, handler, output >>> 0),
        // #endif
        // #if fetch
        fetch_open: (id, p, n) => {
          let url;
          try {
            url = decoder.decode(new Uint8Array(e.memory.buffer, p >>> 0, n));
          } catch {
            queueMicrotask(() => endFetch(value, id >>> 0, 0));
            return;
          }
          startFetch(value, id >>> 0, url);
        },
        fetch_credit: (id) => {
          const request = value?.fetches.get(id >>> 0);
          if (request?.creditWaiter) {
            if (value.diagnostics) value.stats.fetchCredits++;
            const resolve = request.creditWaiter;
            request.creditWaiter = null;
            resolve();
          }
        },
        fetch_close: (id) => {
          const request = value?.fetches.get(id >>> 0);
          if (request) {
            request.abort();
            value.fetches.delete(id >>> 0);
          }
        },
        // #endif
      },
    }),
  );
  e = instance.exports;
  if (e.__ocamljs_abi?.() !== 13)
    throw new InitializationError("abi", {
      expected: "13",
      actual: String(e.__ocamljs_abi?.()),
    });
  if (e.__ocamljs_paranoid && Boolean(e.__ocamljs_paranoid()) !== paranoid)
    throw new InitializationError("mode", {
      expected: String(paranoid),
      actual: String(Boolean(e.__ocamljs_paranoid())),
    });
  value = {
    // #if input
    inputWriter: null,
    // #endif
    e,
    diagnostics,
    // #if completion_view
    completionData: null,
    // #endif
    // #if string_memory
    stringBytes: null,
    stringWords: null,
    stringInputLength: 0,
    // #endif
    // #if numeric_input
    numericViews: [],
    // #endif
    // #if host_promises
    hostPromises: new Map(),
    // #endif
    // #if owners
    finalizerPending: new Set(),
    // #endif
    // #if fetch
    fetches: new Map(),
    // #endif
    busy: false,
    // #if async
    jobs: new Map(),
    ready: new Set(),
    timer: null,
    // #endif
    // #if callbacks
    callbackError: null,
    // #endif
    stats: {
      calls: 0,
      inputBytes: 0,
      outputBytes: 0,
      // #if async
      polls: 0,
      wakes: 0,
      // #endif
      // #if fetch
      fetchPushes: 0,
      fetchWaits: 0,
      fetchCredits: 0,
      // #endif
    },
  };
  // #if native_string_input
  value.nativeCopy = null;
  try {
    const compiled = await nativeModule;
    if (compiled)
      value.nativeCopy = (
        await WebAssembly.instantiate(compiled, { env: { memory: e.memory } })
      ).exports.copy;
  } catch {
    /* Unavailable builtins use the SIMD preparation path. */
  }
  // #endif
  // #if native_string_output
  value.nativeRead = null;
  try {
    const compiled = await nativeOutputModule;
    if (compiled)
      value.nativeRead = (
        await WebAssembly.instantiate(compiled, { env: { memory: e.memory } })
      ).exports.read;
  } catch {
    /* Older engines retain the owned byte-string conversion. */
  }
  // #endif
  // #if paranoid
  instances.add(value);
  // #endif
  defaultInstance = value;
  return value;
}
function inst(i) {
  // #if paranoid
  knownInstance(i);
  // #endif
  if (i.poisoned) fail("invalid or poisoned instance");
  return i;
}
// #if paranoid
function knownInstance(i) {
  // #if paranoid
  if (!instances.has(i)) fail("invalid instance");
  // #endif
  return i;
}
// #endif
// #if owners
function handle(i, h, type) {
  // A zero check protects aliases after explicit disposal/transfer.
  // #if paranoid
  if (!(h instanceof Owner) || h.owner !== i || h.type !== type)
    fail("foreign, wrong-type or disposed handle");
  // #endif
  if (!h.ptr) fail("disposed handle");
  return h.ptr;
}
function own(i, id, type) {
  try {
    const value = new Owner(id);
    // #if paranoid
    value.owner = i;
    value.type = type;
    // #endif
    finalizers?.register(value, id, value);
    return value;
  } catch (error) {
    // #if callbacks
    const callbackError = i.callbackError;
    // #endif
    if (!i.poisoned) {
      try {
        freeOwner(i, id);
      } catch (_) {
        /* Preserve construction failure. */
      }
    }
    // #if callbacks
    i.callbackError = callbackError;
    // #endif
    throw error;
  }
}
function freeOwner(i, id) {
  if (i.poisoned) return;
  const busy = i.busy;
  i.busy = true;
  try {
    if (!i.e.__ocamljs_dispose(id)) {
      i.finalizerPending.add(id);
      return;
    }
  } catch (error) {
    poison(i, error);
    throw error;
  } finally {
    i.busy = busy;
  }
}
function finalizeOwner(i, id) {
  if (i.poisoned) return;
  if (i.busy) {
    i.finalizerPending.add(id);
    return;
  }
  i.busy = true;
  try {
    if (!i.e.__ocamljs_dispose(id)) {
      i.finalizerPending.add(id);
      return;
    }
    i.finalizerPending.delete(id);
    // #if callbacks
    if (i.callbackError) {
      globalThis.reportError?.(i.callbackError.value);
      i.callbackError = null;
    }
    // #endif
    return true;
  } catch (error) {
    // A throw can occur after user Drop freed the allocation. Only status0
    // proves retry is safe; never retry an indeterminate pointer.
    i.finalizerPending.delete(id);
    poison(i, error);
    globalThis.reportError?.(error);
  } finally {
    i.busy = false;
  }
}
function flushFinalizers(i) {
  if (i.busy || i.poisoned) return;
  let progress;
  do {
    progress = false;
    for (const id of i.finalizerPending) {
      if (i.poisoned) return;
      progress = finalizeOwner(i, id) || progress;
    }
  } while (progress && i.finalizerPending.size);
}
function dispose(i, h, type, internal = false) {
  const id = handle(inst(i), h, type);
  if (i.busy && !internal) fail("reentrant call");
  const busy = i.busy;
  // #if callbacks
  let callbackFailure;
  // #endif
  i.busy = true;
  // Invalidate aliases before invoking user Drop. A rejected borrow has not
  // touched the value and explicitly returns zero, so only that case restores it.
  h.ptr = 0;
  try {
    if (!i.e.__ocamljs_dispose(id)) {
      h.ptr = id;
      fail("cannot dispose borrowed object");
    }
    i.finalizerPending.delete(id);
    finalizers?.unregister(h);
    // #if callbacks
    if (i.callbackError) {
      callbackFailure = i.callbackError;
      const error = callbackFailure.value;
      i.callbackError = null;
      throw error;
    }
    // #endif
  } catch (error) {
    let trapped = h.ptr === 0;
    // #if callbacks
    trapped &&= !callbackFailure;
    // #endif
    if (trapped) poison(i, error);
    // #if callbacks
    i.callbackError = null;
    // #endif
    throw error;
  } finally {
    if (!h.ptr) finalizers?.unregister(h);
    i.busy = busy;
    flushFinalizers(i);
  }
}
// #endif
// #if writer
class Writer {
  constructor(owner = null) {
    this.owner = owner;
    this.input = undefined;
    this.offset = 0;
    this.capacity = 128;
    this.buffer = new ArrayBuffer(128);
    this.data = new DataView(this.buffer);
    this.byteView = new Uint8Array(this.buffer);
    this.length = 0;
    // #if paranoid
    this.items = MAX_ITEMS;
    this.depth = 0;
    // #endif
  }
  refresh() {
    if (this.input !== undefined && this.buffer.byteLength === 0) {
      this.buffer = this.owner.e.memory.buffer;
      this.data = new DataView(this.buffer, this.offset, this.capacity);
      this.byteView = new Uint8Array(this.buffer, this.offset, this.capacity);
    }
  }
  byteArray() {
    this.refresh();
    return this.byteView;
  }
  view(at, length) {
    this.refresh();
    return new Uint8Array(this.buffer, this.offset + at, length);
  }
  reserve(n) {
    // #if paranoid
    if (!Number.isSafeInteger(n) || n < 0 || this.length + n > MAX_BYTES)
      fail("output size limit");
    // #endif
    this.refresh();
    if (this.length + n > this.capacity) this.grow(this.length + n);
    const at = this.length;
    this.length += n;
    return at;
  }
  grow(required) {
    const capacity = Math.max(required + 128, this.capacity * 2);
    if (this.owner && !paranoid && capacity >= 2048) {
      const i = this.owner,
        previous = this.input,
        oldOffset = this.offset,
        oldBuffer = this.buffer;
      let input;
      try {
        input = i.e.__ocamljs_alloc(capacity);
        const offset = i.e.__ocamljs_input_ptr(input) >>> 0;
        const buffer = i.e.memory.buffer;
        const source =
          previous === undefined
            ? new Uint8Array(oldBuffer, 0, this.length)
            : new Uint8Array(buffer, oldOffset, this.length);
        const data = new DataView(buffer, offset, capacity),
          byteView = new Uint8Array(buffer, offset, capacity);
        byteView.set(source);
        if (previous !== undefined) freeInput(i, previous);
        this.input = input;
        this.offset = offset;
        this.buffer = buffer;
        this.capacity = capacity;
        this.data = data;
        this.byteView = byteView;
      } catch (error) {
        if (error instanceof WebAssembly.RuntimeError) poison(i, error);
        if (input !== undefined) freeInput(i, input);
        throw error;
      }
    } else {
      const next = new ArrayBuffer(capacity);
      new Uint8Array(next).set(this.view(0, this.length));
      this.buffer = next;
      this.capacity = capacity;
      this.data = new DataView(next);
      this.byteView = new Uint8Array(next);
    }
  }
  bytes() {
    return this.view(0, this.length);
  }
  dispose() {
    if (this.input !== undefined) {
      const id = this.input;
      this.input = undefined;
      freeInput(this.owner, id);
    }
  }
}

// #endif
// #if reader
class Reader {
  constructor(owner, id, plain = false) {
    this.owner = owner;
    this.id = id;
    try {
      const info = owner.e.__ocamljs_output_info(id) >>> 0;
      this.buffer = owner.e.memory.buffer;
      this.data = new DataView(this.buffer);
      // #if paranoid
      if (info === 0 || info % 4 !== 0 || info + 16 > this.buffer.byteLength)
        fail("invalid output metadata");
      // #endif
      // Capture every word before any further Rust call: arena growth can move
      // this inline descriptor. Only the segment allocations stay stable.
      this.ptr = this.data.getUint32(info, true);
      this.length = this.data.getUint32(info + 4, true);
      // #if pool_metadata
      this.poolPtr = this.data.getUint32(info + 8, true);
      this.poolLength = this.data.getUint32(info + 12, true);
      // #endif
      // #if no_pool_metadata
      if (owner.diagnostics)
        owner.stats.outputBytes += this.data.getUint32(info + 12, true);
      // #endif
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) poison(owner, error);
      throw error;
    }
    this.position = 0;
    // #if pool_metadata
    this.poolPosition = 0;
    // #endif
    // #if r_pool_string
    this.textPosition = 0;
    this.textBytePosition = 0;
    this.textPoolEnd = 0;
    this.poolText = null;
    this.poolBytes = null;
    // #endif
    // #if paranoid
    this.items = MAX_ITEMS;
    // #endif
    // #if paranoid
    this.depth = 0;
    // #endif
    this.closed = false;
    // #if paranoid
    if (
      !Number.isSafeInteger(this.length) ||
      this.length < 0 ||
      !Number.isSafeInteger(this.poolLength) ||
      this.poolLength < 0 ||
      this.length + this.poolLength > MAX_BYTES
    )
      fail("output size limit");
    if (
      this.ptr + this.length > this.buffer.byteLength ||
      this.poolPtr + this.poolLength > this.buffer.byteLength
    )
      fail("invalid output segment range");
    // #endif
    if (owner.diagnostics)
      owner.stats.outputBytes +=
        this.length +
        // #if pool_metadata
        this.poolLength;
    // #endif
  }
  view() {
    // This output allocation stays live for the Reader's lifetime. Unshared
    // memory growth detaches its old buffer, including grow(0).
    if (!this.buffer || this.buffer.byteLength === 0) {
      this.buffer = this.owner.e.memory.buffer;
      this.data = new DataView(this.buffer);
    }
    return this.data;
  }
  take(n) {
    // #if paranoid
    if (
      this.closed ||
      !Number.isSafeInteger(n) ||
      n < 0 ||
      this.position + n > this.length
    )
      fail("truncated value");
    // #endif
    const at = this.ptr + this.position;
    this.position += n;
    return at;
  }
}
// #endif
// #if w_string
// #if paranoid
function encodeString(s) {
  if (byteStrings) {
    if (typeof s !== "string" || s.length > MAX_BYTES)
      fail("invalid or oversized string");
    const bytes = new Uint8Array(s.length);
    for (let n = 0; n < s.length; n++) {
      const c = s.charCodeAt(n);
      if (c > 255) fail("expected native UTF-8 OCaml string");
      bytes[n] = c;
    }
    return bytes;
  }
  return encodeHostString(s);
}
function encodeHostString(s) {
  if (typeof s !== "string" || s.length > MAX_BYTES)
    fail("invalid or oversized string");
  for (let j = 0; j < s.length; j++) {
    const c = s.charCodeAt(j);
    if (c >= 0xd800 && c <= 0xdbff) {
      const d = s.charCodeAt(++j);
      if (!(d >= 0xdc00 && d <= 0xdfff)) fail("unpaired surrogate");
    } else if (c >= 0xdc00 && c <= 0xdfff) fail("unpaired surrogate");
  }
  return encoder.encode(s);
}
// #endif
// #endif
// #if scalar_helpers
function boolean(v) {
  // #if paranoid
  if (typeof v !== "boolean") fail("invalid bool");
  // #endif
  return +v;
}
function number(v) {
  // #if paranoid
  if (typeof v !== "number") fail("expected float");
  // #endif
  return v;
}
function wideInteger(v, unsigned) {
  const n = BigInt(v);
  // #if paranoid
  if (n < -(1n << 63n) || n >= 1n << 63n) fail("int64 out of range");
  // #endif
  return unsigned ? BigInt.asUintN(64, n) : n;
}
function integer(v, min, max) {
  // #if paranoid
  if (!Number.isInteger(v) || v < min || v > max) fail("integer out of range");
  // #endif
  return v;
}
// #endif
// Cleanup and queued I/O must poison on traps before any later Wasm access.
// #if free_output
function freeOutput(i, id) {
  if (i.poisoned) return;
  const busy = i.busy;
  i.busy = true;
  try {
    i.e.__ocamljs_output_free(id);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  } finally {
    i.busy = busy;
  }
}
// #endif
// #if input
function freeInput(i, id) {
  if (i.poisoned) return;
  const busy = i.busy;
  i.busy = true;
  try {
    i.e.__ocamljs_input_free(id);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  } finally {
    i.busy = busy;
  }
}
// #endif
const littleEndian = new Uint8Array(new Uint16Array([1]).buffer)[0] === 1;
// #if text_writer
function writeShortUtf8(text, out, offset) {
  let at = offset;
  for (let j = 0; j < text.length; j++) {
    let c = text.charCodeAt(j);
    if (c < 128) out[at++] = c;
    else if (c < 2048) {
      out[at++] = 192 | (c >> 6);
      out[at++] = 128 | (c & 63);
    } else {
      if (c >= 0xd800 && c <= 0xdbff && j + 1 < text.length) {
        const low = text.charCodeAt(j + 1);
        if (low >= 0xdc00 && low <= 0xdfff) {
          c = 0x10000 + ((c - 0xd800) << 10) + low - 0xdc00;
          j++;
          out[at++] = 240 | (c >> 18);
          out[at++] = 128 | ((c >> 12) & 63);
          out[at++] = 128 | ((c >> 6) & 63);
          out[at++] = 128 | (c & 63);
          continue;
        }
      }
      if (c >= 0xd800 && c <= 0xdfff) c = 0xfffd;
      out[at++] = 224 | (c >> 12);
      out[at++] = 128 | ((c >> 6) & 63);
      out[at++] = 128 | (c & 63);
    }
  }
  return at - offset;
}
function writeText(w, text, bytes) {
  if (text.length > 256) return writeLargeText(w, text, bytes);
  const header = w.reserve(4),
    at = w.reserve(text.length * (bytes ? 1 : 3));
  let length;
  if (bytes) {
    length = text.length;
    const out = w.byteArray();
    for (let j = 0; j < length; j++) out[at + j] = text.charCodeAt(j);
  } else
    length =
      text.length <= 32
        ? writeShortUtf8(text, w.byteArray(), at)
        : encoder.encodeInto(text, w.view(at, text.length * 3)).written;
  w.length = at + length;
  w.data.setUint32(header, length, true);
}
function writeLargeText(w, text, bytes) {
  const header = w.reserve(4),
    units = text.length;
  let asciiPrefix = !bytes;
  if (asciiPrefix)
    for (let j = 0; j < 32; j++)
      if (text.charCodeAt(j) >= 128) {
        asciiPrefix = false;
        break;
      }
  // A late wide character can exhaust a byte-sized destination and force a
  // second allocation/copy. Use the engine scan already used by root inputs.
  const narrow = asciiPrefix && !stringWideCharacter.test(text);
  const at = w.reserve(units * (bytes ? (w.owner ? 2 : 1) : narrow ? 1 : 3));
  let length;
  if (bytes) {
    length = units;
    if (w.input !== undefined) {
      const i = w.owner,
        p = w.offset + at;
      let high = false;
      if (i.nativeCopy)
        for (let j = 0; j < 64; j++)
          if (text.charCodeAt(j) >= 128) {
            high = true;
            break;
          }
      if (high) i.nativeCopy(text, p);
      else {
        const encoded = encoder.encodeInto(text, w.view(at, units * 2));
        if (encoded.written !== units)
          i.e.__ocamljs_string_compact_latin1(p, encoded.written);
      }
    } else {
      const out = w.view(at, length);
      for (let j = 0; j < length; j++) out[j] = text.charCodeAt(j);
    }
  } else if (narrow) {
    const first = encoder.encodeInto(text, w.view(at, w.capacity - at));
    length = first.written;
    w.length = at + length;
    if (first.read !== units) {
      const remaining = (units - first.read) * 3;
      const tail = w.reserve(remaining);
      length += encoder.encodeInto(
        text.slice(first.read),
        w.view(tail, remaining),
      ).written;
    }
  } else length = encoder.encodeInto(text, w.view(at, units * 3)).written;
  w.length = at + length;
  w.refresh();
  w.data.setUint32(header, length, true);
}

// #endif
// #if typed_reader
function readTyped(r, Ctor, n, get) {
  const size = Ctor.BYTES_PER_ELEMENT,
    p = r.take(n * size),
    data = r.view(),
    out = new Ctor(n);
  if (size === 1 || littleEndian)
    new Uint8Array(out.buffer).set(new Uint8Array(data.buffer, p, n * size));
  else for (let j = 0; j < n; j++) out[j] = data[get](p + j * size, true);
  return out;
}
// #endif
// #if typed_writer
function writeTyped(w, value, set) {
  const size = value.BYTES_PER_ELEMENT,
    length = value.byteLength,
    offset = value.byteOffset,
    buffer = value.buffer,
    inWasm = w.owner && buffer === w.owner.e.memory.buffer,
    at = w.reserve(length);
  const source = new Uint8Array(
    inWasm ? w.owner.e.memory.buffer : buffer,
    offset,
    length,
  );
  if (size === 1 || littleEndian) w.view(at, length).set(source);
  else {
    if (inWasm && buffer.byteLength === 0)
      value = new value.constructor(
        w.owner.e.memory.buffer,
        offset,
        length / size,
      );
    for (let j = 0; j < value.length; j++)
      w.data[set](at + j * size, value[j], true);
  }
}

// #endif
// #if numeric_input
// #if paranoid
function numericValue(kind, value) {
  switch (kind) {
    case 0:
      return integer(value, 0, 255);
    case 1:
      return integer(value, -128, 127);
    case 2:
      return integer(value, 0, 65535);
    case 3:
      return integer(value, -32768, 32767);
    case 4:
    case 8:
      return integer(value, 0, 4294967295);
    case 5:
    case 9:
      return integer(value, -2147483648, 2147483647);
    default:
      return number(value);
  }
}
// #endif
function freeNumericInput(i, pointer, length, kind) {
  if (i.poisoned) return;
  try {
    i.e.__ocamljs_numeric_free(pointer, length, kind);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  }
}
// #endif

// #if string_memory
// #if string_prepare
function stringPreparationFailed(i, error, p, cap) {
  if (error instanceof WebAssembly.RuntimeError) poison(i, error);
  if (p !== undefined) freeStringInput(i, p, cap);
  throw error;
}
// #if unicode_prepare
// Takes ownership of preparation storage, including on failure.
function encodeStringTail(i, text, units, p, cap, prefix) {
  try {
    const next = prefix + (units - prefix) * 3;
    if (cap < next) {
      p = i.e.__ocamljs_string_realloc(p, cap, next) >>> 0;
      cap = next;
    }
    const n =
      prefix +
      encoder.encodeInto(
        text.slice(prefix),
        stringView(i).subarray(p + prefix, p + cap),
      ).written;
    if (cap !== n) {
      p = i.e.__ocamljs_string_realloc(p, cap, n) >>> 0;
      cap = n;
    }
    i.stringInputLength = n;
    return p;
  } catch (error) {
    return stringPreparationFailed(i, error, p, cap);
  }
}
// #endif
// #if native_prepare
function prepareNativeByteString(i, text, units) {
  let p, cap;
  try {
    let high = false;
    if (i.nativeCopy)
      for (let j = 0; j < 64; j++)
        if (text.charCodeAt(j) >= 128) {
          high = true;
          break;
        }
    if (high) {
      cap = units;
      p = i.e.__ocamljs_string_alloc(cap) >>> 0;
      i.nativeCopy(text, p);
      return p;
    }
    cap = units * 2;
    p = i.e.__ocamljs_string_alloc(cap) >>> 0;
    const encoded = encoder.encodeInto(
      text,
      stringView(i).subarray(p, p + cap),
    );
    p = i.e.__ocamljs_string_finish_latin1(p, encoded.written, cap) >>> 0;
    cap = units;
    return p;
  } catch (error) {
    return stringPreparationFailed(i, error, p, cap);
  }
}
// #endif
// #if unicode_prepare
function prepareUnicodeString(i, text, units) {
  if (units > 32) return prepareLongUnicodeString(i, text, units);
  let p,
    cap = units;
  try {
    p = i.e.__ocamljs_string_alloc(cap) >>> 0;
    const bytes = stringView(i);
    let j = 0;
    for (; j < units; j++) {
      const code = text.charCodeAt(j);
      if (code >= 128) break;
      bytes[p + j] = code;
    }
    if (j !== units) {
      const owned = p;
      p = undefined;
      return encodeStringTail(i, text, units, owned, cap, j);
    }
    i.stringInputLength = units;
    return p;
  } catch (error) {
    return stringPreparationFailed(i, error, p, cap);
  }
}
function prepareLongUnicodeString(i, text, units) {
  let p, cap;
  try {
    let prefix = 0;
    for (; prefix < 32; prefix++) if (text.charCodeAt(prefix) >= 128) break;
    if (prefix < 32) {
      cap = prefix + (units - prefix) * 3;
      p = i.e.__ocamljs_string_alloc(cap) >>> 0;
      const bytes = stringView(i);
      for (let j = 0; j < prefix; j++) bytes[p + j] = text.charCodeAt(j);
      const owned = p;
      p = undefined;
      return encodeStringTail(i, text, units, owned, cap, prefix);
    }
    const wide = text.search(stringWideCharacter);
    cap = wide < 0 ? units : units * 3;
    p = i.e.__ocamljs_string_alloc(cap) >>> 0;
    let n;
    if (wide < 0) {
      const first = encoder.encodeInto(
        text,
        stringView(i).subarray(p, p + cap),
      );
      n = first.written;
      if (first.read !== units) {
        const next = n + (units - first.read) * 2;
        p = i.e.__ocamljs_string_realloc(p, cap, next) >>> 0;
        cap = next;
        n += encoder.encodeInto(
          text.slice(first.read),
          stringView(i).subarray(p + n, p + cap),
        ).written;
      }
    } else {
      n = encoder.encodeInto(
        text.slice(0, wide),
        stringView(i).subarray(p, p + cap),
      ).written;
      n += encoder.encodeInto(
        text.slice(wide),
        stringView(i).subarray(p + n, p + cap),
      ).written;
    }
    if (cap !== n) {
      p = i.e.__ocamljs_string_realloc(p, cap, n) >>> 0;
      cap = n;
    }
    i.stringInputLength = n;
    return p;
  } catch (error) {
    return stringPreparationFailed(i, error, p, cap);
  }
}
// #endif
// #endif
// #if string_prefix
const stringNonAscii = /[\x80-\uffff]/;
function stringPrefix(text) {
  const length = text.length,
    end = Math.min(length, 32);
  for (let j = 0; j < end; j++) if (text.charCodeAt(j) >= 128) return j;
  if (length <= end) return length;
  const at = text.search(stringNonAscii);
  return at < 0 ? length : at;
}
// #endif
// #if paranoid
function checkedUnicodeScalars(text) {
  for (let j = 0; j < text.length; j++) {
    const code = text.charCodeAt(j);
    if (code >= 0xd800 && code <= 0xdbff) {
      const low = text.charCodeAt(++j);
      if (!(low >= 0xdc00 && low <= 0xdfff)) fail("unpaired surrogate");
    } else if (code >= 0xdc00 && code <= 0xdfff) fail("unpaired surrogate");
  }
}
// #endif
function stringView(i) {
  let bytes = i.stringBytes;
  if (!bytes || bytes.byteLength === 0) {
    const memory = i.e.memory.buffer;
    bytes = i.stringBytes = new Uint8Array(memory);
    i.stringWords = littleEndian
      ? new Uint32Array(memory)
      : new DataView(memory);
  }
  return bytes;
}
function freeStringInput(i, p, cap) {
  if (i.poisoned) return;
  try {
    i.e.__ocamljs_string_free(p, cap);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  }
}
function discardStringOutput(i, info) {
  stringView(i);
  const words = i.stringWords,
    at = info >>> 2;
  const p = littleEndian ? words[at] : words.getUint32(info, true);
  const cap = littleEndian ? words[at + 2] : words.getUint32(info + 8, true);
  freeStringInput(i, p, cap);
}
// #if string_result
function discardStringResultOutput(i, info) {
  stringView(i);
  const words = i.stringWords,
    at = info >>> 2;
  const tag = littleEndian ? words[at] : words.getUint32(info, true),
    value = littleEndian ? words[at + 1] : words.getUint32(info + 4, true);
  if (tag === 0) discardStringOutput(i, value);
  // #if buffered_string_error
  else if (tag === 1) freeOutput(i, value);
  // #endif
}
function readStringResultOutput(i, info, decode, plain, scalarError) {
  stringView(i);
  const words = i.stringWords,
    at = info >>> 2;
  const tag = littleEndian ? words[at] : words.getUint32(info, true),
    value = littleEndian ? words[at + 1] : words.getUint32(info + 4, true);
  // #if paranoid
  if (tag > 2 || (tag === 2 && !scalarError)) fail("invalid string result tag");
  // #endif
  if (tag === 2) return { scalarError: scalarError(i, value) };
  if (tag === 0) return decode(i, value);
  // #if buffered_string_error
  return readOutput(i, value, plain);
  // #endif
  // #if no_buffered_string_error
  fail("unexpected buffered string error");
  // #endif
}
// #endif
// #if unicode_string_output
function readUnicodeStringOutput(i, info) {
  const bytes = stringView(i),
    words = i.stringWords,
    at = info >>> 2;
  const p = littleEndian ? words[at] : words.getUint32(info, true),
    n = littleEndian ? words[at + 1] : words.getUint32(info + 4, true),
    cap = littleEndian ? words[at + 2] : words.getUint32(info + 8, true),
    kind = littleEndian ? words[at + 3] : words.getUint32(info + 12, true);
  try {
    if (i.diagnostics) i.stats.outputBytes += n;
    if (n === 0) return "";
    if (n <= 16 && shortAscii(bytes, p, p + n))
      return nativeBytes(bytes, p, p + n);
    return (kind === 2 ? validStringDecoder : decoder).decode(
      bytes.subarray(p, p + n),
    );
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  } finally {
    freeStringInput(i, p, cap);
  }
}
// #endif
function readNativeStringOutput(i, info) {
  const bytes = stringView(i);
  const words = i.stringWords,
    at = info >>> 2;
  const p = littleEndian ? words[at] : words.getUint32(info, true),
    n = littleEndian ? words[at + 1] : words.getUint32(info + 4, true),
    cap = littleEndian ? words[at + 2] : words.getUint32(info + 8, true),
    kind = littleEndian ? words[at + 3] : words.getUint32(info + 12, true);
  try {
    if (i.diagnostics) i.stats.outputBytes += n;
    if (n === 0) return "";
    const value = bytes.subarray(p, p + n);
    // #if paranoid
    decoder.decode(value);
    // #endif
    if (n > 512 && kind === 0) return decoder.decode(value);
    // #if native_string_output
    if (n > 512 && i.nativeRead) return i.nativeRead(p, n);
    // #endif
    return nativeBytes(value, 0, n, n > 512);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  } finally {
    freeStringInput(i, p, cap);
  }
}
// #endif

// #if r_pool_string
// #if trusted
function shortUtf8(bytes, start, end) {
  let value = "";
  for (let at = start; at < end;) {
    const first = bytes[at++];
    let point = first;
    if (first >= 240) {
      point =
        ((first & 7) << 18) |
        ((bytes[at++] & 63) << 12) |
        ((bytes[at++] & 63) << 6) |
        (bytes[at++] & 63);
    } else if (first >= 224) {
      point =
        ((first & 15) << 12) | ((bytes[at++] & 63) << 6) | (bytes[at++] & 63);
    } else if (first >= 192) {
      point = ((first & 31) << 6) | (bytes[at++] & 63);
    }
    value += String.fromCodePoint(point);
  }
  return value;
}
function readLargePooledString(r, pool, start, end, unicode) {
  if (!r.owner || r.owner.poisoned)
    return byteStrings && !unicode
      ? nativeBytes(pool, start, end)
      : decoder.decode(pool.subarray(start, end));
  const n = end - start;
  const i = r.owner,
    p = r.poolPtr + start;
  try {
    // The Rust pool allocation stays live through Reader.close, including
    // across claims/memory growth. Classification cannot allocate or move it.
    const kind = i.e.__ocamljs_string_kind(p, n);
    if (byteStrings && !unicode) {
      if (kind === 0) return decoder.decode(pool.subarray(start, end));
      // #if native_string_output
      if (i.nativeRead) return i.nativeRead(p, n);
      // #endif
      return nativeBytes(pool, start, end, true);
    }
    r.poolText = null;
    return (kind === 2 ? validStringDecoder : decoder).decode(
      pool.subarray(start, end),
    );
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  }
}
// #endif
// #endif

const methods = {
  // #if string_result
  string_result_ok: (value) => typeof value === "string",
  string_result_value: (value) => value,
  string_result_error: (value) => value.scalarError,
  string_result_wrap: (value) => ({ tag: 0, ok: value }),
  // #if string_result_js_error
  string_result_wrap_error: (value) => ({ tag: 1, error: value }),
  // #endif
  // #endif
  host_string: hostString,
  native_string: nativeString,
  observe_load: (pending, ok, error) => {
    pending.then((raw) => {
      try {
        ok(raw);
      } catch (cause) {
        error(cause);
      }
    }, error);
  },
  load_ok: (raw) => raw === null,
  load_kind: (raw) => nativeString(raw.kind),
  load_url: (raw) => nativeString(raw.url ?? ""),
  load_status: (raw) => raw.status ?? 0,
  load_expected: (raw) => nativeString(raw.expected ?? ""),
  load_actual: (raw) => nativeString(raw.actual ?? ""),
  load_cause: (raw) => raw.cause,
  foreign_detail: (cause) => nativeString(foreignDetail(cause)),
  module_failure_cause: () => defaultInstance?.failureCause,
  // #if writer
  writer: () => new Writer(),
  // #endif
  module_failed: () => !!defaultInstance?.poisoned,
  module_failure: () => nativeString(defaultInstance?.failure ?? ""),
  load: async (s) => {
    try {
      await initialize(hostString(s));
      return null;
    } catch (cause) {
      return cause instanceof InitializationError
        ? cause
        : new InitializationError("unexpected", { cause });
    }
  },
  // #if r_host
  r_host: (r) => r.owner.e.__ocamljs_host_output(r.id, methods.r_u32(r)),
  // #endif
  // #if host_promises
  on_promise: (p, ok, error) => {
    observePromise(p, ok, error);
  },
  promise_create: (executor) => new Promise(executor),
  // #if jsoo_promise
  promise_from_native: (promise, observe) => {
    promiseObservers.set(promise, observe);
    return promise;
  },
  // #endif
  // #endif
  // #if finish
  finish: (r) => {
    // #if paranoid
    if (r.position !== r.length || r.poolPosition !== r.poolLength)
      fail("trailing output bytes");
    // #endif
    // #if owners
    if (r.claimed) r.committed = true;
    // #endif
  },
  // #endif
  // #if close
  close: (r) => {
    if (r.closed) return;
    r.closed = true;
    // #if r_pool_string
    r.poolText = null;
    r.poolBytes = null;
    // #endif
    let failure;
    try {
      // #if owners
      if (!r.committed && r.claimed)
        for (const h of r.claimed) {
          if (h.ptr)
            try {
              dispose(r.owner, h, h.type, true);
            } catch (error) {
              failure ??= { value: error };
            }
        }
      // #endif
    } finally {
      if (!r.borrowed && !r.owner.poisoned) {
        try {
          freeOutput(r.owner, r.id);
        } catch (error) {
          failure ??= { value: error };
        }
      }
      // #if callbacks
      failure ??= r.owner.callbackError;
      r.owner.callbackError = null;
      // #endif
    }
    if (failure) throw failure.value;
  },
  // #endif
  // #if enter
  // #if paranoid
  enter: (r) => {
    if (++r.depth > MAX_DEPTH) fail("nesting limit");
  },
  // #endif
  // #if trusted
  enter: () => {},
  // #endif
  // #endif
  // #if leave
  // #if paranoid
  leave: (r) => {
    r.depth--;
  },
  // #endif
  // #if trusted
  leave: () => {},
  // #endif
  // #endif
  // #if r_budget
  // #if paranoid
  r_budget: (r, n) => {
    integer(n, 0, r.items);
    r.items -= n;
  },
  // #endif
  // #if trusted
  r_budget: () => {},
  // #endif
  // #endif
  // #if w_budget
  // #if paranoid
  w_budget: (w, n) => {
    integer(n, 0, w.items);
    w.items -= n;
  },
  // #endif
  // #if trusted
  w_budget: () => {},
  // #endif
  // #endif
  // #if r_length
  // #if paranoid
  r_length: (r) => {
    const n = methods.r_u32(r);
    if (n > r.items) fail("collection budget");
    r.items -= n;
    return n;
  },
  // #endif
  // #if trusted
  r_length: (r) => methods.r_u32(r),
  // #endif
  // #endif
  // #if w_length
  // #if paranoid
  w_length: (w, n) => {
    integer(n, 0, w.items);
    w.items -= n;
    methods.w_u32(w, n);
  },
  // #endif
  // #if trusted
  w_length: (w, n) => methods.w_u32(w, n),
  // #endif
  // #endif
  // #if r_handle
  r_handle: (r, type) => {
    inst(r.owner);
    const id = methods.r_u32(r);
    try {
      r.owner.e.__ocamljs_claim(r.id, id);
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) poison(r.owner, error);
      throw error;
    }
    const h = own(r.owner, id, type);
    (r.claimed ??= []).push(h);
    return h;
  },
  // #endif
  // #if r_span
  r_span: (r, n) => {
    const at = r.take(n);
    r.view();
    return at;
  },
  // #endif
  // #if r_string
  r_string: (r) => methods.r_pool_string(r, methods.r_u32(r)),
  // #endif
  // #if r_js_string
  r_js_string: (r) => methods.r_pool_string(r, methods.r_u32(r), true),
  // #endif
  // #if r_pool_string
  // #if paranoid
  r_pool_string: (
    r,
    n,
    // #if js_projection
    unicode = false,
    // #endif
  ) => {
    if (
      r.closed ||
      !Number.isSafeInteger(n) ||
      n < 0 ||
      r.poolPosition + n > r.poolLength
    )
      fail("truncated string pool");
    if (r.poolText === null) {
      // Tiny pools capture each byte once, then validate only those owned
      // values. The scalar decoder enforces the same UTF-8 rules as TextDecoder.
      if (r.poolLength <= 16) {
        const captured = new Array(r.poolLength),
          data = r.view();
        for (let at = 0; at < captured.length; at++)
          captured[at] = data.getUint8(r.poolPtr + at);
        let text = "";
        for (let at = 0; at < captured.length;) {
          const lead = captured[at++];
          let code = lead,
            remaining = 0,
            minimum = 0;
          if (lead >= 0xc2 && lead <= 0xdf) {
            code = lead & 0x1f;
            remaining = 1;
            minimum = 0x80;
          } else if (lead >= 0xe0 && lead <= 0xef) {
            code = lead & 0x0f;
            remaining = 2;
            minimum = 0x800;
          } else if (lead >= 0xf0 && lead <= 0xf4) {
            code = lead & 0x07;
            remaining = 3;
            minimum = 0x10000;
          } else if (lead >= 0x80) {
            throw new TypeError("Invalid UTF-8");
          }
          if (at + remaining > captured.length)
            throw new TypeError("Invalid UTF-8");
          for (let index = 0; index < remaining; index++) {
            const next = captured[at++];
            if ((next & 0xc0) !== 0x80) throw new TypeError("Invalid UTF-8");
            code = (code << 6) | (next & 0x3f);
          }
          if (
            code < minimum ||
            code > 0x10ffff ||
            (code >= 0xd800 && code <= 0xdfff)
          )
            throw new TypeError("Invalid UTF-8");
          text += String.fromCodePoint(code);
        }
        r.poolBytes = captured;
        r.poolText = text;
      }
      if (r.poolText === null) {
        // Larger pools retain an owned snapshot across claim/host calls.
        r.poolBytes = new Uint8Array(
          new Uint8Array(r.view().buffer, r.poolPtr, r.poolLength),
        );
        r.poolText = decoder.decode(r.poolBytes);
      }
    }
    const pool = r.poolBytes;
    // Fatal validation of the snapshot plus aligned sequential boundaries
    // proves each individual string is valid UTF-8 (even across empty strings).
    if (n === 0) return "";
    const start = r.poolPosition,
      end = start + n;
    if (end < pool.length && (pool[end] & 0xc0) === 0x80)
      fail("invalid UTF-8 string boundary");
    r.poolPosition = end;
    // This exact range owns the entire decoded text, so retaining it cannot
    // retain unrelated strings. Equal UTF-8/UTF-16 lengths imply ASCII here.
    if (
      start === 0 &&
      end === r.poolLength &&
      (!byteStrings ||
        // #if js_projection
        unicode ||
        // #endif
        r.poolText.length === r.poolLength)
    ) {
      r.textPosition = r.poolText.length;
      // #if js_projection
      r.textBytePosition = end;
      // #endif
      return r.poolText;
    }
    if (
      byteStrings &&
      // #if js_projection
      !unicode &&
      // #endif
      Array.isArray(pool)
    ) {
      if (start === 0 && end === pool.length)
        return String.fromCharCode.apply(null, pool);
      let value = "";
      for (let at = start; at < end; at++)
        value += String.fromCharCode(pool[at]);
      return value;
    }
    // #if js_projection
    if (!unicode) {
      // #endif
      if (byteStrings) return nativeBytes(pool, start, end);
      // #if js_projection
    }
    // #endif
    let units = 0;
    for (let at = start; at < end; at++) {
      const byte = pool[at];
      if ((byte & 0xc0) !== 0x80) units += byte >= 0xf0 ? 2 : 1;
    }
    // #if js_projection
    if (unicode && byteStrings) {
      // Native byte-string reads leave the UTF-16 cursor untouched. Catch up
      // only when a raw Unicode slice actually needs that cursor.
      for (let at = r.textBytePosition ?? 0; at < start; at++) {
        const byte = pool[at];
        if ((byte & 0xc0) !== 0x80) r.textPosition += byte >= 0xf0 ? 2 : 1;
      }
    }
    // #endif
    // Explicit bounded code-unit copies prevent a retained short string from
    // keeping the whole decoded pool alive through substring backing storage.
    const text = r.poolText,
      textStart = r.textPosition;
    r.textPosition += units;
    // #if js_projection
    r.textBytePosition = end;
    // Raw projections intentionally allow retaining the owned decoded pool.
    if (unicode) return text.slice(textStart, textStart + units);
    // #endif
    let value = "";
    const chunk = new Uint16Array(Math.min(units, 4096));
    for (let at = 0; at < units; at += chunk.length) {
      const count = Math.min(chunk.length, units - at);
      for (let index = 0; index < count; index++)
        chunk[index] = text.charCodeAt(textStart + at + index);
      value += String.fromCharCode.apply(
        null,
        count === chunk.length ? chunk : chunk.subarray(0, count),
      );
    }
    return value;
  },
  // #endif
  // #if trusted
  r_pool_string: (r, n, unicode = false) => {
    const start = r.poolPosition,
      end = start + n;
    r.poolPosition = end;
    if (n === 0) return "";
    // Cache a borrowed view, not a byte copy. Claims can grow unshared Wasm
    // memory, detaching the previous view; refresh it before the next field.
    if (r.poolBytes === null || r.poolBytes.byteLength === 0)
      r.poolBytes = new Uint8Array(r.view().buffer, r.poolPtr, r.poolLength);
    const pool = r.poolBytes;
    if (n > 256) return readLargePooledString(r, pool, start, end, unicode);
    if (byteStrings && !unicode) return nativeBytes(pool, start, end);
    if (n <= 16 && shortAscii(pool, start, end))
      return nativeBytes(pool, start, end);
    // A few trailing fields cannot amortize another decoder/cache allocation.
    // Earlier fields in large label pools retain the bounded shared decoder.
    if (n <= 16 && pool.length - start <= 64)
      return shortUtf8(pool, start, end);
    if (r.poolText === null || end > r.textPoolEnd) {
      let limit = Math.min(pool.length, start + 16384);
      while (limit < pool.length && (pool[limit] & 192) === 128) limit--;
      r.poolText = decoder.decode(pool.subarray(start, limit));
      r.textPoolEnd = limit;
      r.textPosition = 0;
      r.textBytePosition = start;
    }
    let units = 0;
    for (let at = r.textBytePosition; at < start; at++) {
      const byte = pool[at];
      if ((byte & 192) !== 128) r.textPosition += byte >= 240 ? 2 : 1;
    }
    for (let at = start; at < end; at++) {
      const byte = pool[at];
      if ((byte & 192) !== 128) units += byte >= 240 ? 2 : 1;
    }
    const at = r.textPosition;
    r.textPosition += units;
    r.textBytePosition = end;
    return r.poolText.slice(at, at + units);
  },
  // #endif
  // #endif
  // #if w_string
  // #if paranoid
  w_string: (w, s) => {
    const b = encodeString(s);
    methods.w_u32(w, b.length);
    const p = w.reserve(b.length);
    w.view(p, b.length).set(b);
  },
  // #endif
  // #if trusted
  w_string: (w, s) => writeText(w, s, byteStrings),
  // #endif
  // #endif
  // #if w_js_string
  // #if paranoid
  w_js_string: (w, value) => {
    const bytes = encodeHostString(value);
    methods.w_u32(w, bytes.length);
    const at = w.reserve(bytes.length);
    w.view(at, bytes.length).set(bytes);
  },
  // #endif
  // #if trusted
  w_js_string: (w, value) => writeText(w, value, false),
  // #endif
  // #endif
  // #if r_bool
  r_bool: (r) => {
    const b = methods.r_u8(r);
    // #if paranoid
    if (b > 1) fail("invalid bool");
    // #endif
    return b === 1;
  },
  // #endif
  // #if w_bool
  w_bool: (w, b) => {
    // #if paranoid
    if (typeof b !== "boolean") fail("invalid bool");
    // #endif
    methods.w_u8(w, +b);
  },
  // #endif
};
__NUMERIC_CODECS__;

// #if host_promises
// Mark typed Promise inputs/import results handled immediately, even when Rust
// defers awaiting or drops them. Preserve identity and the original rejection.
const ignoreHostRejection = () => {};
function observeHostPromise(value) {
  Promise.resolve(value).then(undefined, ignoreHostRejection);
  return value;
}
function settleHostPromise(i, channel, result, ok) {
  if (!i || i.poisoned) return;
  i.busy = true;
  try {
    i.e.__ocamljs_host_settle(channel, result, +ok);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    else throw error;
  } finally {
    i.busy = false;
    // #if owners
    flushFinalizers(i);
    // #endif
  }
}
// #endif

// #if input
function encodeInput(i, encode) {
  // #if trusted
  const w = i.inputWriter ?? new Writer(i);
  i.inputWriter = null;
  w.length = 0;
  // #endif
  // #if paranoid
  const w = new Writer(i);
  // #endif
  try {
    encode(w);
    if (w.input === undefined) return copyInput(i, w.bytes());
    i.e.__ocamljs_input_truncate(w.input, w.length);
    const id = w.input;
    w.input = undefined;
    if (i.diagnostics) i.stats.inputBytes += w.length;
    return id;
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  } finally {
    w.dispose();
    // #if trusted
    // Cache only small JS storage. A transferred Wasm range is never reused.
    if (w.offset === 0 && w.capacity <= 2048) i.inputWriter ??= w;
    // #endif
  }
}
function copyInput(i, bytes) {
  const length = bytes.byteLength,
    offset = bytes.byteOffset;
  const inWasm = bytes.buffer === i.e.memory.buffer;
  // #if paranoid
  if (length > MAX_BYTES) fail("input size limit");
  // #endif
  const id = i.e.__ocamljs_alloc(length);
  try {
    // Allocation may grow memory and detach the caller's Wasm-backed view.
    const source = inWasm
      ? new Uint8Array(i.e.memory.buffer, offset, length)
      : bytes;
    new Uint8Array(
      i.e.memory.buffer,
      i.e.__ocamljs_input_ptr(id) >>> 0,
      length,
    ).set(source);
    if (i.diagnostics) i.stats.inputBytes += length;
    return id;
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    freeInput(i, id);
    throw error;
  }
}
// #endif
// #if reader
function readOutput(i, id, plain = false) {
  try {
    return new Reader(i, id, plain);
  } catch (error) {
    freeOutput(i, id);
    throw error;
  }
}
// #endif
function poison(i, error) {
  if (i.poisoned) return;
  i.failureCause = error;
  i.failure ??= foreignDetail(error);
  i.poisoned = true;
  // #if references
  const references = i.e.__ocamljs_host_references;
  for (let slot = 0; slot < references.length; slot++)
    references.set(slot, null);
  // #endif
  // #if async
  for (const j of i.jobs.values()) {
    if (j.status !== "pending") continue;
    j.status = "failed";
    j.error = foreignDetail(error);
    j.nativeError = false;
    notify(j);
  }
  i.jobs.clear();
  i.ready.clear();
  if (i.timer !== null) cancelTurn(i.timer);
  i.timer = null;
  // #endif
  // #if fetch
  for (const controller of i.fetches?.values() ?? []) controller.abort();
  i.fetches?.clear();
  // #endif
}

// #if completion_view
function completionView(i) {
  let view = i.completionData;
  if (!view || view.buffer.byteLength === 0)
    view = i.completionData = new DataView(i.e.memory.buffer);
  return view;
}
// #endif
// #if async_host
function discardHost(i, id) {
  if (!i.poisoned) i.e.__ocamljs_host_drop(id);
}
// #endif
// #if async_opaque
function completeOpaque(i, id, decode, j, type) {
  const h = own(i, id, type);
  try {
    const value = decode(h);
    // #if delivery_owners
    j.handles = [h];
    // #endif
    return value;
  } catch (error) {
    if (h.ptr) dispose(i, h, type, true);
    throw error;
  }
}
// #endif
// #if async_string_result
function completeStringResult(i, output, decode, j, read, plain, scalarError) {
  const value = readStringResultOutput(i, output, read, plain, scalarError);
  // #if buffered_string_error
  if (typeof value === "string" || "scalarError" in value) return decode(value);
  try {
    return decode(value);
  } finally {
    // #if delivery_owners
    j.handles = value.claimed;
    // #endif
    methods.close(value);
  }
  // #endif
  // #if no_buffered_string_error
  return decode(value);
  // #endif
}
// #endif
// #if async
function taskInfo(token) {
  // #if paranoid
  if (!tasks.has(token)) fail("foreign task");
  // #endif
  return token;
}
function createTask(i, id, plain, lower, discard) {
  const j = {
    i,
    id,
    status: "pending",
    decode: null,
    receiver: null,
    receiverAttached: false,
    // #if async_direct
    lower,
    discard,
    // #endif
    // #if async_owners
    plain,
    // #endif
    // #if delivery_owners
    handles: null,
    // #endif
  };
  // #if paranoid
  tasks.add(j);
  // #endif
  i.jobs.set(id, j);
  queueTask(i, id);
  return j;
}
let turnChannel,
  nextTurn = 1;
const pendingTurns = new Map();
function scheduleTurn(callback) {
  if (typeof setImmediate === "function")
    return { node: setImmediate(callback) };
  if (!turnChannel) {
    turnChannel = new MessageChannel();
    turnChannel.port1.onmessage = (event) => {
      const fn = pendingTurns.get(event.data);
      pendingTurns.delete(event.data);
      fn?.();
    };
  }
  if (nextTurn > Number.MAX_SAFE_INTEGER) fail("scheduler identity exhausted");
  const id = nextTurn++;
  pendingTurns.set(id, callback);
  turnChannel.port2.postMessage(id);
  return { id };
}
function cancelTurn(token) {
  if (!token) return;
  if (token.node) clearImmediate(token.node);
  else pendingTurns.delete(token.id);
}
function queueTask(i, id) {
  if (i.poisoned) return;
  if (i.diagnostics) i.stats.wakes++;
  i.ready.add(id);
  if (i.timer === null) i.timer = scheduleTurn(() => drainTasks(i));
}
function drainTasks(i) {
  i.timer = null;
  if (i.poisoned) return;
  let budget = 16;
  for (const id of [...i.ready].slice(0, 16)) {
    if (!budget--) break;
    i.ready.delete(id);
    const j = i.jobs.get(id);
    if (!j || j.status !== "pending") continue;
    if (i.busy) {
      queueTask(i, id);
      break;
    }
    let output = 0,
      r;
    // #if callbacks
    let callbackFailure;
    // #endif
    try {
      i.busy = true;
      try {
        if (i.diagnostics) i.stats.polls++;
        output = i.e.__ocamljs_task_poll(id) >>> 0;
      } finally {
        i.busy = false;
      }
      // #if callbacks
      if (i.callbackError) {
        callbackFailure = i.callbackError;
        const error = i.callbackError.value;
        i.callbackError = null;
        if (output) {
          // #if async_direct
          if (j.lower) j.discard?.(i, output);
          else
            // #endif
            freeOutput(i, output);
        } else {
          i.busy = true;
          try {
            i.e.__ocamljs_task_cancel(id);
          } finally {
            i.busy = false;
          }
        }
        j.status = "failed";
        j.error = error;
        j.nativeError = true;
        i.callbackError = null;
        i.jobs.delete(id);
        notify(j);
        continue;
      }
      // #endif
      if (!output) continue;
      // #if async_direct
      if (j.lower) {
        try {
          j.value = j.lower(i, output, j.decode, j);
          j.status = "finished";
        } catch (error) {
          if (error instanceof WebAssembly.RuntimeError) {
            poison(i, error);
            throw error;
          }
          j.error = error;
          j.nativeError = true;
          j.status = "failed";
        }
      } else {
        // #endif
        // #if reader
        try {
          // #if async_owners
          r = new Reader(i, output, j.plain);
          // #endif
          // #if no_async_owners
          r = new Reader(i, output, true);
          // #endif
        } catch (error) {
          freeOutput(i, output);
          throw error;
        }
        try {
          j.value = j.decode(r);
          j.status = "finished";
        } catch (error) {
          j.error = error;
          j.nativeError = true;
          j.status = "failed";
        } finally {
          // #if delivery_owners
          if (!j.plain) j.handles = r.claimed;
          // #endif
          methods.close(r);
        }
        // #endif
        // #if async_direct
      }
      // #endif
      notify(j);
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) {
        poison(i, error);
        break;
      }
      // #if callbacks
      const callback = callbackFailure ?? i.callbackError;
      i.callbackError = null;
      j.error = callback ? callback.value : String(error);
      j.nativeError = !!callback;
      // #endif
      // #if no_callbacks
      j.error = foreignDetail(error);
      j.nativeError = false;
      // #endif
      j.status = "failed";
      i.jobs.delete(id);
      notify(j);
    } finally {
      // #if owners
      flushFinalizers(i);
      // #endif
    }
  }
  if (i.ready.size && i.timer === null)
    i.timer = scheduleTurn(() => drainTasks(i));
}
// #if delivery_ack
function discardUndelivered(j) {
  let failure;
  // #if delivery_owners
  const owned = j.handles;
  j.handles = null;
  for (const h of owned ?? []) {
    if (h.ptr) {
      if (j.i.poisoned) h.ptr = 0;
      else
        try {
          dispose(j.i, h, h.type);
        } catch (error) {
          failure ??= { value: error };
        }
    }
  }
  // #endif
  j.value = undefined;
  j.error = "task result abandoned";
  j.nativeError = false;
  j.status = "released";
  j.i.jobs.delete(j.id);
  return failure;
}
function notify(j) {
  if (j.status === "pending" || !j.receiver) return;
  const [
    ok,
    // #if lwt
    cancelled,
    // #endif
    nativeError,
    boundaryError,
  ] = j.receiver;
  j.receiver = null;
  j.i.jobs.delete(j.id);
  queueMicrotask(() => {
    let accepted = false,
      failure;
    try {
      const status = j.status;
      if (status === "finished") accepted = !!ok(j.value);
      if (status === "failed")
        accepted = !!(j.nativeError
          ? nativeError(j.error)
          : boundaryError(nativeString(String(j.error))));
      // #if lwt
      if (status === "cancelled") accepted = !!cancelled(0);
      // #endif
    } catch (error) {
      failure = { error };
    }
    if (!accepted) {
      const cleanup = discardUndelivered(j);
      if (cleanup && !failure) failure = { error: cleanup.value };
    }
    if (failure) throw failure.error;
  });
}
// #endif
// #if direct_promise
function notify(j) {
  if (j.status === "pending" || !j.receiver) return;
  const [resolve, reject] = j.receiver;
  j.receiver = null;
  j.i.jobs.delete(j.id);
  queueMicrotask(() => {
    if (j.status === "finished") resolve(j.value);
    else reject(j.nativeError ? j.error : new Error(String(j.error)));
  });
}
// #endif
methods.task_map = (token, decode) => {
  const j = taskInfo(token);
  if (j.decode) fail("task decoder already attached");
  j.decode = decode;
  return token;
};
// Each Rust future has exactly one native Lwt/Promise receiver.
// #if delivery_ack
methods.task_receive = (
  token,
  ok,
  // #if lwt
  cancelled,
  // #endif
  nativeError,
  boundaryError,
) => {
  const j = taskInfo(token);
  if (j.receiverAttached) fail("task receiver already attached");
  j.receiverAttached = true;
  j.receiver = [
    ok,
    // #if lwt
    cancelled,
    // #endif
    nativeError,
    boundaryError,
  ];
  notify(j);
};
// #endif
// #if promise
// #if delivery_ack
methods.task_await = (token) =>
  new Promise((resolve, reject) => {
    methods.task_receive(
      token,
      (value) => {
        resolve(value);
        return true;
      },
      (error) => {
        reject(error);
        return true;
      },
      (reason) => {
        reject(new Error(hostString(reason)));
        return true;
      },
    );
  });
// #endif
// #if direct_promise
methods.task_await = (token) => {
  const j = taskInfo(token);
  if (j.receiverAttached) fail("task receiver already attached");
  j.receiverAttached = true;
  return new Promise((resolve, reject) => {
    j.receiver = [resolve, reject];
    notify(j);
  });
};
// #endif
// #endif
// #if lwt
methods.task_cancel_safe = (token) => {
  const j = taskInfo(token);
  if (j.status !== "pending" || j.i.poisoned) return;
  if (j.i.busy) {
    queueMicrotask(() => methods.task_cancel_safe(token));
    return;
  }
  try {
    methods.task_cancel(token);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(j.i, error);
    else {
      j.status = "failed";
      j.error = error;
      j.nativeError = true;
      j.i.jobs.delete(j.id);
      notify(j);
    }
  }
};
methods.task_cancel = (token) => {
  const j = taskInfo(token);
  inst(j.i);
  if (j.status !== "pending") return false;
  if (j.i.busy) fail("reentrant call");
  let accepted;
  j.i.busy = true;
  try {
    accepted = j.i.e.__ocamljs_task_cancel(j.id);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(j.i, error);
    throw error;
  } finally {
    j.i.busy = false;
  }
  if (!accepted) return false;
  j.status = "cancelled";
  j.i.ready.delete(j.id);
  j.i.jobs.delete(j.id);
  // #if callbacks
  if (j.i.callbackError) {
    j.error = j.i.callbackError.value;
    j.i.callbackError = null;
    j.nativeError = true;
    j.status = "failed";
  }
  // #endif
  // #if owners
  flushFinalizers(j.i);
  // #endif
  notify(j);
  return true;
};
// #endif
// #endif
// #if callbacks
function invokeCallback(i, handler, output) {
  let r;
  try {
    r = new Reader(i, output);
    r.borrowed = true;
    handler(r);
    return 0;
  } catch (error) {
    i.callbackError ??= { value: error };
    return 1;
  } finally {
    if (r)
      try {
        methods.close(r);
      } catch (error) {
        i.callbackError ??= { value: error };
      }
  }
}
// #endif

// #if fetch
function endFetch(i, id, status) {
  if (!i || i.poisoned) return;
  const busy = i.busy;
  i.busy = true;
  try {
    i.e.__ocamljs_fetch_end(id, status);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    else throw error;
  } finally {
    i.busy = busy;
  }
}
async function startFetch(i, id, url) {
  const controller = new AbortController();
  i.fetches.set(id, controller);
  let reader;
  try {
    const response = await fetch(url, { signal: controller.signal });
    if (!response.ok || !response.body) throw new Error("fetch failed");
    reader = response.body.getReader();
    const push = async (value) => {
      if (!value.byteLength || i.poisoned || i.fetches.get(id) !== controller)
        return;
      const bytes = value;
      let buffer;
      try {
        buffer = i.e.__ocamljs_alloc(bytes.length);
        new Uint8Array(
          i.e.memory.buffer,
          i.e.__ocamljs_input_ptr(buffer) >>> 0,
          bytes.length,
        ).set(bytes);
        while (i.fetches.get(id) === controller && !i.poisoned) {
          const status = i.e.__ocamljs_fetch_push(id, buffer);
          if (status === 1) {
            if (i.diagnostics) i.stats.fetchPushes++;
            return;
          }
          if (status === 0) throw new Error("fetch stream closed");
          if (i.diagnostics) i.stats.fetchWaits++;
          await new Promise((resolve) => {
            const resume = () => {
              controller.signal.removeEventListener("abort", resume);
              resolve();
            };
            controller.creditWaiter = resume;
            controller.signal.addEventListener("abort", resume, { once: true });
          });
        }
      } catch (error) {
        if (error instanceof WebAssembly.RuntimeError) poison(i, error);
        throw error;
      } finally {
        if (buffer !== undefined) freeInput(i, buffer);
      }
    };
    while (i.fetches.get(id) === controller) {
      const { done, value } = await reader.read();
      if (done) break;
      for (let at = 0; at < value.length; at += 16384)
        await push(value.subarray(at, at + 16384));
    }
    if (i.fetches.get(id) === controller) endFetch(i, id, 1);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    if (i.fetches.get(id) === controller) endFetch(i, id, 0);
  } finally {
    try {
      await reader?.cancel();
    } catch {}
    if (i.fetches.get(id) === controller) i.fetches.delete(id);
  }
}
// #endif
__FAST_WRAPPERS__;
export const api = Object.freeze(methods);
if (Object.prototype.hasOwnProperty.call(globalThis, "__GLOBAL__"))
  fail("runtime already registered; use the existing module");
globalThis.__GLOBAL__ = api;
