import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
for (const backend of ["melange", "js_of_ocaml", "rescript"]) {
  const folder = resolve(root, "demos/calculator/generated", backend);
  const { initialize, api } = await import(`${folder}/bindings.runtime.mjs`);
  const i = await initialize(await readFile(`${folder}/backend.wasm`));
  assert.equal(i.e.__ocamljs_fn___ocamljs_Geometry_distance.length, 4);
  assert.equal(i.e.__ocamljs_fn_tuple_distance.length, 4);
  const calls = [];
  i.e = Object.fromEntries(
    Object.entries(i.e).map(([name, value]) => [
      name,
      typeof value === "function"
        ? (...args) => {
            calls.push(name);
            return value(...args);
          }
        : value,
    ]),
  );
  function direct(name, args, expected) {
    calls.length = 0;
    assert.deepEqual(api[`call_${name}`](...args), expected);
    assert.deepEqual(calls, [`__ocamljs_fn_${name}`]);
  }
  direct("__ocamljs_Geometry_distance", [0, 0, 3, 4], 5);
  direct("tuple_distance", [0, 0, 3, 4], 5);
  direct("tuple_bits", ["-1", true], "-1");
  direct("nested_tuple", [-10, 12, 0.5], 2.5);
  direct("geometry_identity", [3, 4], [3, 4]);
  const first = api.call_geometry_identity(3, 4);
  direct("__ocamljs_Geometry_origin", [], [0, 0]);
  assert.deepEqual(first, [3, 4], "a later return overwrote an earlier value");
  direct("__ocamljs_IntegerGeometry_origin", [], [0, 0]);
  assert.throws(() => api.call_nested_tuple(-10, 256, 0.5), /out of range/);
  const source = i.e.__ocamljs_alloc(3);
  const view = new Uint8Array(
    i.e.memory.buffer,
    i.e.__ocamljs_input_ptr(source) >>> 0,
    3,
  );
  view.set([97, 98, 99]);
  const alloc = i.e.__ocamljs_numeric_alloc;
  const offset = view.byteOffset;
  i.e.__ocamljs_numeric_alloc = (n, kind) => {
    const id = alloc(n, kind);
    i.e.memory.grow(1);
    return id;
  };
  try {
    const args =
      backend === "js_of_ocaml"
        ? [
            3,
            (destination, destinationOffset) => {
              const source = new Uint8Array(i.e.memory.buffer, offset, 3);
              for (let j = 0; j < 3; j++)
                api.numeric_set_0(
                  destination,
                  destinationOffset + j,
                  source[j],
                );
            },
          ]
        : [view];
    assert.equal(api.call_bytes_sum(...args), 294);
  } finally {
    i.e.__ocamljs_numeric_alloc = alloc;
    i.e.__ocamljs_input_free(source);
  }
  assert.equal(view.byteLength, 0, "fixture did not detach the source view");
  assert.equal(i.stats.calls, 0, "production diagnostics were enabled");
  assert.equal(i.e.__ocamljs_live(), 0);
  const runtime = await readFile(`${folder}/bindings.runtime.mjs`, "utf8");
  assert(!runtime.includes("function callable("));
  assert(!runtime.includes("switch (type)"));
  console.log(
    `lowering/${backend}: direct records, tuples and fixed returns; checked conversions; growth-safe byte input; no leaked allocations; diagnostics opt-in`,
  );
  delete globalThis.Ocamljs_calculator;
}
