/** Opt-in geometry contract. A failing app must produce a nonzero exit status. */
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { chromium } from "playwright";
import { preview } from "vite";

if (process.argv.includes("--help")) {
  console.log(
    "Run after npm run build: node scripts/probe-battlefield-geometry.js\nGEOMETRY_DIST selects an existing production directory; CHROMIUM_PATH selects Chromium.\nFour opt-in cold/warm desktop Mirror and phone saved Bell cases; JSON/screenshots and wall time in tmp/battlefield-geometry/. Exits nonzero on any invariant violation.",
  );
  process.exit(0);
}

const started = performance.now();
const output = `tmp/battlefield-geometry/${new Date().toISOString().replaceAll(/[:.]/g, "-")}`;
await mkdir(output, { recursive: true });
const dist = resolve(process.env.GEOMETRY_DIST || "dist");
assert(existsSync(resolve(dist, "index.html")), "Build production assets first: npm run build");
const report = { dist, cases: [], preparationSeconds: 0, runSeconds: 0 };
let server, browser;

// The same oracle runs live and against injected faults. Opening details never
// replaces either baseline. Geometry tolerance is a quarter physical pixel.
function violations(base, sample, tolerance) {
  const result = [];
  const differs = (a, b) => Math.abs(a - b) > tolerance;
  if (base.reserved && !sample.reserved) return ["window-disappeared"];
  if (base.reserved && sample.reserved) {
    const [x, y, w, h] = base.reserved,
      [sx, sy, sw, sh] = sample.reserved;
    if (differs(y, sy)) result.push("window-top");
    if (differs(h, sh)) result.push("window-height");
    if (differs(x, sx)) result.push("window-left");
    if (sample.details) {
      if (sw > w + tolerance) result.push("clip-expanded");
    } else if (differs(w, sw)) result.push("window-width");
  }
  if (base.projection && !sample.projection) result.push("world-disappeared");
  if (base.projection && sample.projection) {
    if (base.transform !== sample.transform) result.push("canvas-transform");
    if (
      base.projection.some((point, i) =>
        point.some((n, axis) => differs(n, sample.projection[i][axis])),
      )
    )
      result.push("world-projection");
    if (base.canvas.some((n, i) => differs(n, sample.canvas[i]))) result.push("canvas-geometry");
  }
  return result;
}

function instrument(checkSource) {
  const check = (0, eval)(`(${checkSource})`);
  const probe = (window.__geometry = {
    tools: {},
    frames: [],
    failures: [],
    resources: [],
    phase: "startup",
  });
  Object.defineProperty(document, "modelContext", {
    value: {
      registerTool: (t) => (probe.tools[t.name] = t),
      unregisterTool: (n) => delete probe.tools[n],
    },
  });
  let phaser;
  Object.defineProperty(window, "Phaser", {
    get: () => phaser,
    set(value) {
      phaser = value;
      const boot = value.Game.prototype.boot;
      value.Game.prototype.boot = function () {
        probe.game = this;
        return boot.call(this);
      };
    },
  });
  new PerformanceObserver((list) => {
    for (const r of list.getEntries())
      probe.resources.push({
        at: r.responseEnd,
        url: new URL(r.name).pathname,
        transferSize: r.transferSize,
        decodedBodySize: r.decodedBodySize,
      });
  }).observe({ type: "resource", buffered: true });
  const rect = (node) => {
    if (!node) return null;
    const r = node.getBoundingClientRect();
    return [r.x, r.y, r.width, r.height];
  };
  probe.capture = () => {
    const host = document.querySelector("#map, .loading-map");
    const game = probe.game,
      canvas = game?.canvas,
      camera = game?.scene?.getScene("battlefield")?.cameras.main;
    const canvasRect = rect(canvas);
    const visible = canvas?.isConnected && getComputedStyle(canvas).visibility === "visible";
    let projection = null;
    if (visible && camera)
      projection = [
        [0, 0],
        [100, 0],
        [0, 100],
      ].map(([x, y]) => {
        const p = camera.matrix.transformPoint(x - camera.scrollX, y - camera.scrollY);
        return [
          canvasRect[0] + (p.x * canvasRect[2]) / canvas.width,
          canvasRect[1] + (p.y * canvasRect[3]) / canvas.height,
        ];
      });
    return {
      at: performance.now(),
      phase: probe.phase,
      reserved: rect(host),
      canvas: canvasRect,
      clip: host ? [host.clientWidth, host.clientHeight] : null,
      transform: canvas ? getComputedStyle(canvas).transform : null,
      loadingCard: rect(document.querySelector(".loading-card, .loading-message")),
      details: !!document.querySelector(".workspace:not(.map-only)"),
      visible: !!visible,
      projection,
      progress: document.querySelector("progress")
        ? [document.querySelector("progress").value, document.querySelector("progress").max]
        : null,
    };
  };
  let base, world, previous;
  const seen = new Set();
  const sample = () => {
    const next = probe.capture();
    probe.frames.push(next);
    if (!base && next.reserved) base = next;
    if (!world && next.projection) world = next;
    if (base)
      for (const kind of check(
        {
          ...base,
          projection: world?.projection,
          canvas: world?.canvas,
          transform: world?.transform,
        },
        next,
        0.25 / devicePixelRatio,
      )) {
        if (seen.has(kind)) continue;
        seen.add(kind);
        probe.failures.push({
          kind,
          baseline: kind.startsWith("world") || kind.startsWith("canvas") ? world : base,
          before: previous,
          after: next,
        });
      }
    previous = next;
    requestAnimationFrame(sample);
  };
  requestAnimationFrame(sample);
}
const frames = (page) =>
  page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))),
  );
const phase = (page, value) =>
  page.evaluate((value) => {
    __geometry.phase = value;
  }, value);
const ready = (page) =>
  page.waitForFunction(() => document.querySelector("#map canvas")?.style.visibility === "visible");
async function tools(page, name, input) {
  return page.evaluate(({ name, input }) => __geometry.tools[name].execute(input), { name, input });
}

// Keep requests pending, rather than adding real network sleeps to the test.
function delivery(page) {
  let module,
    engine,
    enabled = true;
  const images = [],
    heldAudio = [];
  const install = async () => {
    await page.route("**/*.wasm", (route) => (enabled ? (engine = route) : route.continue()));
    await page.route("**/assets/phaser-battlefield-*.js", (route) =>
      enabled ? (module = route) : route.continue(),
    );
    await page.route("**/assets/sprites/*.png", (route) =>
      enabled ? images.push(route) : route.continue(),
    );
    // Holding one optional audio request leaves the rest of the six native slots usable.
    await page.route("**/assets/audio/bell-1.wav*", (route) =>
      enabled && !heldAudio.length ? heldAudio.push(route) : route.continue(),
    );
  };
  return {
    install,
    async releaseEngine() {
      assert(engine, "WASM request was not held");
      await phase(page, "engine-release");
      await engine.continue();
    },
    async releaseModule() {
      await page.waitForFunction(() => !!document.querySelector("#map"));
      assert(module, "Phaser request was not held");
      await module.continue();
    },
    async current() {
      const deadline = performance.now() + 30000;
      let count = 0;
      while (
        !(await page.evaluate(
          () => document.querySelector("#map canvas")?.style.visibility === "visible",
        ))
      ) {
        assert(
          performance.now() < deadline,
          "Current map did not appear while releasing image dependencies",
        );
        const route = images.shift();
        if (route) {
          await phase(page, `image-${++count}`);
          await route.continue();
        }
        await frames(page);
      }
      return count;
    },
    async finish() {
      enabled = false;
      await phase(page, "background-release");
      await Promise.all(
        [...images.splice(0), ...heldAudio.splice(0)].map((r) => r.continue().catch(() => {})),
      );
      await frames(page);
    },
  };
}

async function inspect(page, directory) {
  await phase(page, "details-open");
  const point = await page.evaluate(() => {
    const game = __geometry.game,
      scene = game.scene.getScene("battlefield"),
      camera = scene.cameras.main;
    const unit = scene.frame.view.game.units.find(
      (u) => scene.frame.view.game.faction(u.id).controllable && u.role === "leader",
    );
    const tile = scene.frame.tiles.find((t) => t.id === unit.tile),
      bounds = game.canvas.getBoundingClientRect();
    const projected = camera.matrix.transformPoint(
      tile.x - camera.scrollX,
      tile.y - camera.scrollY,
    );
    return {
      x: bounds.left + (projected.x * bounds.width) / game.canvas.width,
      y: bounds.top + (projected.y * bounds.height) / game.canvas.height,
    };
  });
  await page.mouse.click(point.x, point.y);
  await page.waitForFunction(() => !!document.querySelector(".workspace:not(.map-only)"));
  await frames(page);
  await page.screenshot({ path: `${directory}/details.png` });
  await phase(page, "details-close");
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => !!document.querySelector(".workspace.map-only"));
  await frames(page);
}

async function mutations(page) {
  const check = violations;
  const before = await page.evaluate(() => __geometry.capture());
  await page.evaluate(() => {
    const host = document.querySelector("#map");
    __geometry.originalStyle = host.getAttribute("style");
    host.style.height = `${host.getBoundingClientRect().height + 40}px`;
  });
  await frames(page);
  const height = check(before, await page.evaluate(() => __geometry.capture()), 0.125);
  await page.evaluate(() => {
    document.querySelector("#map").setAttribute("style", __geometry.originalStyle);
  });
  await frames(page);
  const b2 = await page.evaluate(() => __geometry.capture());
  await page.evaluate(() => {
    const outer = document.querySelector(".battle-content");
    __geometry.outerTop = outer.scrollTop;
    __geometry.outerStyle = outer.getAttribute("style");
    // Fault delivery must work even when the correct architecture forbids
    // parent scrolling. Temporarily make the ancestor illegally scrollable.
    outer.style.overflow = "auto";
    const spacer = document.createElement("div");
    spacer.dataset.geometryFault = "scroll";
    spacer.style.height = "80px";
    outer.append(spacer);
    outer.scrollTop += 20;
  });
  await frames(page);
  const scroll = check(b2, await page.evaluate(() => __geometry.capture()), 0.125);
  await page.evaluate(() => {
    const outer = document.querySelector(".battle-content");
    outer.scrollTop = __geometry.outerTop;
    outer.querySelector('[data-geometry-fault="scroll"]').remove();
    if (__geometry.outerStyle === null) outer.removeAttribute("style");
    else outer.setAttribute("style", __geometry.outerStyle);
  });
  await frames(page);
  const b3 = await page.evaluate(() => __geometry.capture());
  await page.evaluate(() => {
    const camera = __geometry.game.scene.getScene("battlefield").cameras.main;
    __geometry.zoom = [camera.zoomX, camera.zoomY];
    camera.setZoom(camera.zoomX * 0.8, camera.zoomY * 0.8);
  });
  await frames(page);
  const fit = check(b3, await page.evaluate(() => __geometry.capture()), 0.125);
  await page.evaluate(() => {
    __geometry.game.scene.getScene("battlefield").cameras.main.setZoom(...__geometry.zoom);
  });
  assert(height.includes("window-height"), "Oracle missed injected height change");
  assert(scroll.includes("window-top"), "Oracle missed injected parent scroll");
  assert(fit.includes("world-projection"), "Oracle missed injected camera fit");
  return { height, scroll, fit };
}

try {
  server = await preview({
    logLevel: "error",
    build: { outDir: dist },
    preview: {
      host: "127.0.0.1",
      port: 0,
      strictPort: true,
      headers: { "Cache-Control": "public, max-age=3600" },
    },
  });
  const url = `http://127.0.0.1:${server.httpServer.address().port}`;
  browser = await chromium.launch({
    executablePath: process.env.CHROMIUM_PATH || "/usr/bin/chromium",
    args: ["--no-sandbox", "--enable-unsafe-swiftshader"],
  });
  report.browser = browser.version();
  // Seed a real save through the normal Rust/controller command boundary.
  const seed = await browser.newContext();
  await seed.addInitScript(instrument, violations.toString());
  const seedPage = await seed.newPage();
  await seedPage.goto(url);
  await ready(seedPage);
  await tools(seedPage, "start_campaign", { id: "bell-night" });
  await frames(seedPage);
  await ready(seedPage);
  const savedBell = await seed.storageState();
  await seed.close();
  report.preparationSeconds = (performance.now() - started) / 1000;
  const runStart = performance.now();
  for (const fixture of [
    { name: "desktop-mirror", viewport: { width: 1280, height: 900 } },
    {
      name: "phone-bell",
      viewport: { width: 390, height: 844 },
      mobile: true,
      storageState: savedBell,
    },
  ]) {
    const context = await browser.newContext({
      viewport: fixture.viewport,
      deviceScaleFactor: 2,
      isMobile: !!fixture.mobile,
      hasTouch: true,
      storageState: fixture.storageState,
    });
    await context.addInitScript(instrument, violations.toString());
    const page = await context.newPage();
    page.setDefaultTimeout(30000);
    const errors = [];
    page.on("pageerror", (e) => errors.push(e.stack ?? String(e)));
    for (const warm of [false, true]) {
      const begin = performance.now(),
        name = `${fixture.name}-${warm ? "warm" : "cold"}`,
        directory = `${output}/${name}`;
      await mkdir(directory);
      const result = { name };
      const loader = delivery(page);
      try {
        if (!warm) await loader.install();
        if (warm) {
          await page.unrouteAll({ behavior: "wait" });
          const prime = performance.now();
          await page.reload({ waitUntil: "domcontentloaded" });
          await ready(page);
          result.cachePrimeSeconds = (performance.now() - prime) / 1000;
          await page.reload({ waitUntil: "domcontentloaded" });
        } else {
          await page.goto(url, { waitUntil: "domcontentloaded" });
          await page.waitForFunction(() => !!document.querySelector(".loading-map, #map"));
          await page.screenshot({ path: `${directory}/before.png` });
          await loader.releaseEngine();
          await page.waitForFunction(() => !!document.querySelector("#map"));
          await frames(page);
          await page.screenshot({ path: `${directory}/handoff.png` });
          await loader.releaseModule();
          result.releasedImages = await loader.current();
          await loader.finish();
        }
        await ready(page);
        await frames(page);
        await page.screenshot({ path: `${directory}/ready.png` });
        await inspect(page, directory);
        const trace = await page.evaluate(() => {
          const { frames, failures, resources } = __geometry;
          return { frames, failures, resources };
        });
        result.failures = trace.failures;
        result.cacheHits = trace.resources.filter(
          (r) => r.transferSize === 0 && r.decodedBodySize > 0,
        ).length;
        if (fixture.mobile && warm) result.oracleMutations = await mutations(page);
        await writeFile(`${directory}/trace.json`, JSON.stringify(trace, null, 2));
        result.passed = !trace.failures.length && !errors.length;
      } catch (error) {
        result.passed = false;
        result.error = error.stack ?? String(error);
      }
      result.errors = [...errors];
      result.seconds = (performance.now() - begin) / 1000;
      report.cases.push(result);
      console.log(
        `${name}: ${result.passed ? "PASS" : "FAIL"}, ${result.seconds.toFixed(2)}s, ${(result.failures ?? []).map((f) => f.kind).join(", ")}${result.error ? `; ${result.error}` : ""}`,
      );
    }
    await context.close();
  }
  report.runSeconds = (performance.now() - runStart) / 1000;
} finally {
  await browser?.close();
  await server?.close();
  report.totalSeconds = (performance.now() - started) / 1000;
  await writeFile(`${output}/report.json`, JSON.stringify(report, null, 2));
  console.log(
    `Browser run ${report.runSeconds.toFixed(2)}s; preparation ${report.preparationSeconds.toFixed(2)}s; total ${report.totalSeconds.toFixed(2)}s. Artifacts: ${output}`,
  );
}
process.exitCode = report.cases.length === 4 && report.cases.every((c) => c.passed) ? 0 : 1;
