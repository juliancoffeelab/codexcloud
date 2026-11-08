import { frontendModuleAliases } from "./frontend-paths.mjs";
import Benchmark, { maxTime } from "./benchmark-options.mjs";
import { readFile, readdir, mkdir, writeFile, access } from "node:fs/promises";
import { resolve, dirname, extname, sep } from "node:path";
import { createServer } from "node:http";
import { gzipSync } from "node:zlib";
import { build } from "esbuild";
import { chromium } from "playwright";
const project = resolve(import.meta.dirname, ".."),
  args = process.argv.slice(2);
const option = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
const variants = args.flatMap((arg, i) => {
  if (arg !== "--variant") return [];
  const [name, ...path] = args[i + 1].split("=");
  return [{ name, root: resolve(path.join("=")) }];
});
if (!variants.length) throw Error("Supply --variant name=/path/to/built/project");
const scenario = option("--map", "mirror"),
  startupOnly = args.includes("--startup-only"),
  filter = option("--case", "");
const output = resolve(option("--out", "tmp/browser-benchmarkjs.json"));
const cacheMode = option("--cache", "cold"),
  cpuRate = Number(option("--cpu-rate", "1"));
const networks = {
  unthrottled: null,
  "fast-4g": { latency: 80, downloadThroughput: 9000000 / 8 },
  "slow-4g": { latency: 150, downloadThroughput: 1600000 / 8 },
  "3g": { latency: 400, downloadThroughput: 400000 / 8 },
};
const networkName = option("--network", "unthrottled");
const exists = (path) =>
  access(path).then(
    () => true,
    () => false,
  );
const files = async (path) =>
  (
    await Promise.all(
      (await readdir(path, { withFileTypes: true })).map((e) =>
        e.isDirectory() ? files(resolve(path, e.name)) : [resolve(path, e.name)],
      ),
    )
  ).flat();
for (const [index, v] of variants.entries()) {
  const ts = await exists(resolve(v.root, "src/controller/battle.ts"));
  const frontend = (await exists(resolve(v.root, "output/frontend/Native.js")))
    ? resolve(v.root, "output/frontend")
    : resolve(v.root, "_build/default/frontend/output/frontend");
  v.pkg = (await exists(resolve(v.root, "pkg")))
    ? resolve(v.root, "pkg")
    : resolve(v.root, "rust/pkg");
  const art = (await exists(resolve(v.root, "src/presentation/pixel-art.mjs")))
    ? resolve(v.root, "src/presentation/pixel-art.mjs")
    : (await exists(resolve(v.root, "src/presentation/pixel-art.ts")))
      ? resolve(v.root, "src/presentation/pixel-art.ts")
      : resolve(project, "src/presentation/pixel-art.mjs");
  const history = (await exists(resolve(v.root, "packages/save-migrations/src/legacy/data")))
    ? resolve(v.root, "packages/save-migrations/src/legacy/data")
    : resolve(project, "packages/save-migrations/src/legacy/data");
  const compiled = await build({
    entryPoints: [
      resolve(
        project,
        "scripts",
        ts ? "benchmark-session-typescript.mjs" : "benchmark-session.mjs",
      ),
    ],
    bundle: true,
    write: false,
    format: "esm",
    platform: "browser",
    target: "es2022",
    nodePaths: [resolve(v.root, "node_modules")],
    define: { "import.meta.env.BASE_URL": JSON.stringify(`/${index}/`) },
    alias: {
      "@upstream": resolve(v.root, "src"),
      "@frontend": frontend,
      ...frontendModuleAliases(v.root),
      melange: resolve(frontend, "../node_modules/melange"),
      "@wasm": v.pkg,
      "@presentation": resolve(v.root, "src/presentation"),
      "@assets/pixel-art": art,
      "@save-history": history,
      "@migration-formats": resolve(v.root, "packages/save-migrations/src/formats"),
    },
    plugins: [
      {
        name: "vite-dev-terrain-manifest",
        setup(b) {
          b.onResolve({ filter: /^virtual:terrain-prebuilt$/ }, () => ({
            path: "terrain",
            namespace: "vite-dev",
          }));
          b.onLoad({ filter: /.*/, namespace: "vite-dev" }, () => ({
            contents: "export default {};",
            loader: "js",
          }));
        },
      },
    ],
  });
  v.entry = compiled.outputFiles[0].contents;
  v.assets = await Promise.all(
    (await files(resolve(v.root, "dist")))
      .filter((p) => /\.(js|mjs|wasm|css)$/.test(p))
      .map(async (p) => {
        const bytes = await readFile(p);
        return {
          file: p.slice(v.root.length + 1),
          bytes: bytes.length,
          gzipBytes: gzipSync(bytes, { level: 9 }).length,
        };
      }),
  );
}
const dependencies = new Map();
for (const name of ["lodash", "benchmark"])
  dependencies.set(
    `/${name}.js`,
    await readFile(resolve(project, `node_modules/${name}/${name}.js`)),
  );
const server = createServer(async (request, response) => {
  try {
    const url = new URL(request.url, "http://localhost");
    if (url.pathname === "/") {
      response.setHeader("Content-Type", "text/html");
      response.end('<script src="/lodash.js"></script><script src="/benchmark.js"></script>');
      return;
    }
    if (dependencies.has(url.pathname)) {
      response.setHeader("Content-Type", "text/javascript");
      response.end(dependencies.get(url.pathname));
      return;
    }
    const parts = url.pathname.split("/").filter(Boolean),
      index = Number(parts.shift()),
      v = variants[index],
      relative = parts.join("/");
    let bytes, mime;
    if (relative === "__benchmark/session.mjs") {
      bytes = v.entry;
      mime = "text/javascript";
    } else if (relative === "__benchmark/index.html") {
      bytes = Buffer.from(
        `<script>window.benchReady=import('./session.mjs').then(async bench=>{window.bench=bench;await bench.initialize(${JSON.stringify(scenario)});});</script>`,
      );
      mime = "text/html";
    } else {
      const folder = relative.startsWith("__benchmark/") ? v.pkg : resolve(v.root, "dist");
      const path = relative.startsWith("__benchmark/")
        ? resolve(folder, parts.at(-1))
        : resolve(folder, relative || "index.html");
      if (!path.startsWith(folder + sep)) throw Error("Invalid asset path");
      bytes = await readFile(path);
      mime =
        {
          ".html": "text/html",
          ".js": "text/javascript",
          ".mjs": "text/javascript",
          ".wasm": "application/wasm",
          ".css": "text/css",
          ".png": "image/png",
          ".svg": "image/svg+xml",
          ".wav": "audio/wav",
        }[extname(path)] ?? "application/octet-stream";
    }
    response.setHeader("Content-Type", mime);
    response.setHeader("Cache-Control", cacheMode === "warm" ? "public,max-age=3600" : "no-store");
    if (
      /javascript|wasm|css|html/.test(mime) &&
      /gzip/.test(request.headers["accept-encoding"] ?? "")
    ) {
      bytes = gzipSync(bytes, { level: 9 });
      response.setHeader("Content-Encoding", "gzip");
    }
    response.end(bytes);
  } catch {
    response.writeHead(404);
    response.end();
  }
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
let browser;
try {
  browser = await chromium.launch({
    executablePath: process.env.CHROMIUM ?? "/usr/bin/chromium",
    args: [
      "--no-sandbox",
      ...(args.includes("--software")
        ? ["--use-angle=swiftshader", "--enable-unsafe-swiftshader"]
        : []),
    ],
  });
  const page = await browser.newPage({ viewport: { width: 1360, height: 1000 } });
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Network.enable");
  await cdp.send("Network.setCacheDisabled", { cacheDisabled: cacheMode === "cold" });
  if (cpuRate !== 1) await cdp.send("Emulation.setCPUThrottlingRate", { rate: cpuRate });
  if (networks[networkName])
    await cdp.send("Network.emulateNetworkConditions", {
      offline: false,
      uploadThroughput: networks[networkName].downloadThroughput,
      ...networks[networkName],
    });
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  const pageFailure = new Promise((_, reject) => page.on("pageerror", reject));
  const results = await Promise.race([
    pageFailure,
    page.evaluate(
      async ({ variants, scenario, startupOnly, filter, maxTime }) => {
        window.Benchmark.options.maxTime = maxTime;
        const rows = [];
        for (let index = 0; index < variants.length; index++) {
          const core = document.createElement("iframe");
          core.width = 1360;
          core.height = 1000;
          core.src = `/${index}/__benchmark/index.html`;
          const loaded = new Promise((resolve, reject) => {
            core.onload = resolve;
            core.onerror = reject;
          });
          document.body.append(core);
          await loaded;
          await core.contentWindow.benchReady;
          const api = core.contentWindow.bench,
            checkpoint = await api.checkpoint();
          const tasks = startupOnly ? [] : await api.cases();
          const openGame = async () => {
            await api.prepare(checkpoint);
            return new Promise((resolve, reject) => {
              localStorage.clear();
              localStorage.setItem("hex-command-sound-enabled", "false");
              const frame = document.createElement("iframe");
              frame.width = 1360;
              frame.height = 1000;
              frame.onerror = reject;
              frame.onload = () => {
                const doc = frame.contentDocument;
                const ready = () => {
                  const failure = doc.querySelector(
                    '.loading-shell[aria-busy="false"] .startup-failure pre',
                  );
                  if (failure) {
                    observer.disconnect();
                    frame.remove();
                    reject(Error(failure.textContent));
                    return;
                  }
                  const terrain = doc.querySelector("canvas.map-terrain, .map-terrain canvas");
                  if (
                    doc.querySelector("#end-turn")?.disabled !== false ||
                    !terrain ||
                    !(
                      terrain.style.visibility === "visible" ||
                      (!terrain.style.visibility && terrain.width > 300)
                    )
                  )
                    return;
                  observer.disconnect();
                  Promise.all(
                    [...doc.images].filter((i) => i.getClientRects().length).map((i) => i.decode()),
                  ).then(
                    () =>
                      requestAnimationFrame(() =>
                        requestAnimationFrame(() => {
                          resolve(frame);
                        }),
                      ),
                    reject,
                  );
                };
                const observer = new frame.contentWindow.MutationObserver(ready);
                observer.observe(doc, { subtree: true, childList: true, attributes: true });
                ready();
              };
              frame.src = `/${index}/`;
              document.body.append(frame);
            });
          };

          tasks.push({
            name: "startup-to-playable iframe lifecycle",
            defer: true,
            call: async () => {
              const frame = await openGame();
              frame.remove();
            },
          });
          if (!startupOnly)
            tasks.push({
              name: "rendered commands iframe lifecycle",
              defer: true,
              call: async () => {
                const frame = await openGame();
                try {
                  const doc = frame.contentDocument;
                  const saved = async () => JSON.parse(await api.readCheckpoint()).checkpoint.state;
                  const paint = () =>
                    new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done)));
                  const wait = async (test) => {
                    while (!(await test())) await paint();
                  };
                  const click = (selector) =>
                    doc
                      .querySelector(selector)
                      .dispatchEvent(
                        new frame.contentWindow.MouseEvent("click", { bubbles: true }),
                      );
                  const initial = await saved(),
                    actor =
                      initial.units.find((u) => u.id === "u3") ??
                      initial.units.find((u) => u.side === "player");
                  click(`#map [data-tile="${actor.tile}"]`);
                  await paint();
                  const occupied = new Set(initial.units.map((u) => u.tile));
                  const destination = [...doc.querySelectorAll("#map .reachable[data-tile]")]
                    .map((n) => n.getAttribute("data-tile"))
                    .find((id) => id !== actor.tile && !occupied.has(id));
                  click(`#map [data-tile="${destination}"]`);
                  await wait(
                    async () =>
                      (await saved()).units.find((u) => u.id === actor.id).tile === destination,
                  );
                  await paint();
                  click("#undo-move");
                  await wait(
                    async () =>
                      (await saved()).units.find((u) => u.id === actor.id).tile === actor.tile,
                  );
                  await paint();
                  click("#end-turn");
                  await wait(async () => {
                    const current = await saved();
                    return (
                      current.winner ||
                      (current.round === initial.round + 1 &&
                        current.turn === "player" &&
                        !doc.querySelector("#end-turn").disabled)
                    );
                  });
                  await paint();
                } finally {
                  frame.remove();
                }
              },
            });
          const suite = new window.Benchmark.Suite();
          let failure;
          for (const task of tasks.filter((t) => !filter || t.name.includes(filter))) {
            const name = `${variants[index].name}/${task.name}`;
            if (task.defer)
              suite.add(name, {
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
            else suite.add(name, task.call);
          }
          try {
            await new Promise((done, reject) =>
              suite
                .on("cycle", (e) => {
                  const b = e.target;
                  rows.push({
                    name: b.name,
                    hz: b.hz,
                    stats: b.stats,
                    times: b.times,
                    count: b.count,
                    cycles: b.cycles,
                  });
                })
                .on("error", (e) => reject(e.target.error))
                .on("complete", () => (failure ? reject(failure) : done()))
                .run({ async: true }),
            );
          } finally {
            api.close();
            core.remove();
          }
        }
        return rows;
      },
      { variants: variants.map((v) => ({ name: v.name })), scenario, startupOnly, filter, maxTime },
    ),
  ]);
  await mkdir(dirname(output), { recursive: true });
  await writeFile(
    output,
    JSON.stringify(
      {
        framework: `Benchmark.js ${Benchmark.version}`,
        options: { maxTime },
        browser: browser.version(),
        operation:
          "Browser-native Benchmark.js functions. Startup includes repository reset/seeding, iframe creation, navigation, playable/terrain readiness and iframe disposal. Rendered commands additionally include DOM move/undo/end-turn and two animation frames after each state transition. Session commands include create/move/undo/end-turn/close.",
        networkName,
        cpuRate,
        cacheMode,
        variants: variants.map(({ entry, ...v }) => v),
        results,
      },
      null,
      2,
    ) + "\n",
  );
  console.log(results);
} finally {
  await browser?.close();
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
}
