/** Atlas coordinates exist only here, during extraction. Renderers get frame images. */
export interface SpriteFrame {
  source: string;
  crop: readonly [number, number, number, number];
  /** Native placements are flattened once, before publishing the artwork snapshot. */
  layers?: readonly SpriteLayer[];
}

export interface SpriteLayer {
  source: string;
  crop: readonly [number, number, number, number];
  x: number;
  y: number;
}

export interface FrameImage {
  url: string;
  width: number;
  height: number;
}

const images = new Map<string, FrameImage>();
const decoding = new Map<string, Promise<HTMLImageElement | null>>();

function decode(source: string): Promise<HTMLImageElement | null> {
  const existing = decoding.get(source);
  if (existing) return existing;
  const loading = new Promise<HTMLImageElement | null>((resolve) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => resolve(null);
    image.src = source;
  });
  decoding.set(source, loading);
  void loading.then(() => decoding.delete(source));
  return loading;
}

function key(frame: SpriteFrame): string {
  return JSON.stringify(frame);
}

export function spriteImage(frame: SpriteFrame): FrameImage | undefined {
  return images.get(key(frame));
}

/** Extract at source resolution before publication; never expose an atlas to the DOM. */
export async function prepareSpriteImages(frames: readonly SpriteFrame[]): Promise<boolean> {
  const pending = frames.filter((frame) => !images.has(key(frame)));
  const sources = [
    ...new Set(
      pending.flatMap((frame) => [
        frame.source,
        ...(frame.layers ?? []).map((layer) => layer.source),
      ]),
    ),
  ];
  const decoded = await Promise.all(sources.map(decode));
  if (decoded.some((image) => image === null)) return false;
  const staged = new Map<string, FrameImage>();
  try {
    for (const frame of pending) {
      const image = decoded[sources.indexOf(frame.source)]!;
      const [x, y, width, height] = frame.crop;
      const canvas = document.createElement("canvas");
      canvas.width = Math.ceil(width);
      canvas.height = Math.ceil(height);
      const context = canvas.getContext("2d");
      if (!context) return false;
      context.imageSmoothingEnabled = false;
      context.drawImage(image!, x, y, width, height, 0, 0, canvas.width, canvas.height);
      for (const layer of frame.layers ?? []) {
        const [lx, ly, lw, lh] = layer.crop;
        context.drawImage(
          decoded[sources.indexOf(layer.source)]!,
          lx,
          ly,
          lw,
          lh,
          layer.x,
          layer.y,
          lw,
          lh,
        );
      }
      staged.set(key(frame), {
        url: canvas.toDataURL("image/png"),
        width: canvas.width,
        height: canvas.height,
      });
    }
  } catch {
    return false;
  }
  for (const [frame, image] of staged) images.set(frame, image);
  return true;
}
