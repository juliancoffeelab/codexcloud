// First-principles OCamlJS ABI runtime. Generated native codecs own OCaml values.
const stamp = "13:8db263eac5a737072fd045c840cffb8872836f7287e8a878ed784cb4ac4a31c5";
const byteStrings = true;
const paranoid = true;
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
const MAX_BYTES = 64 * 1024 * 1024,
  MAX_ITEMS = 1000000,
  MAX_DEPTH = 128;
const instances = new WeakSet();
class Owner {
  constructor(ptr) {
    this.ptr = ptr;
  }
}
const tasks = new WeakSet();
const finalizers =
  typeof FinalizationRegistry === "function"
    ? new FinalizationRegistry((id) => {
        finalizeOwner(defaultInstance, id);
        flushFinalizers(defaultInstance);
      })
    : null;
const encoder = new TextEncoder(),
  decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const validStringDecoder = paranoid
  ? decoder
  : new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const stringWideCharacter = /[\u0100-\uffff]/;
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
async function instantiate(source, diagnostics) {
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
      ocamljs: {
        boundary_error: (p, n) =>
          fail(decoder.decode(new Uint8Array(e.memory.buffer, p >>> 0, n))),
        task_wake: (id) => {
          if (value) queueTask(value, id >>> 0);
        },
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
    inputWriter: null,
    e,
    diagnostics,
    stringBytes: null,
    stringWords: null,
    stringInputLength: 0,
    numericViews: [],
    finalizerPending: new Set(),
    fetches: new Map(),
    busy: false,
    jobs: new Map(),
    ready: new Set(),
    timer: null,
    stats: {
      calls: 0,
      inputBytes: 0,
      outputBytes: 0,
      polls: 0,
      wakes: 0,
      fetchPushes: 0,
      fetchWaits: 0,
      fetchCredits: 0,
    },
  };
  instances.add(value);
  defaultInstance = value;
  return value;
}
function inst(i) {
  knownInstance(i);
  if (i.poisoned) fail("invalid or poisoned instance");
  return i;
}
function knownInstance(i) {
  if (!instances.has(i)) fail("invalid instance");
  return i;
}
function handle(i, h, type) {
  // A zero check protects aliases after explicit disposal/transfer.
  if (!(h instanceof Owner) || h.owner !== i || h.type !== type)
    fail("foreign, wrong-type or disposed handle");
  if (!h.ptr) fail("disposed handle");
  return h.ptr;
}
function own(i, id, type) {
  try {
    const value = new Owner(id);
    value.owner = i;
    value.type = type;
    finalizers?.register(value, id, value);
    return value;
  } catch (error) {
    if (!i.poisoned) {
      try {
        freeOwner(i, id);
      } catch (_) {
        /* Preserve construction failure. */
      }
    }
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
  } catch (error) {
    let trapped = h.ptr === 0;
    if (trapped) poison(i, error);
    throw error;
  } finally {
    if (!h.ptr) finalizers?.unregister(h);
    i.busy = busy;
    flushFinalizers(i);
  }
}
class Reader {
  constructor(owner, id, plain = false) {
    this.owner = owner;
    this.id = id;
    try {
      const info = owner.e.__ocamljs_output_info(id) >>> 0;
      this.buffer = owner.e.memory.buffer;
      this.data = new DataView(this.buffer);
      if (info === 0 || info % 4 !== 0 || info + 16 > this.buffer.byteLength)
        fail("invalid output metadata");
      // Capture every word before any further Rust call: arena growth can move
      // this inline descriptor. Only the segment allocations stay stable.
      this.ptr = this.data.getUint32(info, true);
      this.length = this.data.getUint32(info + 4, true);
      this.poolPtr = this.data.getUint32(info + 8, true);
      this.poolLength = this.data.getUint32(info + 12, true);
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) poison(owner, error);
      throw error;
    }
    this.position = 0;
    this.poolPosition = 0;
    this.textPosition = 0;
    this.textBytePosition = 0;
    this.textPoolEnd = 0;
    this.poolText = null;
    this.poolBytes = null;
    this.items = MAX_ITEMS;
    this.depth = 0;
    this.closed = false;
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
    if (owner.diagnostics)
      owner.stats.outputBytes +=
        this.length +
        this.poolLength;
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
    if (
      this.closed ||
      !Number.isSafeInteger(n) ||
      n < 0 ||
      this.position + n > this.length
    )
      fail("truncated value");
    const at = this.ptr + this.position;
    this.position += n;
    return at;
  }
}
function boolean(v) {
  if (typeof v !== "boolean") fail("invalid bool");
  return +v;
}
function number(v) {
  if (typeof v !== "number") fail("expected float");
  return v;
}
function wideInteger(v, unsigned) {
  const n = BigInt(v);
  if (n < -(1n << 63n) || n >= 1n << 63n) fail("int64 out of range");
  return unsigned ? BigInt.asUintN(64, n) : n;
}
function integer(v, min, max) {
  if (!Number.isInteger(v) || v < min || v > max) fail("integer out of range");
  return v;
}
// Cleanup and queued I/O must poison on traps before any later Wasm access.
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
const littleEndian = new Uint8Array(new Uint16Array([1]).buffer)[0] === 1;
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
function freeNumericInput(i, pointer, length, kind) {
  if (i.poisoned) return;
  try {
    i.e.__ocamljs_numeric_free(pointer, length, kind);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  }
}

function checkedUnicodeScalars(text) {
  for (let j = 0; j < text.length; j++) {
    const code = text.charCodeAt(j);
    if (code >= 0xd800 && code <= 0xdbff) {
      const low = text.charCodeAt(++j);
      if (!(low >= 0xdc00 && low <= 0xdfff)) fail("unpaired surrogate");
    } else if (code >= 0xdc00 && code <= 0xdfff) fail("unpaired surrogate");
  }
}
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
    decoder.decode(value);
    if (n > 512 && kind === 0) return decoder.decode(value);
    return nativeBytes(value, 0, n, n > 512);
  } catch (error) {
    if (error instanceof WebAssembly.RuntimeError) poison(i, error);
    throw error;
  } finally {
    freeStringInput(i, p, cap);
  }
}


const methods = {
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
  finish: (r) => {
    if (r.position !== r.length || r.poolPosition !== r.poolLength)
      fail("trailing output bytes");
    if (r.claimed) r.committed = true;
  },
  close: (r) => {
    if (r.closed) return;
    r.closed = true;
    r.poolText = null;
    r.poolBytes = null;
    let failure;
    try {
      if (!r.committed && r.claimed)
        for (const h of r.claimed) {
          if (h.ptr)
            try {
              dispose(r.owner, h, h.type, true);
            } catch (error) {
              failure ??= { value: error };
            }
        }
    } finally {
      if (!r.borrowed && !r.owner.poisoned) {
        try {
          freeOutput(r.owner, r.id);
        } catch (error) {
          failure ??= { value: error };
        }
      }
    }
    if (failure) throw failure.value;
  },
  enter: (r) => {
    if (++r.depth > MAX_DEPTH) fail("nesting limit");
  },
  leave: (r) => {
    r.depth--;
  },
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
  r_span: (r, n) => {
    const at = r.take(n);
    r.view();
    return at;
  },
  r_string: (r) => methods.r_pool_string(r, methods.r_u32(r)),
  r_pool_string: (
    r,
    n,
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
        r.poolText.length === r.poolLength)
    ) {
      r.textPosition = r.poolText.length;
      return r.poolText;
    }
    if (
      byteStrings &&
      Array.isArray(pool)
    ) {
      if (start === 0 && end === pool.length)
        return String.fromCharCode.apply(null, pool);
      let value = "";
      for (let at = start; at < end; at++)
        value += String.fromCharCode(pool[at]);
      return value;
    }
      if (byteStrings) return nativeBytes(pool, start, end);
    let units = 0;
    for (let at = start; at < end; at++) {
      const byte = pool[at];
      if ((byte & 0xc0) !== 0x80) units += byte >= 0xf0 ? 2 : 1;
    }
    // Explicit bounded code-unit copies prevent a retained short string from
    // keeping the whole decoded pool alive through substring backing storage.
    const text = r.poolText,
      textStart = r.textPosition;
    r.textPosition += units;
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
};
methods.r_u8=r=>{const p=r.take(1);return r.view().getUint8(p,true);};
methods.r_u32=r=>{const p=r.take(4);return r.view().getUint32(p,true);};
methods.numeric_set_0=(view,index,value)=>{numericValue(0,value);if(littleEndian||1===1)view[index]=value;else view.setUint8(index*1,value,true);};
;


function encodeInput(i, encode) {
  const w = new Writer(i);
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
  }
}
function copyInput(i, bytes) {
  const length = bytes.byteLength,
    offset = bytes.byteOffset;
  const inWasm = bytes.buffer === i.e.memory.buffer;
  if (length > MAX_BYTES) fail("input size limit");
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
function readOutput(i, id, plain = false) {
  try {
    return new Reader(i, id, plain);
  } catch (error) {
    freeOutput(i, id);
    throw error;
  }
}
function poison(i, error) {
  if (i.poisoned) return;
  i.failureCause = error;
  i.failure ??= foreignDetail(error);
  i.poisoned = true;
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
  for (const controller of i.fetches?.values() ?? []) controller.abort();
  i.fetches?.clear();
}

function taskInfo(token) {
  if (!tasks.has(token)) fail("foreign task");
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
  };
  tasks.add(j);
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
    try {
      i.busy = true;
      try {
        if (i.diagnostics) i.stats.polls++;
        output = i.e.__ocamljs_task_poll(id) >>> 0;
      } finally {
        i.busy = false;
      }
      if (!output) continue;
        try {
          r = new Reader(i, output, true);
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
          methods.close(r);
        }
      notify(j);
    } catch (error) {
      if (error instanceof WebAssembly.RuntimeError) {
        poison(i, error);
        break;
      }
      j.error = foreignDetail(error);
      j.nativeError = false;
      j.status = "failed";
      i.jobs.delete(id);
      notify(j);
    } finally {
      flushFinalizers(i);
    }
  }
  if (i.ready.size && i.timer === null)
    i.timer = scheduleTurn(() => drainTasks(i));
}
function discardUndelivered(j) {
  let failure;
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
methods.task_map = (token, decode) => {
  const j = taskInfo(token);
  if (j.decode) fail("task decoder already attached");
  j.decode = decode;
  return token;
};
// Each Rust future has exactly one native Lwt/Promise receiver.
methods.task_receive = (
  token,
  ok,
  nativeError,
  boundaryError,
) => {
  const j = taskInfo(token);
  if (j.receiverAttached) fail("task receiver already attached");
  j.receiverAttached = true;
  j.receiver = [
    ok,
    nativeError,
    boundaryError,
  ];
  notify(j);
};
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
methods.r_handle_token=r=>methods.r_handle(r,"Token");
methods.dispose_Token=h=>dispose(defaultInstance,h,"Token");
methods.call___ocamljs_Geometry_distance=function(a0,a1,a2,a3){
const i=inst(defaultInstance);if(i.poisoned)fail('invalid or poisoned instance');if(i.busy)fail('reentrant call');
i.busy=true;
try{
const v0=number(a0);
const v1=number(a1);
const v2=number(a2);
const v3=number(a3);

if(i.diagnostics)i.stats.calls++;
const r=i.e.__ocamljs_fn___ocamljs_Geometry_distance(v0,v1,v2,v3);
return r;
}catch(error){if(error instanceof WebAssembly.RuntimeError)poison(i,error);
throw error;}finally{
i.busy=false;if(i.finalizerPending.size)flushFinalizers(i);
}};
methods.call___ocamljs_Token_value=function(a0){
const i=inst(defaultInstance);if(i.poisoned)fail('invalid or poisoned instance');if(i.busy)fail('reentrant call');
i.busy=true;
try{
const v0=handle(i,a0,"Token");

if(i.diagnostics)i.stats.calls++;
const r=i.e.__ocamljs_fn___ocamljs_Token_value(v0);
return r>>>0;
}catch(error){if(error instanceof WebAssembly.RuntimeError)poison(i,error);
throw error;}finally{
i.busy=false;if(i.finalizerPending.size)flushFinalizers(i);
}};
methods.call___ocamljs_Tokens_parse=function(a0,a1){
const i=inst(defaultInstance);if(i.poisoned)fail('invalid or poisoned instance');if(i.busy)fail('reentrant call');
let v0;
let n0,c0,u0;
i.busy=true;
try{
try{u0=Number(a0);
integer(u0,0,MAX_BYTES);
c0=u0;v0=i.e.__ocamljs_string_alloc(c0)>>>0;
a1(stringView(i),v0);n0=u0;
if(i.diagnostics)i.stats.inputBytes+=n0;

if(i.diagnostics)i.stats.calls++;
}catch(error){if(error instanceof WebAssembly.RuntimeError)poison(i,error);if(v0!==undefined)freeStringInput(i,v0,c0);
throw error;}
const r=i.e.__ocamljs_fn___ocamljs_Tokens_parse(v0,n0,c0);
return readOutput(i,r);
}catch(error){if(error instanceof WebAssembly.RuntimeError)poison(i,error);
throw error;}finally{
i.busy=false;if(i.finalizerPending.size)flushFinalizers(i);
}};
methods.call_fetch_cat=function(a0,a1){
const i=inst(defaultInstance);if(i.poisoned)fail('invalid or poisoned instance');if(i.busy)fail('reentrant call');
let v0;
let n0,c0,u0;
i.busy=true;
try{
try{u0=Number(a0);
integer(u0,0,MAX_BYTES);
c0=u0;v0=i.e.__ocamljs_string_alloc(c0)>>>0;
a1(stringView(i),v0);n0=u0;
if(i.diagnostics)i.stats.inputBytes+=n0;

if(i.jobs.size>=128)fail('task result limit exceeded');
if(i.diagnostics)i.stats.calls++;
}catch(error){if(error instanceof WebAssembly.RuntimeError)poison(i,error);if(v0!==undefined)freeStringInput(i,v0,c0);
throw error;}
const r=i.e.__ocamljs_fn_fetch_cat(v0,n0,c0);
return createTask(i,r>>>0);
}catch(error){if(error instanceof WebAssembly.RuntimeError)poison(i,error);
throw error;}finally{
i.busy=false;if(i.finalizerPending.size)flushFinalizers(i);
}};
;
export const api = Object.freeze(methods);
if (Object.prototype.hasOwnProperty.call(globalThis, "Ocamljs_example"))
  fail("runtime already registered; use the existing module");
globalThis.Ocamljs_example = api;
