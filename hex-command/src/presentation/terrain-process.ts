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
  // Transfer a copy: a failed/unsupported worker can still use the original canvas.
  if (
    typeof Worker !== "undefined" &&
    typeof OffscreenCanvas !== "undefined" &&
    typeof createImageBitmap !== "undefined"
  ) {
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
      void createImageBitmap(canvas)
        .then((bitmap) => {
          if (finished || signal?.aborted) {
            bitmap.close();
            finish(false);
            return;
          }
          try {
            worker = new Worker(new URL("./terrain-process.worker.ts", import.meta.url), {
              type: "module",
            });
            worker.onerror = () => finish(false);
            worker.onmessage = (event: MessageEvent<{ bitmap?: ImageBitmap }>) => {
              const result = event.data.bitmap;
              if (!result || finished || signal?.aborted) {
                result?.close();
                finish(false);
                return;
              }
              try {
                const context = canvas.getContext("2d");
                if (!context) {
                  finish(false);
                  return;
                }
                context.resetTransform();
                context.clearRect(0, 0, canvas.width, canvas.height);
                context.drawImage(result, 0, 0);
                finish(true);
              } catch {
                finish(false);
              } finally {
                result.close();
              }
            };
            worker.postMessage({ bitmap, pixelScale, lighting }, [bitmap]);
          } catch {
            bitmap.close();
            finish(false);
          }
        })
        .catch(() => finish(false));
    });
    if (processed) return true;
    if (signal?.aborted) return false;
  }
  const context = canvas.getContext("2d");
  if (!context) return false;
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
