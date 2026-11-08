import assert from "node:assert/strict";
import {
  readFileSync,
  writeFileSync,
  mkdtempSync,
  rmSync,
  mkdirSync,
} from "node:fs";
import { resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
mkdirSync(resolve(root, "tmp"), { recursive: true });
const temporary = mkdtempSync(resolve(root, "tmp/packaging-"));
const pack = resolve(root, "target/debug/ocamljs_pack");
const source = (name) => {
  const project = {
    calculator: "calculator",
    image_editor_demo: "image-editor",
    cat_pics_demo: "cat-pics",
    hello_world: "hello-world",
  }[name];
  assert(project, `unknown project artifact: ${name}`);
  return resolve(
    root,
    "demos",
    project,
    "rust/target/wasm32-unknown-unknown/release",
    `${name}.wasm`,
  );
};
const leb = (value) => {
  const bytes = [];
  do {
    const byte = value & 127;
    value >>>= 7;
    bytes.push(byte | (value ? 128 : 0));
  } while (value);
  return Buffer.from(bytes);
};
const section = (id, bytes) =>
  Buffer.concat([Buffer.from([id]), leb(bytes.length), bytes]);
const text = (value) =>
  Buffer.concat([leb(Buffer.byteLength(value)), Buffer.from(value)]);
const custom = (name, bytes) => section(0, Buffer.concat([text(name), bytes]));
const customSections = (bytes, name) =>
  WebAssembly.Module.customSections(new WebAssembly.Module(bytes), name);
let importNumber = 0;
// Only this test releases the global FFI bridge while importing isolated glue.
// Captured raw APIs stay lexical; compiled clients continue using the main API.
async function isolatedRuntime(path) {
  const previous = globalThis.Ocamljs_packaging;
  delete globalThis.Ocamljs_packaging;
  try {
    return await import(`${path}?isolated=${++importNumber}`);
  } finally {
    delete globalThis.Ocamljs_packaging;
    if (previous) globalThis.Ocamljs_packaging = previous;
  }
}
function sections(bytes) {
  let at = 8;
  const unsigned = () => {
    let value = 0,
      shift = 0,
      byte;
    do {
      byte = bytes[at++];
      value |= (byte & 127) << shift;
      shift += 7;
    } while (byte & 128);
    return value >>> 0;
  };
  const result = [];
  while (at < bytes.length) {
    const start = at,
      id = bytes[at++],
      length = unsigned(),
      end = at + length;
    let name;
    if (id === 0) {
      const size = unsigned();
      name = bytes.subarray(at, at + size).toString();
    }
    result.push({ name, bytes: bytes.subarray(start, end) });
    at = end;
  }
  return result;
}
function generate(input, folder, extra = [], valid = true) {
  const result = spawnSync(
    pack,
    [
      "generate",
      "--wasm",
      input,
      "--out-dir",
      folder,
      "--name",
      "packaging",
      ...extra,
    ],
    { cwd: root, encoding: "utf8", timeout: 10000 },
  );
  assert.equal(
    result.status,
    valid ? 0 : 1,
    result.error?.message ?? result.stderr,
  );
  return result;
}

try {
  const input = resolve(temporary, "input.wasm"),
    release = resolve(temporary, "release"),
    debug = resolve(temporary, "debug");
  const original = readFileSync(source("calculator"));
  writeFileSync(
    input,
    Buffer.concat([
      original,
      custom(".debug_test", Buffer.from("debug information")),
    ]),
  );
  const badAsyncMode = generate(
    input,
    resolve(temporary, "invalid-async"),
    ["--backend", "js_of_ocaml", "--async-runtime", "unknown"],
    false,
  );
  assert.match(badAsyncMode.stderr, /--async-runtime must be promise or lwt/);
  const badAsyncBackend = generate(
    input,
    resolve(temporary, "invalid-backend"),
    ["--backend", "melange", "--async-runtime", "lwt"],
    false,
  );
  assert.match(
    badAsyncBackend.stderr,
    /Lwt generation requires the js_of_ocaml backend/,
  );
  // A command run inside a Rust directory finds the enclosing project's
  // installed ReScript compiler without a platform path or an injected BSC.
  const rustDirectory = resolve(temporary, "rust");
  const localCompilerOutput = resolve(temporary, "local-rescript");
  mkdirSync(rustDirectory);
  const compilerEnv = { ...process.env };
  delete compilerEnv.BSC;
  const compilerLookup = spawnSync(
    pack,
    [
      "generate",
      "--wasm",
      input,
      "--backend",
      "rescript",
      "--name",
      "packaging",
      "--out-dir",
      localCompilerOutput,
    ],
    { cwd: rustDirectory, env: compilerEnv, encoding: "utf8", timeout: 5000 },
  );
  assert.equal(
    compilerLookup.status,
    0,
    compilerLookup.error?.message ?? compilerLookup.stderr,
  );
  assert(
    readFileSync(resolve(localCompilerOutput, "Bindings.resi"), "utf8").length >
      0,
  );
  generate(input, release);
  generate(input, debug, ["--dev"]);
  const bytes = readFileSync(resolve(release, "backend.wasm"));
  const debugBytes = readFileSync(resolve(debug, "backend.wasm"));
  for (const name of ["name", "producers", "ocamljs.schema", ".debug_test"])
    assert.equal(
      customSections(bytes, name).length,
      0,
      `release retained ${name}`,
    );
  assert.equal(
    customSections(debugBytes, "ocamljs.schema").length,
    customSections(original, "ocamljs.schema").length,
  );
  // Walrus remaps supported DWARF after rewriting function indices. Unknown
  // pseudo-DWARF must not be copied with potentially stale code offsets.
  assert.equal(customSections(debugBytes, ".debug_test").length, 0);
  assert.equal(
    customSections(debugBytes, "name").length,
    customSections(original, "name").length,
  );
  const stamps = customSections(bytes, "ocamljs.abi");
  assert.equal(stamps.length, 1);
  assert.match(Buffer.from(stamps[0]).toString(), /^13:[0-9a-f]{64}$/);
  const runtime = readFileSync(
    resolve(release, "bindings.runtime.mjs"),
    "utf8",
  );
  assert(!/JSON\.parse|crypto|ocamljs\.schema/.test(runtime));
  // Calculator conformance now exercises callbacks and async selectors. Check
  // pruning against the actual scalar-only project instead of that broad fixture.
  const pruned = resolve(temporary, "pruned");
  generate(source("hello_world"), pruned);
  assert(
    !/function createTask|function startFetch|callback_retain/.test(
      readFileSync(resolve(pruned, "bindings.runtime.mjs"), "utf8"),
    ),
  );
  assert(!runtime.includes('"body"') && !runtime.includes('"variants"'));
  const { initialize, api } = await import(
    resolve(release, "bindings.runtime.mjs")
  );
  const cryptoDescriptor = Object.getOwnPropertyDescriptor(
      globalThis,
      "crypto",
    ),
    parse = JSON.parse;
  let i;
  try {
    Object.defineProperty(globalThis, "crypto", {
      value: undefined,
      configurable: true,
    });
    JSON.parse = () => {
      throw new Error("runtime parsed schema JSON");
    };
    i = await initialize(bytes);
  } finally {
    if (cryptoDescriptor)
      Object.defineProperty(globalThis, "crypto", cryptoDescriptor);
    else delete globalThis.crypto;
    JSON.parse = parse;
  }
  assert.equal(api.call_add(2, 3), 5);
  assert.equal(api.module_failed(), false);
  assert.equal(api.module_failure(), "");
  assert.equal(await initialize(debugBytes), i);
  assert(api.load_ok(await api.load("ignored-after-initialization.wasm")));
  let invalidLoad;
  assert.doesNotThrow(() => {
    invalidLoad = api.load("\xff");
  });
  const invalidResult = await invalidLoad;
  assert.equal(api.load_kind(invalidResult), "unexpected");
  assert(api.load_cause(invalidResult) instanceof TypeError);
  assert.equal(api.module_failed(), false);
  await (
    await isolatedRuntime(resolve(debug, "bindings.runtime.mjs"))
  ).initialize(debugBytes);
  await assert.rejects(
    () => import(`${resolve(release, "bindings.runtime.mjs")}?duplicate`),
    /runtime already registered/,
  );
  assert.equal(globalThis.Ocamljs_packaging, api);
  // URL loads compile as the response streams, without an ArrayBuffer copy.
  // Noncanonical MIME types and runtimes without compileStreaming retain the
  // buffered path. HTTP failures and streamed schema mismatches are retryable.
  {
    const fetch = globalThis.fetch,
      compile = WebAssembly.compile,
      streaming = WebAssembly.compileStreaming,
      instantiate = WebAssembly.instantiate;
    try {
      for (const [mime, available, status, payload, expected] of [
        ["application/wasm", true, 200, bytes, "stream"],
        ["application/octet-stream", true, 200, bytes, "buffer"],
        [null, true, 200, bytes, "buffer"],
        ["application/wasm; charset=utf-8", true, 200, bytes, "buffer"],
        ["Application/Wasm", true, 200, bytes, "buffer"],
        ["application/wasm", false, 200, bytes, "buffer"],
        [
          "application/wasm",
          true,
          404,
          bytes,
          /Wasm download failed: HTTP 404/,
        ],
        [
          "application/wasm",
          true,
          200,
          bytes.subarray(0, 8),
          /schema fingerprint mismatch/,
        ],
        [
          "application/wasm",
          true,
          200,
          new Uint8Array([0]),
          (error) =>
            error.kind === "compile" &&
            error.cause instanceof WebAssembly.CompileError,
        ],
      ]) {
        const fresh = await isolatedRuntime(
          resolve(release, "bindings.runtime.mjs"),
        );
        let streamed = 0,
          buffered = 0,
          compiled = 0,
          instantiated = 0;
        const response = new Response(payload, {
          status,
          headers: mime ? { "Content-Type": mime } : {},
        });
        const arrayBuffer = response.arrayBuffer.bind(response);
        response.arrayBuffer = () => {
          buffered++;
          return arrayBuffer();
        };
        globalThis.fetch = async () => response;
        WebAssembly.compile = (...args) => {
          // Optional string helpers are separate modules, not network fallbacks.
          if (!args[1]?.builtins?.includes("js-string")) compiled++;
          return compile(...args);
        };
        WebAssembly.compileStreaming = available
          ? (input) => {
              streamed++;
              return streaming(input);
            }
          : undefined;
        WebAssembly.instantiate = (...args) => {
          if (
            !(args[0] instanceof WebAssembly.Module) ||
            WebAssembly.Module.exports(args[0]).some(
              (entry) => entry.name === "__ocamljs_abi",
            )
          )
            instantiated++;
          return instantiate(...args);
        };
        const pending = fresh.initialize(
          "https://fixture.invalid/backend.wasm",
        );
        if (typeof expected === "string") {
          await pending;
          assert.equal(fresh.api.call_add(2, 3), 5);
          assert.equal(streamed, expected === "stream" ? 1 : 0);
          assert.equal(buffered, expected === "buffer" ? 1 : 0);
          assert.equal(compiled, buffered);
          assert.equal(instantiated, 1);
        } else {
          await assert.rejects(pending, expected);
          assert.equal(instantiated, 0, "failed validation instantiated Wasm");
          assert.equal(buffered, 0, "streaming failure reread the response");
          assert.equal(fresh.api.module_failed(), false);
          await fresh.initialize(bytes);
          assert.equal(fresh.api.call_add(2, 3), 5);
        }
      }
    } finally {
      globalThis.fetch = fetch;
      WebAssembly.compile = compile;
      WebAssembly.compileStreaming = streaming;
      WebAssembly.instantiate = instantiate;
    }
  }
  {
    const fresh = await isolatedRuntime(
      resolve(release, "bindings.runtime.mjs"),
    );
    assert.equal(fresh.api.module_failed(), false);
    assert.throws(() => fresh.api.call_add(1, 2), /invalid instance/);
    const compile = WebAssembly.compile,
      instantiate = WebAssembly.instantiate;
    let compiles = 0,
      instantiations = 0,
      recursive;
    try {
      WebAssembly.compile = (...args) => {
        if (!args[1]?.builtins?.includes("js-string")) {
          compiles++;
          recursive = fresh.initialize(bytes);
        }
        return compile(...args);
      };
      WebAssembly.instantiate = (...args) => {
        if (
          !(args[0] instanceof WebAssembly.Module) ||
          WebAssembly.Module.exports(args[0]).some(
            (entry) => entry.name === "__ocamljs_abi",
          )
        )
          instantiations++;
        return instantiate(...args);
      };
      const first = fresh.initialize(bytes),
        concurrent = fresh.initialize(bytes);
      assert.equal(first, concurrent);
      const [left, right] = await Promise.all([first, concurrent]);
      assert.equal(left, right);
      assert.equal(
        recursive,
        first,
        "synchronous host reentry started another initialization",
      );
      assert.equal(await fresh.initialize(new Uint8Array()), left);
      assert.equal(compiles, 1);
      assert.equal(instantiations, 1);
    } finally {
      WebAssembly.compile = compile;
      WebAssembly.instantiate = instantiate;
    }
  }
  const again = resolve(temporary, "again");
  generate(resolve(release, "backend.wasm"), again, [
    "--schema",
    resolve(release, "schema.json"),
  ]);
  assert.equal(
    readFileSync(resolve(again, "Bindings.ml"), "utf8"),
    readFileSync(resolve(release, "Bindings.ml"), "utf8"),
  );
  assert.deepEqual(readFileSync(resolve(again, "backend.wasm")), bytes);
  const debugAgain = resolve(temporary, "debug-again");
  generate(resolve(debug, "backend.wasm"), debugAgain, [
    "--schema",
    resolve(debug, "schema.json"),
  ]);
  assert.deepEqual(readFileSync(resolve(debugAgain, "backend.wasm")), bytes);
  const wrongSchema = JSON.parse(
    readFileSync(resolve(release, "schema.json"), "utf8"),
  );
  wrongSchema.add.docs = "different contract metadata";
  const wrongSidecar = resolve(temporary, "wrong.json");
  writeFileSync(wrongSidecar, JSON.stringify(wrongSchema));
  assert.match(
    generate(
      resolve(release, "backend.wasm"),
      resolve(temporary, "wrong"),
      ["--schema", wrongSidecar],
      false,
    ).stderr,
    /sidecar does not match/,
  );
  const noStamp = Buffer.concat([
    bytes.subarray(0, 8),
    ...sections(bytes)
      .filter((s) => s.name !== "ocamljs.abi")
      .map((s) => s.bytes),
  ]);
  const stamp = Buffer.from(stamps[0]);
  const changed = Buffer.from(stamp);
  changed[changed.length - 1] ^= 1;
  for (const invalid of [
    noStamp,
    Buffer.concat([bytes, custom("ocamljs.abi", stamp)]),
    Buffer.concat([noStamp, custom("ocamljs.abi", changed)]),
    Buffer.concat([noStamp, custom("ocamljs.abi", Buffer.from([255]))]),
  ]) {
    const fresh = await isolatedRuntime(
      resolve(release, "bindings.runtime.mjs"),
    );
    await assert.rejects(
      () => fresh.initialize(invalid),
      /schema fingerprint mismatch/,
    );
    assert.equal(fresh.api.module_failed(), false);
    await fresh.initialize(bytes);
    assert.equal(
      fresh.api.call_add(1, 2),
      3,
      "failed initialization did not retry",
    );
  }
  // The independent ABI export check remains necessary even with a matching stamp.
  const wrongAbi = Buffer.concat([
    Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]),
    section(1, Buffer.from([1, 96, 0, 1, 127])),
    section(3, Buffer.from([1, 0])),
    section(5, Buffer.from([1, 0, 1])),
    section(
      7,
      Buffer.concat([
        Buffer.from([2]),
        text("memory"),
        Buffer.from([2, 0]),
        text("__ocamljs_abi"),
        Buffer.from([0, 0]),
      ]),
    ),
    section(10, Buffer.from([1, 4, 0, 65, 3, 11])),
    custom("ocamljs.abi", stamp),
  ]);
  await assert.rejects(
    () =>
      isolatedRuntime(resolve(release, "bindings.runtime.mjs")).then((fresh) =>
        fresh.initialize(wrongAbi),
      ),
    /ABI version mismatch/,
  );
  {
    const { initialize, api } = await isolatedRuntime(
      resolve(release, "bindings.runtime.mjs"),
    );
    const trapped = await initialize(bytes),
      exports = trapped.e,
      marker = new WebAssembly.RuntimeError("reader trap");
    let afterTrap = 0,
      threw = false;
    trapped.e = Object.fromEntries(
      Object.entries(exports).map(([name, value]) => [
        name,
        typeof value === "function"
          ? (...args) => {
              if (threw) afterTrap++;
              if (name === "__ocamljs_output_info") {
                threw = true;
                throw marker;
              }
              return value(...args);
            }
          : value,
      ]),
    );
    assert.throws(
      () => api.call_many_counters(1),
      (error) => error === marker,
    );
    assert.equal(api.module_failed(), true);
    assert.match(api.module_failure(), /reader trap/);
    assert.equal(api.module_failure_cause(), marker);
    assert.throws(() => api.call_add(1, 2), /poisoned/);
    assert.equal(afterTrap, 0, "runtime entered Wasm after a trap");
    await assert.rejects(() => initialize(bytes), /reader trap/);
    assert.equal(afterTrap, 0, "initialization replaced a poisoned singleton");
  }
  // An imported JS exception may arrive after Rust already freed an owner.
  // Neither explicit disposal nor finalization may retry that pointer.
  for (const mode of ["dispose", "finalize", "chain"]) {
    const NativeRegistry = globalThis.FinalizationRegistry;
    const originalReport = globalThis.reportError;
    const registered = new WeakMap();
    let finalize;
    globalThis.FinalizationRegistry = class {
      constructor(callback) {
        finalize = callback;
      }
      register(target, held, token) {
        registered.set(token ?? target, held);
      }
      unregister(token) {
        return registered.delete(token);
      }
    };
    globalThis.reportError = () => {};
    try {
      const { initialize, api } = await isolatedRuntime(
        resolve(release, "bindings.runtime.mjs"),
      );
      const instance = await initialize(bytes);
      const owner = api.call_counter(1),
        held = registered.get(owner);
      const originalExports = instance.e;
      if (mode === "chain") {
        const middle = api.call_counter(2),
          root = api.call_counter(3);
        let middleFreed = false,
          rootFreed = false;
        instance.e = {
          ...originalExports,
          __ocamljs_dispose: (id) => {
            if (
              (id === owner.ptr && !middleFreed) ||
              (id === middle.ptr && !rootFreed)
            )
              return 0;
            const status = originalExports.__ocamljs_dispose(id);
            if (status && id === middle.ptr) middleFreed = true;
            if (status && id === root.ptr) rootFreed = true;
            return status;
          },
        };
        finalize(held);
        finalize(registered.get(middle));
        assert.equal(instance.finalizerPending.size, 2);
        finalize(registered.get(root));
        assert.equal(instance.finalizerPending.size, 0);
        assert.equal(originalExports.__ocamljs_live(), 0);
        continue;
      }
      const marker = new Error("exception after owner destruction");
      let frees = 0;
      instance.e = {
        ...originalExports,
        __ocamljs_dispose: (id) => {
          frees++;
          assert.equal(originalExports.__ocamljs_dispose(id), 1);
          throw marker;
        },
      };
      if (mode === "dispose") {
        assert.throws(
          () => api.dispose_Counter(owner),
          (error) => error === marker,
        );
        assert.equal(owner.ptr, 0);
        assert.equal(registered.has(owner), false);
      } else finalize(held);
      assert.equal(api.module_failed(), true);
      assert.equal(instance.finalizerPending.size, 0);
      finalize(held);
      assert.equal(frees, 1, "finalizer retried an indeterminate pointer");
    } finally {
      globalThis.FinalizationRegistry = NativeRegistry;
      if (originalReport === undefined) delete globalThis.reportError;
      else globalThis.reportError = originalReport;
    }
  }
  for (const target of [
    "__ocamljs_fn_add",
    "__ocamljs_alloc",
    "__ocamljs_input_ptr",
    "__ocamljs_input_free",
    "__ocamljs_string_alloc",
    "__ocamljs_string_realloc",
    "__ocamljs_string_free",
    "__ocamljs_output_info",
    "__ocamljs_claim",
    "__ocamljs_output_free",
    "__ocamljs_dispose",
  ]) {
    const { initialize, api } = await isolatedRuntime(
      resolve(release, "bindings.runtime.mjs"),
    );
    const instance = await initialize(bytes),
      originalExports = instance.e;
    let output, owner;
    if (["__ocamljs_claim", "__ocamljs_output_free"].includes(target)) {
      output = api.call_many_counters(1);
      api.r_length(output);
    }
    if (target === "__ocamljs_dispose") owner = api.call_counter(1);
    let trapped = false,
      after = 0;
    const marker = new WebAssembly.RuntimeError(target);
    instance.e = Object.fromEntries(
      Object.entries(originalExports).map(([name, value]) => [
        name,
        typeof value === "function"
          ? (...args) => {
              if (trapped) after++;
              if (name === target) {
                trapped = true;
                throw marker;
              }
              return value(...args);
            }
          : value,
      ]),
    );
    const invoke = () => {
      if (target === "__ocamljs_fn_add") return api.call_add(1, 2);
      if (target === "__ocamljs_claim") return api.r_handle_counter(output);
      if (target === "__ocamljs_output_free") return api.close(output);
      if (target === "__ocamljs_dispose") return api.dispose_Counter(owner);
      if (target === "__ocamljs_output_info") return api.call_many_counters(1);
      if (target.startsWith("__ocamljs_string_"))
        return api.call_selector_text_echo("λ");
      if (
        [
          "__ocamljs_alloc",
          "__ocamljs_input_ptr",
          "__ocamljs_input_free",
        ].includes(target)
      )
        return api.call___ocamljs_Direction_flip((writer) =>
          api.w_u32(writer, 0),
        );
      return api.call_calculate(1, 2, "+");
    };
    assert.throws(invoke, (error) => error === marker);
    if (output) api.close(output);
    assert.equal(api.module_failed(), true, target);
    assert.equal(instance.busy, false, target);
    assert.equal(after, 0, `${target}: cleanup entered poisoned Wasm`);
  }
  const image = resolve(temporary, "image");
  generate(source("image_editor_demo"), image);
  const imageRuntime = await isolatedRuntime(
    resolve(image, "bindings.runtime.mjs"),
  );
  const imageInstance = await imageRuntime.initialize(
    readFileSync(resolve(image, "backend.wasm")),
  );
  const imageApi = imageRuntime.api;
  const output = imageApi.call_open_png(
    readFileSync(resolve(root, "demos/image-editor/tests/fixture.png")),
  );
  assert.equal(imageApi.r_u8(output), 0);
  const owner = imageApi.r_handle_image(output);
  imageApi.finish(output);
  imageApi.close(output);
  const imageExports = imageInstance.e;
  let moveTrapped = false,
    moveAfterTrap = 0;
  imageInstance.e = Object.fromEntries(
    Object.entries(imageExports).map(([name, value]) => [
      name,
      typeof value === "function"
        ? (...args) => {
            if (moveTrapped) moveAfterTrap++;
            if (name === "__ocamljs_fn_upload") {
              moveTrapped = true;
              throw new WebAssembly.RuntimeError("move trap");
            }
            return value(...args);
          }
        : value,
    ]),
  );
  assert.throws(() => imageApi.call_upload(owner), /move trap/);
  assert.equal(
    moveAfterTrap,
    0,
    "ownership reconciliation entered poisoned Wasm",
  );
  assert.equal(imageApi.module_failed(), true);
  const cats = resolve(temporary, "cats");
  generate(source("cat_pics_demo"), cats);
  assert(
    !readFileSync(resolve(cats, "bindings.runtime.mjs"), "utf8").includes(
      "callback_retain",
    ),
  );
  const originalFetch = globalThis.fetch;
  try {
    // The Rust collector must report TooLarge, regardless of the browser's
    // chunk size. A transport-side total cap used to turn this into Network.
    for (const size of [64 * 1024 + 1, 1024 * 1024 + 1]) {
      globalThis.fetch = async () =>
        new Response(new Uint8Array(size).fill(120));
      const fresh = await isolatedRuntime(
        resolve(cats, "bindings.runtime.mjs"),
      );
      const instance = await fresh.initialize(
        readFileSync(resolve(cats, "backend.wasm")),
      );
      const task = fresh.api.call_fetch_cat("https://fixture.invalid");
      fresh.api.task_map(task, (r) => {
        try {
          assert.equal(fresh.api.r_u8(r), 1); // Result::Err
          assert.equal(fresh.api.r_u32(r), 1); // CatError::TooLarge
          fresh.api.finish(r);
        } finally {
          fresh.api.close(r);
        }
      });
      await fresh.api.task_await(task);
      await new Promise((resolve) => setImmediate(resolve));
      assert.equal(instance.fetches.size, 0);
      assert.equal(instance.e.__ocamljs_live(), 0);
    }
    globalThis.fetch = async () => ({
      ok: true,
      body: new ReadableStream({
        start(controller) {
          controller.enqueue(
            new TextEncoder().encode(
              '[{"id":"sample","url":"https://example.test/a.png"}]',
            ),
          );
          controller.close();
        },
      }),
    });
    for (const target of [
      "__ocamljs_fetch_push",
      "__ocamljs_fetch_end",
      "__ocamljs_input_free",
    ]) {
      const catRuntime = await isolatedRuntime(
        resolve(cats, "bindings.runtime.mjs"),
      );
      const instance = await catRuntime.initialize(
        readFileSync(resolve(cats, "backend.wasm")),
      );
      const exports = instance.e;
      let trapped = false,
        after = 0,
        pushed = false;
      instance.e = Object.fromEntries(
        Object.entries(exports).map(([name, value]) => [
          name,
          typeof value === "function"
            ? (...args) => {
                if (trapped) after++;
                if (name === "__ocamljs_fetch_push") pushed = true;
                if (
                  name === target &&
                  (target !== "__ocamljs_input_free" || pushed)
                ) {
                  trapped = true;
                  throw new WebAssembly.RuntimeError(target);
                }
                return value(...args);
              }
            : value,
        ]),
      );
      const task = catRuntime.api.call_fetch_cat("https://example.test");
      catRuntime.api.task_map(task, () => assert.fail("trapped fetch decoded"));
      await assert.rejects(catRuntime.api.task_await(task), (error) =>
        error.message.includes(target),
      );
      await new Promise((resolve) => setImmediate(resolve));
      assert.equal(after, 0, `${target}: fetch cleanup entered poisoned Wasm`);
      assert.equal(instance.busy, false);
      assert.equal(catRuntime.api.module_failed(), true);
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
  // Malformed imported URLs complete on a queued microtask. The queued bridge
  // must both guard its own export and skip it if an earlier call poisoned Wasm.
  for (const alreadyPoisoned of [false, true]) {
    const catRuntime = await isolatedRuntime(
      resolve(cats, "bindings.runtime.mjs"),
    );
    const instantiate = WebAssembly.instantiate;
    let bridge, instance;
    try {
      WebAssembly.instantiate = (module, imports) => {
        bridge = imports.ocamljs;
        return instantiate(module, imports);
      };
      instance = await catRuntime.initialize(
        readFileSync(resolve(cats, "backend.wasm")),
      );
    } finally {
      WebAssembly.instantiate = instantiate;
    }
    const exports = instance.e,
      input = exports.__ocamljs_alloc(1),
      pointer = exports.__ocamljs_input_ptr(input);
    new Uint8Array(exports.memory.buffer, pointer, 1)[0] = 255;
    let trapped = false,
      after = 0,
      ends = 0;
    instance.e = Object.fromEntries(
      Object.entries(exports).map(([name, value]) => [
        name,
        typeof value === "function"
          ? (...args) => {
              if (trapped) after++;
              if (name === "__ocamljs_fetch_end") ends++;
              if (
                name ===
                (alreadyPoisoned
                  ? "__ocamljs_fn_fetch_cat"
                  : "__ocamljs_fetch_end")
              ) {
                trapped = true;
                throw new WebAssembly.RuntimeError("queued fetch trap");
              }
              return value(...args);
            }
          : value,
      ]),
    );
    bridge.fetch_open(0, pointer, 1);
    exports.__ocamljs_input_free(input);
    if (alreadyPoisoned)
      assert.throws(
        () => catRuntime.api.call_fetch_cat("https://example.test"),
        /queued fetch trap/,
      );
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(ends, alreadyPoisoned ? 0 : 1);
    assert.equal(after, 0);
    assert.equal(instance.busy, false);
    assert.equal(catRuntime.api.module_failed(), true);
  }
  console.log(
    `packaging: stamp/ABI rejection, validated sidecar, metadata stripping, no-crypto load and trap cleanup passed (calculator Wasm ${original.length} → ${bytes.length} bytes; runtime ${Buffer.byteLength(runtime)} bytes)`,
  );
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
