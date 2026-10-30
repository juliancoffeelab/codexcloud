import { shadePixels, type BattlefieldLighting } from "./battlefield-lighting";

self.onmessage = (
  event: MessageEvent<{
    pixels: ArrayBuffer;
    width: number;
    height: number;
    pixelScale: number;
    lighting: BattlefieldLighting;
  }>,
) => {
  const { pixels, width, height, pixelScale, lighting } = event.data;
  try {
    shadePixels(new Uint8ClampedArray(pixels), width, 0, height, pixelScale, lighting);
    self.postMessage({ pixels }, { transfer: [pixels] });
  } catch {
    self.postMessage({ error: true });
  }
};
