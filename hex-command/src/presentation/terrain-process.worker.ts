import { paintLights, shadePixels, type BattlefieldLighting } from "./battlefield-lighting";

self.onmessage = (
  event: MessageEvent<{ bitmap: ImageBitmap; pixelScale: number; lighting: BattlefieldLighting }>,
) => {
  const { bitmap, pixelScale, lighting } = event.data;
  try {
    const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
    const context = canvas.getContext("2d");
    if (!context) throw new Error("Offscreen canvas unavailable");
    context.drawImage(bitmap, 0, 0);
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
    shadePixels(pixels.data, canvas.width, 0, canvas.height, pixelScale, lighting);
    context.putImageData(pixels, 0, 0);
    context.scale(pixelScale, pixelScale);
    paintLights(context, lighting);
    const result = canvas.transferToImageBitmap();
    self.postMessage({ bitmap: result }, { transfer: [result] });
  } catch {
    self.postMessage({ error: true });
  } finally {
    bitmap.close();
  }
};
