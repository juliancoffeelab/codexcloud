import { renderErrorReport } from "./render-error";
import {
  configureRendererTrace,
  stopRendererTrace,
  rendererTraceStart,
  rendererTraceEnd,
  setRenderTraceSink,
} from "./render-trace";
import type { TraceBatch } from "./render-trace";
import type { TraceConfig } from "./performance-trace";
import { PhaserBattlefield } from "./phaser-battlefield";
import type { MapFrameRequest, MapRenderRequest, MapRenderResponse } from "./map-renderer";

export function createMapRenderRuntime(
  canvas: HTMLCanvasElement,
  respond: (response: MapRenderResponse | TraceBatch) => void,
) {
  let disposed = false;
  const lifetime = new AbortController();
  const emit = (response: MapRenderResponse | TraceBatch) => {
    if (!disposed) respond(response);
  };
  const releaseTrace = setRenderTraceSink(emit);
  function failureContext() {
    try {
      return {
        revision: latestRevision,
        displaySize,
        renderer: gpu?.diagnostics(),
        userAgent: navigator.userAgent,
        time: new Date().toISOString(),
      };
    } catch (error) {
      return { diagnosticsError: String(error) };
    }
  }
  let gpu: PhaserBattlefield | undefined;
  let initializing: Promise<PhaserBattlefield> | undefined;
  let traceConfig: TraceConfig | undefined;
  let displaySize: { width: number; height: number } | undefined;
  let baseKey: string | undefined;
  type RenderJob = MapFrameRequest & { revision: number };
  let scene: Omit<MapFrameRequest, "fog" | "battlefield"> | undefined;
  let battlefield: MapFrameRequest["battlefield"];
  let displayedBattlefield: MapFrameRequest["battlefield"];
  let displayedJob: RenderJob | undefined;
  let animation: number | undefined;
  let repaintRequested = false;
  let interaction = { hovered: null as string | null, focused: null as string | null };
  let pending: RenderJob | undefined;
  let running = false;
  let latestRevision = 0;
  let activeLoad: { terrainKey: string; controller: AbortController } | undefined;
  let cancelSpriteWait: (() => void) | undefined;

  async function loadImage(url: string, signal: AbortSignal, purpose = "terrain") {
    const started = rendererTraceStart();
    const response = await fetch(url, { signal });
    if (!response.ok) throw new Error(`Map image failed to load (${response.status}): ${url}`);
    const image = await createImageBitmap(await response.blob(), {
      premultiplyAlpha: purpose === "terrain" ? "none" : "premultiply",
    });
    rendererTraceEnd("Sprite fetch and decode", started, {
      width: image.width,
      height: image.height,
      asset: new URL(url, location.href).pathname,
      purpose,
    });
    return image;
  }

  // Speculative loads share the exact renderer fetch/decode/upload path with first use.
  // Only three background requests run at a time; gameplay requests bypass that queue.
  const spriteLoads = new Map<
    string,
    { key: string; url: string; controller: AbortController; promise: Promise<void> }
  >();
  let warmQueue: { url: string; key: string }[] = [];
  let warming = 0;
  function ensureSprite(url: string, key: string, background = false): Promise<void> {
    if (gpu!.hasSprite(url, key)) return Promise.resolve();
    const identity = JSON.stringify([key, url]);
    const existing = spriteLoads.get(identity);
    if (existing) return existing.promise;
    const controller = new AbortController();
    const timeout = setTimeout(
      () => controller.abort(new Error("Battlefield sprite did not load within 15 seconds")),
      15000,
    );
    const promise = loadImage(url, controller.signal, background ? "prefetch" : "required")
      .then((image) => {
        try {
          if (controller.signal.aborted) throw controller.signal.reason;
          const started = rendererTraceStart();
          gpu!.uploadSprite(url, key, image);
          rendererTraceEnd("Sprite texture upload", started, {
            asset: new URL(url, location.href).pathname,
            purpose: background ? "prefetch" : "required",
          });
        } finally {
          image.close();
        }
      })
      .finally(() => {
        clearTimeout(timeout);
        if (spriteLoads.get(identity)?.controller === controller) spriteLoads.delete(identity);
      });
    spriteLoads.set(identity, { key, url, controller, promise });
    return promise;
  }
  function warmSprites() {
    if (disposed || running || pending || !gpu) return;
    while (warming < 3 && warmQueue.length) {
      const next = warmQueue.shift()!;
      if (
        gpu.hasSprite(next.url, next.key) ||
        spriteLoads.has(JSON.stringify([next.key, next.url]))
      )
        continue;
      warming++;
      void ensureSprite(next.url, next.key, true)
        .catch(() => {
          // A failed speculative request can be retried by actual use and diagnosed there.
        })
        .finally(() => {
          warming--;
          warmSprites();
        });
    }
  }

  const decodedBases = new Map<string, ImageBitmap>();
  const decodedPixelBudget = 4_194_304;
  let preparedProfile: string | undefined;

  async function loadBase(url: string, signal: AbortSignal) {
    const cached = decodedBases.get(url);
    if (cached) {
      decodedBases.delete(url);
      decodedBases.set(url, cached);
      return { image: cached, cached: true };
    }
    const image = await loadImage(url, signal);
    if (signal.aborted) {
      image.close();
      throw signal.reason;
    }
    if (image.width * image.height > decodedPixelBudget) return { image, cached: false };
    decodedBases.set(url, image);
    let pixels = [...decodedBases.values()].reduce(
      (sum, bitmap) => sum + bitmap.width * bitmap.height,
      0,
    );
    while (decodedBases.size > 2 || pixels > decodedPixelBudget) {
      const [key, bitmap] = decodedBases.entries().next().value!;
      decodedBases.delete(key);
      pixels -= bitmap.width * bitmap.height;
      bitmap.close();
    }
    return { image, cached: true };
  }

  async function preparePrograms(request: RenderJob) {
    const lighting = request.plan.lighting;
    const profile = JSON.stringify([
      lighting?.time ?? "day",
      request.fog.length > 0,
      !!request.prebuilt,
    ]);
    if (profile === preparedProfile) return;
    await gpu!.prepare(lighting, request.fog.length > 0, !!request.prebuilt);
    preparedProfile = profile;
  }

  async function render(request: RenderJob) {
    const { plan, terrainKey, prebuilt, fog } = request;
    if (!gpu) throw new Error("Map renderer received no visible canvas");
    if (disposed || request.revision !== latestRevision) return false;
    if (baseKey !== terrainKey) {
      const load = { terrainKey, controller: new AbortController() };
      activeLoad = load;
      const timeout = setTimeout(
        () => load.controller.abort(new Error("Map images did not load within 15 seconds")),
        15000,
      );
      try {
        if (prebuilt) {
          const [loading, preparing] = await Promise.allSettled([
            loadBase(prebuilt.url, load.controller.signal),
            preparePrograms(request),
          ]);
          if (loading.status === "rejected") throw loading.reason;
          const { image, cached } = loading.value;
          try {
            if (preparing.status === "rejected") throw preparing.reason;
            if (disposed || (pending && pending.terrainKey !== terrainKey)) return false;
            gpu.setTerrain(plan, new Map(), image);
          } finally {
            if (!cached) image.close();
          }
        } else {
          const urls = [
            ...new Set(
              plan.tiles.map((tile) => {
                if (!tile.terrainFrame)
                  throw new Error(`Missing terrain frame at ${tile.x},${tile.y}`);
                return tile.terrainFrame.url;
              }),
            ),
          ];
          const images = new Map<string, ImageBitmap>();
          try {
            const results = await Promise.allSettled([
              ...urls.map(async (url) =>
                images.set(url, await loadImage(url, load.controller.signal)),
              ),
              preparePrograms(request),
            ]);
            const failed = results.find((result) => result.status === "rejected");
            if (failed?.status === "rejected") throw failed.reason;
            if (disposed || (pending && pending.terrainKey !== terrainKey)) return false;
            gpu.setTerrain(plan, images);
          } finally {
            for (const image of images.values()) image.close();
          }
        }
        baseKey = terrainKey;
      } finally {
        clearTimeout(timeout);
        if (activeLoad === load) activeLoad = undefined;
      }
    }
    await preparePrograms(request);
    if (disposed || pending) return false;
    if (request.battlefield && request.battlefield !== displayedBattlefield) {
      gpu.prepareBattlefield(request.battlefield.key);
      const urls = [
        ...new Set(
          [...request.battlefield.units, ...request.battlefield.items].flatMap((sprite) =>
            sprite.url ? [sprite.url] : [],
          ),
        ),
      ];
      // A newer revision must not wait for this revision's assets, even if decode
      // cannot be aborted. Still-needed URLs keep sharing their in-flight load.
      let cancel!: () => void;
      const superseded = new Promise<undefined>((resolve) => {
        cancel = () => resolve(undefined);
      });
      cancelSpriteWait = cancel;
      const loaded = await Promise.race([
        Promise.allSettled(urls.map((url) => ensureSprite(url, request.battlefield!.key))),
        superseded,
      ]).finally(() => {
        if (cancelSpriteWait === cancel) cancelSpriteWait = undefined;
      });
      if (disposed || !loaded || pending) return false;
      const failed = loaded.find((result) => result.status === "rejected");
      if (failed?.status === "rejected") throw failed.reason;
      if (disposed || pending) return false;
      gpu.setBattlefield(request.battlefield, new Map());
      gpu.inspect(interaction.hovered, interaction.focused);
      displayedBattlefield = request.battlefield;
      warmQueue = (request.battlefield.prefetch ?? []).map((url) => ({
        url,
        key: request.battlefield!.key,
      }));
    }
    const painted = await new Promise<boolean>((resolve, reject) => {
      requestAnimationFrame(() => {
        if (disposed || request.revision !== latestRevision) {
          resolve(false);
          return;
        }
        try {
          gpu!.render(plan.lighting, fog);
          resolve(true);
        } catch (error) {
          reject(error);
        }
      });
    });
    if (!painted) return false;
    displayedJob = request;
    return true;
  }

  function receive(request: Partial<MapRenderRequest>) {
    if (disposed) return;
    const data = request as MapRenderRequest;
    if (data.repaint) {
      repaintRequested = true;
      gpu?.invalidatePresentation();
      scheduleAnimation();
      return;
    }
    if (data.resize) {
      displaySize = data.resize;
      gpu?.setPresentationSize(displaySize);
      repaintRequested = true;
      scheduleAnimation();
      return;
    }
    if (data.traceControl) {
      const control = data.traceControl;
      if (control.stop !== undefined) {
        traceConfig = undefined;
        void stopRendererTrace(control.stop);
      } else {
        traceConfig = control.config;
        configureRendererTrace(traceConfig, gpu?.context);
      }
      return;
    }
    if (data.preparePrograms) {
      const input = data.preparePrograms;
      if (gpu)
        void gpu.prepare(input.lighting, input.fog, input.prebuilt).catch(() => {
          // Background preparation is speculative. Actual entry retries and reports
          // failures through the ordinary renderer diagnostic/retry path.
        });
      return;
    }
    if (data.motion || data.interaction) {
      if (data.motion) gpu?.pulse(data.motion.tile, data.motion.kind);
      if (data.interaction) {
        interaction = data.interaction;
        gpu?.inspect(interaction.hovered, interaction.focused);
      }
      scheduleAnimation();
      return;
    }
    latestRevision = data.revision;
    if (!gpu && !initializing) {
      canvas.addEventListener("webglcontextlost", contextLost);
      try {
        initializing = PhaserBattlefield.create(canvas, lifetime.signal).then((renderer) => {
          if (disposed) {
            renderer.destroy();
            throw new Error("Map renderer disposed during startup");
          }
          gpu = renderer;
          if (displaySize) gpu.setPresentationSize(displaySize);
          if (traceConfig) configureRendererTrace(traceConfig, gpu.context);
          return renderer;
        });
      } catch (error) {
        emit({
          revision: latestRevision,
          error: renderErrorReport("Map GPU initialization failed", error, failureContext()),
        } satisfies MapRenderResponse);
        return;
      }
    }
    if (data.scene) scene = data.scene;
    if (data.battlefield) {
      battlefield = data.battlefield;
      const wanted = new Set([
        ...[...battlefield.units, ...battlefield.items].flatMap((sprite) =>
          sprite.url ? [sprite.url] : [],
        ),
        ...(battlefield.prefetch ?? []),
      ]);
      for (const [identity, load] of spriteLoads)
        if (load.key !== battlefield.key || !wanted.has(load.url)) {
          spriteLoads.delete(identity);
          load.controller.abort(new Error("Battlefield scene replaced"));
        }
      warmQueue = [];
    }
    if (activeLoad && scene && activeLoad.terrainKey !== scene.terrainKey)
      activeLoad.controller.abort(new Error("Map scene was replaced"));
    if (!scene) {
      emit({
        revision: data.revision,
        error: "Map renderer received no scene",
      } satisfies MapRenderResponse);
      return;
    }
    pending = {
      ...scene,
      plan: { ...scene.plan, lighting: data.lighting },
      revision: data.revision,
      fog: data.fog,
      battlefield,
    };
    cancelSpriteWait?.();
    if (!running) void drain();
  }
  async function drain() {
    running = true;
    while (!disposed && pending) {
      const request = pending;
      pending = undefined;
      try {
        if (initializing) await initializing;
        if (!(await render(request)) || pending) continue;
        emit({
          revision: request.revision,
          ready: true,
          width: gpu!.canvas.width,
          height: gpu!.canvas.height,
        } satisfies MapRenderResponse);
      } catch (error) {
        if (disposed || pending) continue;
        displayedJob = undefined;
        emit({
          revision: request.revision,
          error: renderErrorReport("Map rendering failed", error, failureContext()),
        } satisfies MapRenderResponse);
      }
    }
    running = false;
    if (gpu?.animating || repaintRequested) scheduleAnimation();
    warmSprites();
  }

  // Renderer animation runs only while feedback is active. Terrain/light/fog caches
  // survive every tween frame; idle maps have no continuous render loop.
  function scheduleAnimation() {
    if (disposed || animation !== undefined || !displayedJob || running) return;
    animation = requestAnimationFrame(() => {
      animation = undefined;
      if (disposed || running || !gpu || !displayedJob) return;
      try {
        if (repaintRequested) {
          gpu.invalidatePresentation();
          repaintRequested = false;
        }
        gpu.render(displayedJob.plan.lighting, displayedJob.fog);
        if (gpu.animating) scheduleAnimation();
      } catch (error) {
        emit({
          revision: latestRevision,
          error: renderErrorReport("Battlefield animation failed", error, failureContext()),
        } satisfies MapRenderResponse);
      }
    });
  }

  function contextLost(event: Event) {
    emit({
      revision: latestRevision,
      error: renderErrorReport(
        "Map GPU context lost",
        new Error((event as WebGLContextEvent).statusMessage || "Retry map rendering"),
        failureContext(),
      ),
    });
  }
  function destroy() {
    disposed = true;
    lifetime.abort(new Error("Map renderer disposed"));
    pending = undefined;
    latestRevision++;
    warmQueue = [];
    cancelSpriteWait?.();
    activeLoad?.controller.abort();
    for (const load of spriteLoads.values()) load.controller.abort();
    spriteLoads.clear();
    for (const image of decodedBases.values()) image.close();
    decodedBases.clear();
    if (animation !== undefined) cancelAnimationFrame(animation);
    canvas.removeEventListener("webglcontextlost", contextLost);
    releaseTrace();
    gpu?.destroy();
  }
  function send(request: Partial<MapRenderRequest>) {
    try {
      receive(request);
    } catch (error) {
      emit({
        revision: latestRevision,
        error: renderErrorReport("Map renderer update failed", error, failureContext()),
      });
    }
  }
  return { send, destroy, attachParent: () => gpu?.attachParent() };
}
