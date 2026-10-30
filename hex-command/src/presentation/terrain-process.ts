import { paintLights, shadePixels, type BattlefieldLighting } from "./battlefield-lighting";

/** Uncached lighting only. Authored production scenes load finished PNGs instead. */
export async function processTerrain(
  canvas: HTMLCanvasElement,
  pixelScale: number,
  lighting: BattlefieldLighting,
  signal?: AbortSignal,
): Promise<boolean> {
  if (lighting.time === "day") return true;
  if (signal?.aborted) return false;
  const context = canvas.getContext("2d");
  if (!context) return false;
  // Transfer pixel data only. The worker needs no canvas or image APIs.
  // The original canvas remains intact for error recovery.
  if (typeof Worker !== "undefined") {
    const processed = await new Promise<boolean>((resolve) => {
      let worker: Worker | undefined;
      let finished = false;
      const finish = (ok: boolean) => {
        if (finished) return;
        finished = true;
        signal?.removeEventListener("abort", abort);
        clearTimeout(timeout);
        worker?.terminate();
        resolve(ok);
      };
      const abort = () => finish(false);
      const timeout = setTimeout(() => finish(false), 15000);
      signal?.addEventListener("abort", abort, { once: true });
      try {
        const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data.buffer;
        worker = new Worker(new URL("./terrain-process.worker.ts", import.meta.url), {
          type: "module",
        });
        worker.onerror = () => finish(false);
        worker.onmessage = (event: MessageEvent<{ pixels?: ArrayBuffer }>) => {
          if (!event.data.pixels || finished || signal?.aborted) {
            finish(false);
            return;
          }
          try {
            const result = context.createImageData(canvas.width, canvas.height);
            result.data.set(new Uint8ClampedArray(event.data.pixels));
            context.putImageData(result, 0, 0);
            finish(true);
          } catch {
            finish(false);
          }
        };
        worker.postMessage(
          { pixels, width: canvas.width, height: canvas.height, pixelScale, lighting },
          [pixels],
        );
      } catch {
        finish(false);
      }
    });
    if (signal?.aborted) return false;
    if (processed) {
      context.save();
      context.resetTransform();
      context.scale(pixelScale, pixelScale);
      paintLights(context, lighting);
      context.restore();
      return true;
    }
  }
  const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
  for (let row = 0; row < canvas.height; row += 32) {
    if (signal?.aborted) return false;
    shadePixels(
      pixels.data,
      canvas.width,
      row,
      Math.min(row + 32, canvas.height),
      pixelScale,
      lighting,
    );
    if (row + 32 < canvas.height) await new Promise<void>((resolve) => setTimeout(resolve, 0));
  }
  context.resetTransform();
  context.putImageData(pixels, 0, 0);
  context.save();
  context.scale(pixelScale, pixelScale);
  paintLights(context, lighting);
  context.restore();
  return true;
}
