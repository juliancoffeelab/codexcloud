// Actual Node worker fixture: instantiate ordinary bindings in this realm.
import { parentPort } from "node:worker_threads";
import { readFile } from "node:fs/promises";
import { initialize, api } from "../generated/melange/bindings.runtime.mjs";
import * as B from "../_build/default/output/src/ocaml/Bindings.js";
import assert from "node:assert/strict";
const instance = await initialize(
  await readFile(new URL("../generated/melange/backend.wasm", import.meta.url)),
);
assert(!("worker_load" in api));

const result = await B.digest(api.native_string("λ🦀"));
assert.equal(result.TAG, 0);
assert.equal(result._0, 2);
const corpus = B.Corpus.create(0),
  input = B.Input.create(0);
const pending = B.Index.create(corpus, input, () => {});
assert.throws(() => B.Corpus.revision(corpus));
B.Input.finish(input);
const index = await pending;
assert.equal(index.TAG, 0);
B.Index.dispose(index._0);
B.Input.dispose(input);
B.Corpus.dispose(corpus);
const checkCorpus = B.Corpus.create(0),
  cancel = B.SearchCancel.create(0),
  progress = [];
const ndjson =
  Array.from({ length: 16 }, (_, n) =>
    JSON.stringify({ text: `row${n}` }),
  ).join("\n") + "\n".repeat(257);
const checked = await B.Index.build_url(
  checkCorpus,
  "data:application/x-ndjson," + encodeURIComponent(ndjson),
  cancel,
  (event) => progress.push(event.records),
);
assert.equal(checked.TAG, 0);
assert.deepEqual(progress, [16]);
B.Index.dispose(checked._0);
B.SearchCancel.dispose(cancel);
B.Corpus.dispose(checkCorpus);
const blobCorpus = B.Corpus.create(0);
const blob = B.$$Blob.of_text(
  api.native_string('{"text":"native Blob λ🦀"}\n'),
);
assert.equal(blob.TAG, 0);
const native = await B.Index.of_blob(blobCorpus, blob._0, () => {});
assert.equal(native.TAG, 0);
assert.deepEqual(B.Index.query(native._0, api.native_string("Blob")), [
  api.native_string("native Blob λ🦀"),
]);
B.Index.dispose(native._0);
B.Corpus.dispose(blobCorpus);
assert.equal(instance.e.__ocamljs_host_live(), 0);
assert.equal(B.live_entries(0), 0);
assert.equal(instance.e.__ocamljs_tasks(), 0);
parentPort.postMessage({ kind: "passed", chars: result._0 });
