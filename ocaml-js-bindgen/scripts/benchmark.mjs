import Benchmark from "../benchmarks/options.mjs";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
const root = resolve(import.meta.dirname, "..");
const { initialize, api } = await import(
  `${root}/demos/calculator/generated/melange/bindings.runtime.mjs`
);
const instance = await initialize(
  await readFile(`${root}/demos/calculator/generated/melange/backend.wasm`),
);
const B = await import(
  `${root}/demos/calculator/_build/default/test-output/src/ocaml/Bindings.js`
);
const counter = B.Counter.create(7);
const suite = new Benchmark.Suite();
suite
  .add("raw WASM scalar", () => instance.e.__ocamljs_fn_add(7, 1))
  .add("runtime scalar", () => api.call_add(7, 1))
  .add("compiled OCaml scalar", () => B.add(7, 1))
  .add("compiled OCaml opaque read", () => B.Counter.current(counter));
try {
  await new Promise((done, reject) =>
    suite
      .on("cycle", (event) => console.log(String(event.target)))
      .on("error", (event) => reject(event.target.error))
      .on("complete", done)
      .run({ async: true }),
  );
} finally {
  B.Counter.dispose(counter);
}
