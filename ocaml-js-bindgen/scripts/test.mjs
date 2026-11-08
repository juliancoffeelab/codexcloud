import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import { readFile, writeFile, readdir } from "node:fs/promises";
import { resolve } from "node:path";
import { createRequire } from "node:module";
const root = resolve(import.meta.dirname, "..");
const require = createRequire(import.meta.url);
for (const project of [
  "calculator",
  "hello-world",
  "todo",
  "game-of-life",
  "image-editor",
  "document-explorer",
  "async-search",
  "scripted-game",
  "cat-pics",
  "reference-conformance",
])
  for (const backend of [
    "melange",
    "js_of_ocaml",
    "rescript",
    ...([
      "async-search",
      "reference-conformance",
      "document-explorer",
      "cat-pics",
    ].includes(project)
      ? ["js_of_ocaml_lwt"]
      : []),
  ]) {
    const jsoo = backend.startsWith("js_of_ocaml");
    const processHandlers = new Map(
      ["uncaughtException", "unhandledRejection"].map((event) => [
        event,
        new Set(process.listeners(event)),
      ]),
    );
    const lwt = backend === "js_of_ocaml_lwt";
    if (backend === "melange" && project !== "reference-conformance") {
      for (const folder of ["src", "adapters", "web"]) {
        const base = resolve(root, "demos", project, folder);
        for (const name of await readdir(base, { recursive: true })) {
          assert(
            !/\.(?:m?js|cjs)$/.test(name),
            `${project}: handwritten browser JavaScript in ${folder}/${name}`,
          );
          if (/\.(?:ml|res)$/.test(name)) {
            const source = await readFile(resolve(base, name), "utf8");
            assert(
              !/%{1,2}\s*(?:(?:mel|bs)\.)?raw\b|Unsafe\.(?:(?:pure_)?js_expr|eval_string)/.test(
                source,
              ),
              `${project}: embedded JavaScript in ${folder}/${name}`,
            );
          }
        }
      }
      const html = await readFile(
        resolve(root, "demos", project, "web/index.html"),
        "utf8",
      );
      for (const script of html.matchAll(
        /<script\b([^>]*)>([\s\S]*?)<\/script>/gi,
      ))
        assert(
          /\bsrc=/.test(script[1]) && !script[2].trim(),
          `${project}: handwritten inline browser bootstrap`,
        );
    }
    const dir = resolve(
      root,
      "demos",
      project,
      "generated",
      lwt ? "lwt" : backend,
    );
    const publicApi = await readFile(`${dir}/Bindings.mli`, "utf8");
    const schema = JSON.parse(await readFile(`${dir}/schema.json`, "utf8"));
    if (jsoo) {
      assert(
        publicApi.includes(
          lwt
            ? "(unit, Load_error.t) result Lwt.t"
            : "(unit, Load_error.t) result Js_of_ocaml.Promise.t",
        ),
      );
      if (!lwt) assert(!publicApi.includes("Lwt.t"));
    }
    const expected = [
      "Wasm",
      ...Object.values(schema)
        .filter(
          (entry) =>
            (["opaque", "host", "enum"].includes(entry.kind) &&
              !entry.module) ||
            entry.kind === "module",
        )
        .map((entry) => {
          if (entry.kind === "module") return entry.member;
          if (entry.kind !== "enum") return entry.name;
          const name = entry.name
            .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
            .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
            .toLowerCase();
          return name[0].toUpperCase() + name.slice(1);
        }),
      ...["HostError", "JsString", "Uint8ClampedArray"].filter(
        (name) =>
          !schema[name] &&
          (JSON.stringify(schema).includes(`"${name}"`) ||
            (name === "HostError" &&
              Object.values(schema).some((e) => e.kind === "import"))),
      ),
    ].sort();
    assert.deepEqual(
      [...publicApi.matchAll(/^(?:module(?: rec)?|and) (\w+)\s*:/gm)]
        .map((m) => m[1])
        .filter(
          (name) =>
            !new RegExp(`^module ${name} : Map\\.S`, "m").test(publicApi),
        )
        .sort(),
      expected,
    );
    assert(!/\bOwned\b/.test(publicApi));
    assert(!publicApi.includes("__ocamljs_self"));
    assert(!publicApi.includes('ocaml.doc "\\n\\n'));
    assert(publicApi.includes("Binding validation errors raise"));
    if (project === "calculator") {
      for (const doc of [
        "Expected calculation failures.",
        "Division requires a nonzero divisor.",
        "The numeric result.",
        "Evaluates an operation and returns its history.",
        "Rust [Err] raises [Divide_error]",
        "Runs Rust Drop and invalidates this value.",
      ])
        assert(
          publicApi.includes(doc),
          `${backend}: missing generated documentation: ${doc}`,
        );
    }
    if (project === "todo")
      assert(publicApi.includes("Transfers ownership of argument 1 to Rust"));
    const implementation = await readFile(`${dir}/Bindings.ml`, "utf8");
    assert(
      !/^module rec /m.test(implementation),
      "type-only cycles must not introduce runtime module trampolines",
    );
    if (project === "calculator")
      assert(
        /^module rec Counter : sig/m.test(publicApi),
        "mutually referring resource signatures were not exercised",
      );
    assert(
      !/ocamljs_task|ocamljs_callback|ocamljs_completion|\binstance\b|default_instance/.test(
        publicApi,
      ),
    );
    if (project === "calculator") {
      assert.match(
        publicApi,
        /(?:module(?: rec)?|and) Geometry : sig[\s\S]*?type axis[\s\S]*?and point = \{ x : float; y : float \}/,
      );
      assert.match(
        publicApi,
        /val geometry_identity : Geometry\.point -> Geometry\.point/,
      );
      assert(
        !/^type .*geometry___|^and .*geometry___/m.test(publicApi),
        "private carriers leaked for self-contained module data",
      );
      assert.match(
        publicApi,
        /(?:module(?: rec)?|and) IntegerGeometry : sig[\s\S]*?and t = \{ x : int; y : int \}/,
      );
      for (const name of [
        "Geometry",
        "IntegerGeometry",
        "Direction",
        "Arithmetic",
        "Labels",
        "EmptyNamespace",
      ]) {
        const scope = publicApi.split(`${name} : sig`)[1]?.split("end")[0];
        assert(scope !== undefined);
        assert(
          !/val create|val dispose : t ->/.test(scope),
          `${name} invented ownership operations`,
        );
      }
    }
    if (project === "calculator") {
      const token = publicApi.split("Token : sig")[1]?.split("end")[0];
      assert.match(token, /val dispose : t -> unit/);
      assert(
        !/val create|val parse|val optional|val batch/.test(token),
        "invented local producer",
      );
      assert.match(
        publicApi,
        /val parse : string -> \(Token\.t, string\) result/,
      );
      assert.match(publicApi, /val optional : bool -> \(Token\.t\) option/);
      assert.match(publicApi, /val batch : unit -> \(Token\.t\) array/);
      assert(
        !Object.values(schema).some((entry) =>
          Object.hasOwn(entry, "constructor"),
        ),
      );
    }
    const { initialize, api } = await import(`${dir}/bindings.runtime.mjs`);
    const registration = Object.entries(globalThis).find(
      ([name, value]) => name.startsWith("Ocamljs_") && value === api,
    );
    assert(registration, "runtime did not register its module");
    const [globalName] = registration;
    const freshRuntime = async (name) => {
      delete globalThis[globalName];
      try {
        return await import(`${dir}/bindings.runtime.mjs?isolation=${name}`);
      } finally {
        globalThis[globalName] = api;
      }
    };
    const i = await initialize(await readFile(`${dir}/backend.wasm`));
    const originalFetch = globalThis.fetch;
    let nativeDone = false,
      ownershipDone = false;
    if (project === "async-search" && jsoo) {
      let beforeDelivery = null;
      const exports = i.e;
      i.e = {
        ...exports,
        __ocamljs_task_poll: (id) => {
          const output = exports.__ocamljs_task_poll(id);
          if (output && beforeDelivery) {
            const cancel = beforeDelivery;
            beforeDelivery = null;
            queueMicrotask(cancel);
          }
          return output;
        },
      };
      globalThis.OcamljsNative = {
        done: () => {
          ownershipDone = true;
          i.e = exports;
        },
        before_delivery: (callback) => {
          assert(!beforeDelivery);
          beforeDelivery = callback;
        },
        later: (callback) => setImmediate(callback),
      };
    }
    if (
      ["cat-pics", "reference-conformance", "document-explorer"].includes(
        project,
      )
    ) {
      globalThis.OcamljsTest = {
        done: () => {
          nativeDone = true;
        },
      };
      if (project === "cat-pics")
        globalThis.fetch = async (url) => ({
          ok: !String(url).endsWith("/failure"),
          body: new ReadableStream({
            start(controller) {
              controller.enqueue(
                new TextEncoder().encode(
                  String(url).endsWith("/bad")
                    ? "not JSON"
                    : JSON.stringify([
                        { id: "λ🦀", url: "https://example.org/cat.png" },
                      ]),
                ),
              );
              controller.close();
            },
          }),
        });
    }
    if (backend === "melange")
      await import(
        resolve(
          root,
          "demos",
          project,
          "_build/default/test-output/tests/ocaml",
          "Test.js",
        )
      );
    if (jsoo) {
      await writeFile(
        `${dir}/Test.cjs`,
        await readFile(
          resolve(
            root,
            "demos",
            project,
            "_build/default",
            lwt ? "tests/lwt/Test.bc.js" : "adapters/js_of_ocaml/Test.bc.js",
          ),
        ),
      );
      require(`${dir}/Test.cjs`);
    }
    if (backend === "rescript") {
      await import(
        resolve(root, "demos", project, "lib/es6/tests/rescript/Test.js")
      );
    }
    if (
      ["cat-pics", "reference-conformance", "document-explorer"].includes(
        project,
      )
    ) {
      for (let turns = 0; !nativeDone && turns < 128; turns++)
        await new Promise((resolve) => setImmediate(resolve));
      assert(
        nativeDone,
        `native async/${backend}: compiler-native completion missing`,
      );
      globalThis.fetch = originalFetch;
      delete globalThis.OcamljsTest;
    }
    if (project === "async-search" && jsoo) {
      for (let turns = 0; !ownershipDone && turns < 128; turns++)
        await new Promise((resolve) => setImmediate(resolve));
      assert(ownershipDone, "native async ownership checks did not complete");
      delete globalThis.OcamljsNative;
    }
    if (project === "async-search") {
      // Rust settlement queues native Promise/Lwt continuations. Let those
      // finish before another compiler installs its runtime in this process.
      for (let turns = 0; turns < 128; turns++) {
        await new Promise((resolve) => setImmediate(resolve));
        if (i.jobs.size === 0) break;
      }
      assert.equal(i.jobs.size, 0, "async fixture failed to settle");
    }
    if (i.e.__ocamljs_host_live)
      assert.equal(i.e.__ocamljs_host_live(), 0, "retained host reference");
    assert.equal(
      i.e.__ocamljs_live(),
      0,
      `${project}/${backend}: leaked ABI allocation`,
    );
    if (project === "calculator") {
      if (backend === "melange") {
        const bytes = await readFile(`${dir}/backend.wasm`);
        for (const [index, expression] of [
          "new Uint8Array(bytes)",
          "new Uint8Array(bytes).buffer",
          "new WebAssembly.Module(new Uint8Array(bytes))",
          "new DataView(new Uint8Array([0,0,...bytes]).buffer,2)",
        ].entries()) {
          const isolated = await freshRuntime(`input-${index}`);
          await isolated.initialize(
            runInNewContext(expression, { bytes: Array.from(bytes) }),
          );
        }
      }
      const operation =
        backend === "js_of_ocaml"
          ? [
              1,
              (view, offset) => {
                view[offset] = 43;
              },
            ]
          : ["+"];
      const r = api.call_calculate(3, 8, ...operation);
      assert.equal(api.r_u8(r), 0);
      assert.equal(api.r_f64(r), 11);
      api.close(r);
      assert.throws(
        () => api.call_nested((w) => api.w_u8(w, 9)),
        /invalid option tag/,
      );
      assert.throws(() => api.call_add(2147483648, 0), /out of range/);
      assert.equal(await initialize(), i, "initialization replaced the module");
      const isolated = await freshRuntime("foreign-owner");
      await isolated.initialize(await readFile(`${dir}/backend.wasm`));
      const h = api.call_counter(7);
      assert.equal(
        i.e.__ocamljs_live(),
        1,
        "direct owner return must allocate no output packet",
      );
      assert.throws(() => isolated.api.call_current(h), /foreign/);
      assert.equal(api.call_current(h), 7);
      api.dispose_Counter(h);
      assert.throws(() => api.call_oversize_mixed(), /collection budget/);
      {
        const packet = api.call_many_counters(1);
        const live = i.e.__ocamljs_live();
        const marker = new Error("direct owner callback failure");
        assert.throws(
          () =>
            api.call_counter_after_callback(() => {
              throw marker;
            }),
          (error) => error === marker,
        );
        assert.equal(
          i.e.__ocamljs_live(),
          live,
          "callback failure leaked a direct owner",
        );
        assert.equal(
          api.r_length(packet),
          1,
          "callback failure touched an unrelated packet",
        );
        api.close(packet);
      }
      {
        const originalRegistry = globalThis.FinalizationRegistry;
        const marker = new Error("owner registration failure");
        let isolated;
        globalThis.FinalizationRegistry = class extends originalRegistry {
          register() {
            throw marker;
          }
        };
        try {
          isolated = await freshRuntime("owner-registration-failure");
        } finally {
          globalThis.FinalizationRegistry = originalRegistry;
        }
        const instance = await isolated.initialize(
          await readFile(`${dir}/backend.wasm`),
        );
        assert.throws(
          () => isolated.api.call_counter(1),
          (error) => error === marker,
        );
        assert.equal(
          instance.e.__ocamljs_live(),
          0,
          "failed direct registration leaked owner",
        );
        const packet = isolated.api.call_many_counters(1);
        isolated.api.r_length(packet);
        try {
          assert.throws(
            () => isolated.api.r_handle_counter(packet),
            (error) => error === marker,
          );
        } finally {
          isolated.api.close(packet);
        }
        assert.equal(
          instance.e.__ocamljs_live(),
          0,
          "failed claimed registration leaked owner",
        );
      }
      const batch = api.call_many_counters(100);
      assert.equal(api.r_length(batch), 100);
      const first = api.r_handle_counter(batch);
      api.close(batch);
      assert.throws(() => api.call_current(first), /disposed/);
      // Two live Readers must independently refresh after ordinary and
      // claim-time memory growth; decoded strings and owners remain detached.
      const growingText = "\ufeffActive reader λ🦀";
      const textPacket = api.call_nested((w) => {
        api.w_u8(w, 1);
        api.w_u8(w, 1);
        api.w_string(w, api.native_string(growingText));
      });
      const ownersPacket = api.call_many_counters(2);
      const originalExports = i.e;
      const owners = [];
      let completed = false;
      try {
        assert.equal(api.r_u8(textPacket), 1);
        assert.equal(api.r_length(ownersPacket), 2);
        const beforeZero = i.e.memory.buffer;
        i.e.memory.grow(0);
        assert.equal(
          beforeZero.byteLength,
          0,
          "grow(0) must detach this fixture's unshared buffer",
        );
        assert.equal(api.r_u8(textPacket), 1);
        const beforeOne = i.e.memory.buffer;
        i.e.memory.grow(1);
        assert.equal(beforeOne.byteLength, 0);
        let claims = 0;
        i.e = {
          ...originalExports,
          __ocamljs_claim(...args) {
            const result = originalExports.__ocamljs_claim(...args);
            if (++claims === 1) originalExports.memory.grow(1);
            return result;
          },
        };
        const beforeClaim = i.e.memory.buffer;
        owners.push(api.r_handle_counter(ownersPacket));
        assert.equal(
          beforeClaim.byteLength,
          0,
          "claim-time growth must detach active Readers",
        );
        assert.equal(api.host_string(api.r_string(textPacket)), growingText);
        owners.push(api.r_handle_counter(ownersPacket));
        api.finish(textPacket);
        api.finish(ownersPacket);
        completed = true;
        api.close(textPacket);
        api.close(ownersPacket);
        assert.equal(api.call_current(owners[0]), 0);
        assert.equal(api.call_current(owners[1]), 1);
      } finally {
        i.e = originalExports;
        api.close(textPacket);
        api.close(ownersPacket);
        if (completed) for (const owner of owners) api.dispose_Counter(owner);
      }
      assert.equal(
        i.e.__ocamljs_live(),
        0,
        "active Reader growth leaked output or owners",
      );
      {
        const first = api.call_nested((w) => {
            api.w_u8(w, 1);
            api.w_u8(w, 1);
            api.w_string(w, api.native_string("held λ"));
          }),
          extra = [];
        try {
          // Growing the output slab may move its inline descriptors. Readers
          // retain captured numeric ranges, never the borrowed info address.
          for (let at = 0; at < 256; at++)
            extra.push(api.call_many_counters(1));
          assert.equal(api.r_u8(first), 1);
          assert.equal(api.r_u8(first), 1);
          assert.equal(api.host_string(api.r_string(first)), "held λ");
          api.finish(first);
        } finally {
          api.close(first);
          for (const packet of extra) api.close(packet);
        }
        assert.equal(
          i.e.__ocamljs_live(),
          0,
          "output-slab growth leaked allocations",
        );
      }
      {
        const original = i.e;
        const storage = original.__ocamljs_alloc(16);
        const info = original.__ocamljs_input_ptr(storage) >>> 0;
        try {
          for (const address of [0, info + 1, 0xfffffffc]) {
            i.e = { ...original, __ocamljs_output_info: () => address };
            assert.throws(
              () => api.call_many_counters(1),
              /invalid output metadata/,
            );
          }
          for (const [words, error] of [
            [[0x80000000, 0, 1, 0], /invalid output segment range/],
            [[1, 0, 0xffffffff, 0], /invalid output segment range/],
            [[1, 0xffffffff, 1, 0], /output size limit/],
            [
              [1, 32 * 1024 * 1024, 1, 32 * 1024 * 1024 + 1],
              /output size limit/,
            ],
          ]) {
            const data = new DataView(original.memory.buffer);
            words.forEach((word, at) =>
              data.setUint32(info + at * 4, word, true),
            );
            i.e = { ...original, __ocamljs_output_info: () => info };
            assert.throws(() => api.call_many_counters(1), error);
          }
        } finally {
          i.e = original;
          original.__ocamljs_input_free(storage);
        }
        assert.equal(
          i.e.__ocamljs_live(),
          0,
          "invalid metadata leaked output owners",
        );
      }
      // A valid concatenation must not hide invalid individual UTF-8 strings.
      const pair = (first, second) => {
        const input = (text) => {
          if (backend !== "js_of_ocaml") return [api.native_string(text)];
          const bytes = Buffer.from(text, "utf8");
          return [
            bytes.length,
            (destination, offset) => {
              for (let at = 0; at < bytes.length; at++)
                api.numeric_set_0(destination, offset + at, bytes[at]);
            },
          ];
        };
        return api.call_text_pair(...input(first), ...input(second));
      };
      // Exercise short COPY ranges inside a larger UTF-8 pool: tiny-pool and
      // whole-pool paths cannot cover an incorrectly indexed nativeBytes case.
      for (const length of [
        ...Array.from({ length: 16 }, (_, at) => at + 1),
        17,
        4097,
      ]) {
        const seed = ["\0", "\u009f", "\ufeff", "🦀"][(length - 1) % 4];
        const text =
          seed + "z".repeat(length - Buffer.byteLength(seed, "utf8"));
        const prefix = "long prefix ".repeat(4) + "\ufeffλ🦀";
        const packet = pair(prefix, text);
        let first, second;
        try {
          assert(packet.poolLength > 16);
          first = api.r_string(packet);
          second = api.r_string(packet);
          api.finish(packet);
          new Uint8Array(
            i.e.memory.buffer,
            packet.poolPtr,
            packet.poolLength,
          ).fill(0);
          packet.poolBytes.fill(0);
        } finally {
          api.close(packet);
        }
        assert.equal(api.host_string(first), prefix);
        assert.equal(api.host_string(second), text);
        assert.equal(
          second,
          backend === "rescript"
            ? text
            : Buffer.from(text, "utf8").toString("latin1"),
        );
        assert.equal(
          first,
          backend === "rescript"
            ? prefix
            : Buffer.from(prefix, "utf8").toString("latin1"),
        );
      }
      {
        const packet = pair("é", "");
        try {
          const body = new DataView(i.e.memory.buffer);
          body.setUint32(packet.ptr, 1, true);
          body.setUint32(packet.ptr + 4, 1, true);
          assert.throws(() => api.r_string(packet), /UTF-8 string boundary/);
        } finally {
          api.close(packet);
        }
      }
      {
        const packet = pair("\ufeffλ", "🦀\0");
        const first = api.r_string(packet);
        // Mutating the live output after validation must not alter subsequent
        // strings or invalidate the cached text/byte-boundary correspondence.
        new Uint8Array(
          i.e.memory.buffer,
          packet.poolPtr + packet.poolPosition,
          packet.poolLength - packet.poolPosition,
        ).fill(0xff);
        const before = i.e.memory.buffer;
        i.e.memory.grow(1);
        assert.equal(before.byteLength, 0);
        const second = api.r_string(packet);
        api.finish(packet);
        api.close(packet);
        assert.equal(api.host_string(first), "\ufeffλ");
        assert.equal(api.host_string(second), "🦀\0");
        assert.throws(
          () => api.r_pool_string(packet, 0),
          /truncated string pool/,
        );
      }
      for (const [left, right] of [
        ["a\0", "tiny"],
        ["a".repeat(8), "b".repeat(8)],
      ]) {
        const packet = pair(left, right);
        try {
          const first = api.r_string(packet);
          new Uint8Array(
            i.e.memory.buffer,
            packet.poolPtr,
            packet.poolLength,
          ).fill(0xff);
          i.e.memory.grow(0);
          const second = api.r_string(packet);
          assert.equal(api.host_string(first), left);
          assert.equal(api.host_string(second), right);
          api.finish(packet);
        } finally {
          api.close(packet);
        }
      }
      for (const length of [-1, 0.5, NaN, Infinity, 1000000]) {
        const packet = pair("one", "two");
        try {
          assert.throws(
            () => api.r_pool_string(packet, length),
            /truncated string pool/,
          );
        } finally {
          api.close(packet);
        }
      }
      {
        const packet = api.call_batched_output();
        let owner;
        try {
          assert.equal(api.host_string(api.r_string(packet)), "\ufeffλ");
          assert.equal(api.r_u32(packet), 17);
          assert.equal(api.r_u8(packet), 1);
          assert.equal(api.r_u32(packet), 15);
          owner = api.r_handle_counter(packet);
          // The validated text is already cached, but endpoints still require
          // checking against the immutable pool. Corrupt a body length to split.
          new DataView(i.e.memory.buffer).setUint32(
            packet.ptr + packet.position,
            1,
            true,
          );
          assert.throws(() => api.r_string(packet), /UTF-8 string boundary/);
        } finally {
          api.close(packet);
        }
        assert.throws(() => api.call_current(owner), /disposed/);
      }
      {
        const packet = api.call_batched_output();
        const original = i.e;
        let owner,
          committed = false;
        try {
          const first = api.r_string(packet);
          assert.equal(api.r_u32(packet), 17);
          assert.equal(api.r_u8(packet), 1);
          assert.equal(api.r_u32(packet), 15);
          i.e = {
            ...original,
            __ocamljs_claim(...args) {
              const result = original.__ocamljs_claim(...args);
              original.memory.grow(1);
              new Uint8Array(
                original.memory.buffer,
                packet.poolPtr,
                packet.poolLength,
              ).fill(0xff);
              return result;
            },
          };
          owner = api.r_handle_counter(packet);
          assert.equal(api.host_string(first), "\ufeffλ");
          assert.equal(api.host_string(api.r_string(packet)), "🦀\0");
          assert.equal(api.r_u8(packet), 1);
          assert.equal(api.r_u8(packet), 2);
          api.finish(packet);
          committed = true;
        } finally {
          i.e = original;
          api.close(packet);
          if (committed) api.dispose_Counter(owner);
        }
      }
      assert.equal(
        i.e.__ocamljs_live(),
        0,
        "pooled-string failure leaked owners",
      );
      const wrong = await readFile(
        resolve(root, "demos/todo/generated/melange/backend.wasm"),
      );
      const mismatch = await freshRuntime("mismatched-schema");
      await assert.rejects(
        () => mismatch.initialize(wrong),
        /schema fingerprint mismatch/,
      );
      assert.equal(i.e.__ocamljs_live(), 0);
    }
    // Each compiled js_of_ocaml client installs its own Node runtime hooks.
    // Keep them for its async checks, then restore this test realm's listeners.
    for (const [event, original] of processHandlers)
      for (const handler of process.listeners(event))
        if (!original.has(handler)) process.removeListener(event, handler);
    console.log(`${project}/${backend}: compiled and executed`);
    if (globalThis[globalName] === api) delete globalThis[globalName];
  }
