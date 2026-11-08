import Benchmark from "../benchmarks/options.mjs";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
const root = resolve(import.meta.dirname, "..");
const { initialize, api } = await import(
  `${root}/demos/async-search/generated/melange/bindings.runtime.mjs`
);
await initialize(
  await readFile(`${root}/demos/async-search/generated/melange/backend.wasm`),
);
const B = await import(
  `${root}/demos/async-search/_build/default/output/src/ocaml/Bindings.js`
);
const text =
  Array.from({ length: 4096 }, (_, n) =>
    JSON.stringify({ text: `OCaml row ${n}` }),
  ).join("\n") + "\n";
const url = "data:application/x-ndjson," + encodeURIComponent(text);
const raw = () =>
  api.task_await(
    api.task_map(api.call_digest("hello"), (r) => {
      api.r_u8(r);
      const value = api.r_u32(r);
      api.finish(r);
      return value;
    }),
  );
const sync = () => {
  const corpus = B.Corpus.create(0);
  try {
    const result = B.Index.build_sync(corpus, text);
    B.Index.dispose(result._0);
  } finally {
    B.Corpus.dispose(corpus);
  }
};
const asyncIndex = async () => {
  const corpus = B.Corpus.create(0),
    cancel = B.SearchCancel.create(0);
  try {
    const result = await B.Index.build_url(corpus, url, cancel, () => {});
    B.Index.dispose(result._0);
  } finally {
    B.SearchCancel.dispose(cancel);
    B.Corpus.dispose(corpus);
  }
};
const suite = new Benchmark.Suite();
let failure;
for (const [name, call] of [
  ["runtime Promise", raw],
  ["compiled OCaml Promise", () => B.digest("hello")],
  ["compiled async indexing", asyncIndex],
]) {
  suite.add(name, {
    defer: true,
    fn(deferred) {
      call().then(
        () => deferred.resolve(),
        (error) => {
          failure = error;
          suite.abort();
          deferred.resolve();
        },
      );
    },
  });
}
suite.add("compiled sync indexing", sync);
await new Promise((done, reject) =>
  suite
    .on("cycle", (event) => console.log(String(event.target)))
    .on("error", (event) => reject(event.target.error))
    .on("complete", () => (failure ? reject(failure) : done()))
    .run({ async: true }),
);
