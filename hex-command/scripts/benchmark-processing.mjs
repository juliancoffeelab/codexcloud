import { frontendModuleAliases } from "./frontend-paths.mjs";
import Benchmark, { maxTime } from "./benchmark-options.mjs";
import { readFile, writeFile, mkdir, mkdtemp, rm } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { build } from "esbuild";
import { chromium } from "playwright";
const project = resolve(import.meta.dirname, ".."),
  args = process.argv.slice(2);
const option = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
if (!option("--upstream") || !(option("--corpus") || option("--heavy")))
  throw Error("Supply --upstream PATH and --corpus FILE (or --heavy FILE)");
const upstream = resolve(option("--upstream")),
  port = resolve(option("--port", project));
const inputBytes = await readFile(resolve(option("--heavy", option("--corpus"))));
const raw = JSON.parse(inputBytes);
let inputs = option("--heavy") ? [{ policy: "predictor", checkpoint: raw.enemyStart }] : raw;
const inputIndices =
  option("--input") !== undefined
    ? [Number(option("--input"))]
    : args.includes("--full") || inputs.length < 3
      ? inputs.map((_, i) => i)
      : [0, Math.floor(inputs.length / 2), inputs.length - 1];
inputs = inputIndices.map((i) => inputs[i]);
const output = resolve(option("--out", "tmp/processing-benchmarkjs.json"));
const browserMode = args.includes("--browser"),
  scratch = await mkdtemp(resolve(project, "tmp/processing-benchmark-"));
const artifacts = option("--artifacts");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const variants = [
  { name: "port", root: port, wasm: resolve(port, "rust/pkg/backend.wasm") },
  { name: "upstream", root: upstream, wasm: resolve(upstream, "rust/pkg/hex_command_bg.wasm") },
];
const routes = new Map();
let browser, server;
const results = [];
try {
  if (artifacts) {
    await mkdir(resolve(artifacts), { recursive: true });
    await writeFile(resolve(artifacts, "corpus.json"), inputBytes);
  }
  for (const v of variants) {
    const frontend = resolve(v.root, "_build/default/frontend/output/frontend");
    const result = await build({
      entryPoints: [resolve(project, "scripts/benchmark-processing", `${v.name}.mjs`)],
      bundle: true,
      write: false,
      format: "esm",
      platform: browserMode ? "browser" : "node",
      target: "es2022",
      external: ["node:*"],
      nodePaths: [resolve(v.root, "node_modules")],
      alias: {
        "@frontend": frontend,
        ...frontendModuleAliases(v.root),
        "@pkg": resolve(v.root, "rust/pkg"),
        "@wasm": resolve(v.root, "rust/pkg"),
        "@upstream": resolve(v.root, "src"),
        melange: resolve(frontend, "../node_modules/melange"),
      },
    });
    const entry = result.outputFiles[0].contents,
      wasm = await readFile(v.wasm);
    v.commit = execFileSync("git", ["-C", v.root, "rev-parse", "HEAD"], {
      encoding: "utf8",
    }).trim();
    v.adapterSha256 = hash(entry);
    v.wasmSha256 = hash(wasm);
    routes.set(`/${v.name}.mjs`, { type: "text/javascript", bytes: entry });
    routes.set(`/${v.name}.wasm`, { type: "application/wasm", bytes: wasm });
    if (artifacts) {
      await writeFile(resolve(artifacts, `${v.name}.mjs`), entry);
      await writeFile(resolve(artifacts, `${v.name}.wasm`), wasm);
    }
    await writeFile(resolve(scratch, `${v.name}.mjs`), entry);
  }
  if (browserMode) {
    routes.set("/lodash.js", {
      type: "text/javascript",
      bytes: await readFile(resolve(project, "node_modules/lodash/lodash.js")),
    });
    routes.set("/benchmark.js", {
      type: "text/javascript",
      bytes: await readFile(resolve(project, "node_modules/benchmark/benchmark.js")),
    });
    server = createServer((request, response) => {
      if (request.url === "/") {
        response.setHeader("Content-Type", "text/html");
        response.end('<script src="/lodash.js"></script><script src="/benchmark.js"></script>');
        return;
      }
      const route = routes.get(request.url);
      if (!route) {
        response.writeHead(404);
        response.end();
        return;
      }
      response.setHeader("Content-Type", route.type);
      response.end(route.bytes);
    });
    await new Promise((done) => server.listen(0, "127.0.0.1", done));
    const base = `http://127.0.0.1:${server.address().port}`;
    browser = await chromium.launch({
      executablePath: process.env.CHROMIUM ?? "/usr/bin/chromium",
      args: ["--no-sandbox"],
    });
    const page = await browser.newPage();
    await page.goto(base);
    for (const v of variants) {
      const measured = await page.evaluate(
        async ({ name, base, inputs, maxTime }) => {
          window.Benchmark.options.maxTime = maxTime;
          const api = await import(`${base}/${name}.mjs`);
          await api.initialize(
            new Uint8Array(await (await fetch(`${base}/${name}.wasm`)).arrayBuffer()),
          );
          const suite = new window.Benchmark.Suite();
          const rows = [];
          suite.add(`${name}/corpus lifecycle`, () => {
            for (const input of inputs) api.phase(input);
          });
          try {
            await new Promise((done, reject) =>
              suite
                .on("cycle", (e) => {
                  const b = e.target;
                  rows.push({
                    variant: name,
                    hz: b.hz,
                    stats: b.stats,
                    times: b.times,
                    count: b.count,
                    cycles: b.cycles,
                  });
                })
                .on("error", (e) => reject(e.target.error))
                .on("complete", done)
                .run({ async: true }),
            );
          } finally {
            api.close();
          }
          return rows;
        },
        { name: v.name, base, inputs, maxTime },
      );
      results.push(...measured);
      console.log(v.name, measured);
    }
  } else {
    for (const v of variants) {
      const api = await import(pathToFileURL(resolve(scratch, `${v.name}.mjs`)));
      await api.initialize(await readFile(v.wasm));
      const suite = new Benchmark.Suite();
      suite.add(`${v.name}/corpus lifecycle`, () => {
        for (const input of inputs) api.phase(input);
      });
      try {
        await new Promise((done, reject) =>
          suite
            .on("cycle", (e) => {
              const b = e.target;
              console.log(String(b));
              results.push({
                variant: v.name,
                hz: b.hz,
                stats: b.stats,
                times: b.times,
                count: b.count,
                cycles: b.cycles,
              });
            })
            .on("error", (e) => reject(e.target.error))
            .on("complete", done)
            .run({ async: true }),
        );
      } finally {
        api.close();
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
        mode: browserMode ? "browser" : "node",
        operation:
          "Entire selected corpus: restore, enemy phase, save and dispose. No internal phase timing or outcome checks.",
        inputs: inputs.length,
        inputIndices,
        variants,
        results,
      },
      null,
      2,
    ) + "\n",
  );
} finally {
  await browser?.close();
  if (server) await new Promise((done) => server.close(done));
  await rm(scratch, { recursive: true, force: true });
}
