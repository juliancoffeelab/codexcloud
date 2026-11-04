/** Optional browser benchmark; never part of the default test suite. */
import { chromium } from "playwright";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import type { BattlefieldScene } from "../src/presentation/battlefield-scene";
import type { MapFrameRequest } from "../src/presentation/map-renderer";

const args = process.argv.slice(2);
const software = args.includes("--software");
const count = (name: string) => {
  const value = Number(args.find((arg) => arg.startsWith(`--${name}=`))?.split("=")[1] ?? 0);
  if (!Number.isSafeInteger(value) || value < 0 || value > 1024)
    throw new Error(`--${name} must be an integer from 0 to 1024`);
  return value;
};
const workload = { lights: count("lights"), units: count("units") };
const server = await createServer({
  root: fileURLToPath(new URL("../", import.meta.url)),
  server: { host: "127.0.0.1", port: 0, open: false },
  plugins: [
    {
      name: "map-benchmark-page",
      configureServer(server) {
        server.middlewares.use("/__map-benchmark", (_request, response) => {
          response.setHeader("Content-Type", "text/html");
          response.end(
            '<!doctype html><title>Map renderer benchmark</title><body style="margin:0;background:#222"></body>',
          );
        });
      },
    },
  ],
});
await server.listen();
const browser = await chromium.launch({
  headless: !args.includes("--headed"),
  executablePath:
    process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH ??
    (existsSync("/usr/bin/chromium") ? "/usr/bin/chromium" : undefined),
  args: software ? ["--enable-unsafe-swiftshader", "--use-angle=swiftshader"] : [],
});
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1400 } });
  // tsx preserves local function names with this helper; Playwright serializes
  // the callback without its surrounding module scope.
  await page.addInitScript("globalThis.__name = (target) => target;");
  await page.goto(server.resolvedUrls!.local[0]! + "__map-benchmark");
  const result = await page.evaluate(async (workload) => {
    const load = (path: string) => import(/* @vite-ignore */ path);
    const [
      { MapRenderer },
      { MAPS },
      { TERRAIN },
      { currentArtwork, artFrame },
      { terrainMapTiles, terrainMapKey },
      { mapDimensions, battlefieldSpriteSize, tileCenter },
    ] = await Promise.all([
      load("/src/presentation/map-renderer.ts"),
      load("/src/game/maps.ts"),
      load("/src/game/catalog.ts"),
      load("/src/presentation/art.ts"),
      load("/src/presentation/terrain-map.ts"),
      load("/src/ui/geometry.ts"),
    ]);
    const infoCanvas = new OffscreenCanvas(1, 1),
      gl = infoCanvas.getContext("webgl2");
    if (!gl)
      throw new Error(
        "WebGL 2 unavailable. Use a GPU-enabled browser or --software for diagnostic results.",
      );
    const debug = gl.getExtension("WEBGL_debug_renderer_info");
    const gpu = debug ? (gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) as string) : "Unreported";
    gl.getExtension("WEBGL_lose_context")?.loseContext();
    const map = MAPS["lantern-bell"];
    if (!map) throw new Error("Stage 3 benchmark map is unavailable");
    const roles = Object.fromEntries(
      Object.entries(TERRAIN).map(([id, definition]) => [
        id,
        (definition as { role: string }).role,
      ]),
    );
    const lighting = {
      ...map.presentation.lighting,
      lights: [
        ...map.presentation.lighting.lights,
        ...Array.from({ length: workload.lights }, (_, i) => ({
          col: (i * 7) % map.cols,
          row: (i * 11) % map.rows,
          radius: 90 + (i % 3) * 40,
          power: 0.6,
          color: [255, (i * 13) % 256, 30],
          fixture: false,
        })),
      ],
    };
    const artwork = currentArtwork();
    const battlefield: BattlefieldScene | undefined = workload.units
      ? {
          key: "benchmark",
          reducedMotion: true,
          fog: map.presentation.fog,
          tiles: map.terrain.map((tile: { id: string; col: number; row: number }) => ({
            id: tile.id,
            ...tileCenter(tile),
            classes: "hex",
          })),
          units: Array.from({ length: workload.units }, (_, i) => {
            const source = map.units[i % map.units.length],
              tile = map.terrain[i % map.terrain.length];
            return {
              id: `benchmark-${i}`,
              tile: tile.id,
              ...tileCenter(tile),
              url: artFrame(artwork, "units", source.type, source.side)?.url,
              size: battlefieldSpriteSize,
              color: "#6fdec0",
              health: 38,
              hp: 30,
              spent: false,
              retreating: false,
            };
          }),
          items: [],
        }
      : undefined;
    const plan = {
      ...mapDimensions(map),
      pixelScale: 64 / battlefieldSpriteSize,
      tiles: terrainMapTiles(map.terrain, map.id, currentArtwork(), roles),
      lighting,
    };
    const NativeWorker = window.Worker;
    let gpuMs: number | null = null;
    window.Worker = class extends NativeWorker {
      constructor(url: string | URL, options?: WorkerOptions) {
        const instrumented = String(url).includes("map-renderer.worker.ts");
        super(
          instrumented ? new URL("/scripts/map-benchmark-worker.ts", location.href) : url,
          options,
        );
        if (instrumented)
          this.addEventListener("message", (event) => {
            if (event.data.type === "benchmark-metrics") {
              event.stopImmediatePropagation();
              return;
            }
            gpuMs = event.data.benchmark?.gpuMs ?? null;
          });
      }
    };
    const host = document.createElement("div");
    host.style.cssText = "position:relative;width:1366px;height:1278px";
    document.body.append(host);
    let complete: (() => void) | undefined, fail: ((error: Error) => void) | undefined;
    const renderer = new MapRenderer(
      host,
      (message: string) => fail?.(new Error(message)),
      () => complete?.(),
    );
    const measure = async (frame: MapFrameRequest) => {
      const started = performance.now();
      const swapped = await new Promise<number>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("Map rendering timed out")), 30000);
        fail = (error) => {
          clearTimeout(timeout);
          reject(error);
        };
        complete = () => {
          clearTimeout(timeout);
          resolve(performance.now() - started);
        };
        renderer.update(frame);
      });
      const frameOpportunity = await new Promise<number>((resolve) =>
        requestAnimationFrame(() => resolve(performance.now() - started)),
      );
      return { completedAckMs: swapped, nextAnimationFrameMs: frameOpportunity, gpuMs };
    };
    try {
      const initial: MapFrameRequest = {
        terrainKey: JSON.stringify([
          terrainMapKey({ mapId: map.id, cols: map.cols, rows: map.rows, tiles: map.terrain }),
          plan.tiles.map((tile: { terrainFrame: unknown }) => tile.terrainFrame),
        ]),
        plan,
        fog: [...map.presentation.fog],
        battlefield,
      };
      const cold = await measure(initial);
      const blood = {
        col: 10,
        row: 10,
        radius: 180,
        power: 1,
        color: [255, 20, 30],
        fixture: false,
      };
      const lit = {
        ...initial,
        plan: { ...plan, lighting: { ...plan.lighting, lights: [...plan.lighting.lights, blood] } },
      };
      const fogIds = new Set<string>(initial.fog);
      for (const id of map.terrain.slice(0, 3).map((tile: { id: string }) => tile.id)) {
        if (fogIds.has(id)) fogIds.delete(id);
        else fogIds.add(id);
      }
      const fog = { ...lit, fog: [...fogIds] };
      const samples: Record<
        string,
        { completedAckMs: number; nextAnimationFrameMs: number; gpuMs: number | null }[]
      > = {
        lighting: [],
        fog: [],
      };
      // Warm up both paths before collecting bounded alternating updates.
      await measure(lit);
      await measure(fog);
      let current: MapFrameRequest = fog;
      for (let i = 0; i < 12; i++) {
        current = { ...current, plan: i % 2 ? lit.plan : plan };
        samples.lighting!.push(await measure(current));
        current = { ...current, fog: i % 2 ? fog.fog : initial.fog };
        samples.fog!.push(await measure(current));
      }
      const percentile = (values: number[], fraction: number) =>
        values.sort((a, b) => a - b)[Math.ceil(values.length * fraction) - 1]!;
      return {
        gpu,
        workload,
        softwareRenderer: /swiftshader|llvmpipe|software/i.test(gpu),
        dimensions: [
          Math.ceil(plan.width * plan.pixelScale),
          Math.ceil(plan.height * plan.pixelScale),
        ],
        cold,
        warmed: Object.fromEntries(
          Object.entries(samples).map(([name, values]) => [
            name,
            {
              samples: values.length,
              gpuTimerMedianMs: values.every((value) => value.gpuMs !== null)
                ? percentile(
                    values.map((value) => value.gpuMs!),
                    0.5,
                  )
                : null,
              completedAckMedianMs: percentile(
                values.map((v) => v.completedAckMs),
                0.5,
              ),
              completedAckP95Ms: percentile(
                values.map((v) => v.completedAckMs),
                0.95,
              ),
              nextAnimationFrameMedianMs: percentile(
                values.map((v) => v.nextAnimationFrameMs),
                0.5,
              ),
            },
          ]),
        ),
      };
    } finally {
      renderer.destroy();
      window.Worker = NativeWorker;
    }
  }, workload);
  console.log(JSON.stringify(result, null, 2));
  console.log(
    "Completed acknowledgement includes main-thread preparation, worker messaging, direct canvas rendering and a benchmark-only GPU fence. No bitmap export or main-thread canvas swap is performed. The fence deliberately waits for completed GPU work; its polling overhead is included. GPU timer results are reported when supported. Next animation frame is a scheduling opportunity, not proof of physical display scanout. No pixel readback is used. Cold time includes image loading and shader compilation.",
  );
  if (result.softwareRenderer || software)
    console.log("SOFTWARE RENDERER: these timings do not establish the hardware GPU budget.");
} finally {
  await browser.close();
  await server.close();
}
