import Benchmark, { maxTime } from "../options.mjs";
import { readFile, writeFile } from "node:fs/promises";
import { resolve, join } from "node:path";
import { pathToFileURL } from "node:url";
import { cases, operation } from "./cases.mjs";

const [before, after, out] = process.argv.slice(2, 5).map((p) => resolve(p));
if (!before || !after || !out)
  throw Error("Usage: node run.mjs BEFORE AFTER OUTPUT.json");
const requested = process.env.ASYNC_CASES?.split(",");
const full = process.argv.includes("--full");
const filtered = Boolean(
  requested || process.env.ASYNC_MODE || process.env.ASYNC_CASE,
);
const focus = new Set([
  "plain-melange-promise/ready",
  "plain-melange-promise/suspended",
  "owned-melange-promise/nested",
  "completion-melange-promise/text_js",
  "completion-js_of_ocaml-promise/host_echo",
  "completion-js_of_ocaml-lwt/unit",
  "completion-rescript-promise/text_native",
  "callbacks-melange-promise/callback_once",
]);
const selected = cases.filter(
  (c) =>
    (full || filtered || focus.has(`${c.mode}/${c.op}`)) &&
    (!process.env.ASYNC_MODE || c.mode.startsWith(process.env.ASYNC_MODE)) &&
    (!process.env.ASYNC_CASE ||
      `${c.mode}/${c.op}` === process.env.ASYNC_CASE) &&
    (!requested || requested.includes(`${c.mode}/${c.op}`)),
);
console.log(`${selected.length} cases × 2 variants; maxTime=${maxTime}s`);
const results = [];
for (const [label, folder] of [
  ["before", before],
  ["after", after],
]) {
  for (const mode of new Set(selected.map((c) => c.mode))) {
    delete globalThis.Ocamljs_async_modes;
    const url = pathToFileURL(join(folder, mode, "bindings.runtime.mjs"));
    const { api, initialize } = await import(`${url}?benchmark=${label}`);
    await initialize(await readFile(join(folder, mode, "backend.wasm")));
    for (const c of selected.filter((c) => c.mode === mode)) {
      const task = operation(api, c.op);
      const suite = new Benchmark.Suite();
      let failure;
      if (c.op === "callback_once")
        suite.add(`${label}/${mode}/${c.op}`, task.call);
      else
        suite.add(`${label}/${mode}/${c.op}`, {
          defer: true,
          fn(deferred) {
            task.call().then(
              () => deferred.resolve(),
              (error) => {
                failure = error;
                suite.abort();
                deferred.resolve();
              },
            );
          },
        });
      try {
        await new Promise((done, reject) => {
          suite
            .on("cycle", (event) => {
              const b = event.target;
              console.log(String(b));
              results.push({
                variant: label,
                mode,
                op: c.op,
                hz: b.hz,
                stats: b.stats,
                times: b.times,
                count: b.count,
                cycles: b.cycles,
              });
            })
            .on("error", (event) => reject(event.target.error))
            .on("complete", () => (failure ? reject(failure) : done()))
            .run({ async: true });
        });
      } finally {
        task.dispose();
      }
    }
  }
}
await writeFile(
  out,
  JSON.stringify(
    {
      framework: `Benchmark.js ${Benchmark.version}`,
      options: { maxTime },
      selection: full ? "full" : filtered ? "filtered" : "focused",
      node: process.version,
      before,
      after,
      results,
    },
    null,
    2,
  ) + "\n",
);
