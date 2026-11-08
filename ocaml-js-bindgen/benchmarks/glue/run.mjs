import Benchmark, { maxTime } from "../options.mjs";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { resolve, dirname, basename } from "node:path";
import { pathToFileURL } from "node:url";
import { cases } from "./cases.mjs";
import { stringCases } from "./strings.mjs";
import { operation } from "./operations.mjs";

const args = process.argv.slice(2);
const get = (name, fallback) =>
  args.includes(name) ? args[args.indexOf(name) + 1] : fallback;
const artifacts = resolve(get("--artifacts", "tmp/glue-benchmark"));
const output = resolve(get("--out", `${artifacts}/benchmarkjs.json`));
const metadata = JSON.parse(await readFile(resolve(artifacts, "build.json")));
const full = args.includes("--full");
const suiteName = get("--suite", full ? "glue" : "focus");
if (!["focus", "glue", "strings"].includes(suiteName))
  throw Error("--suite must be focus, glue or strings");
const variants = get(
  "--variants",
  full
    ? suiteName === "glue"
      ? "native,js,wasm-bindgen,game-wire"
      : metadata.variants["js-record"]
        ? "native,js,js-record,wasm-bindgen"
        : "native,js,wasm-bindgen"
    : "native,js,wasm-bindgen",
).split(",");
const focus = new Set([
  "counter-get",
  "counter-lifecycle",
  "numeric-slice/16",
  "numeric-slice/65536",
  "numeric-eight/65536",
  "text/unicode/262144/host",
  "strings/ascii4k/alias-in",
  "strings/ascii4k/host",
  "strings/latin1k/alias-in",
  "strings/json16k/result-host",
  "strings/json16k/throws-out",
  "strings/saveAscii/throws-out",
  "strings/saveUnicode/throws-out",
  "strings/saveUnicode/envelope-echo",
]);
const filtered = Boolean(get("--case") || get("--case-regex"));
const catalog =
  suiteName === "glue"
    ? cases
    : suiteName === "strings"
      ? stringCases
      : [...cases, ...stringCases];
const selected = catalog.filter(
  (c) =>
    (full || filtered || focus.has(c.name)) &&
    (!get("--case") || c.name.includes(get("--case"))) &&
    (!get("--case-regex") || new RegExp(get("--case-regex")).test(c.name)) &&
    (c.labels === undefined || metadata.pooledLabelControls) &&
    (!c.operation?.startsWith("throws-") || metadata.throwsStringControls) &&
    (c.operation !== "throws-native-out" ||
      metadata.nativeThrowsStringControls) &&
    (metadata.projectStringControls ||
      !(
        c.operation?.startsWith("result-") ||
        c.operation?.startsWith("envelope-") ||
        c.payload?.startsWith("save")
      )) &&
    (metadata.directNumeric !== false || c.kind !== "numeric-slice"),
);
if (
  variants.includes("game-wire") &&
  selected.some((c) => c.kind === "string-control")
)
  throw Error("game-wire supports --suite glue only");
console.log(
  `${suiteName}: ${selected.length} cases × ${variants.length} variants; maxTime=${maxTime}s`,
);
const results = [];
for (const variant of variants) {
  delete globalThis.Ocamljs_glue_benchmark;
  const folder =
    variant === "game-wire" || variant === "wasm-bindgen"
      ? "baseline"
      : variant;
  const files = metadata.variants[folder];
  const adapter = await import(
    pathToFileURL(resolve(artifacts, folder, "adapter.mjs"))
  );
  await adapter.load(
    await readFile(resolve(artifacts, folder, basename(files.wasm))),
  );
  for (const c of selected) {
    const task = operation(adapter, variant, c);
    const suite = new Benchmark.Suite();
    suite.add(`${variant}/${c.name}`, task.call);
    try {
      await new Promise((done, reject) => {
        suite
          .on("cycle", (event) => {
            const b = event.target;
            console.log(String(b));
            results.push({
              variant,
              case: c,
              hz: b.hz,
              stats: b.stats,
              times: b.times,
              count: b.count,
              cycles: b.cycles,
            });
          })
          .on("error", (event) => reject(event.target.error))
          .on("complete", done)
          .run({ async: true });
      });
    } finally {
      task.dispose();
    }
  }
}
await mkdir(dirname(output), { recursive: true });
await writeFile(
  output,
  JSON.stringify(
    {
      framework: `Benchmark.js ${Benchmark.version}`,
      options: { maxTime },
      selection: full ? "full" : filtered ? "filtered" : "focused",
      suite: suiteName,
      node: process.version,
      metadata,
      variants,
      results,
    },
    null,
    2,
  ) + "\n",
);
