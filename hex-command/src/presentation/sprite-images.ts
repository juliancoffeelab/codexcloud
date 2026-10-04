/** Atlas coordinates exist only here, during extraction. Renderers get frame images. */
export interface SpriteFrame {
  source: string;
  crop: readonly [number, number, number, number];
  pixel: boolean;
  mask?: readonly (readonly [number, number])[];
}

export interface FrameImage {
  url: string;
  width: number;
  height: number;
  pixel: boolean;
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
  const sources = [...new Set(pending.map((frame) => frame.source))];
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
      context.imageSmoothingEnabled = !frame.pixel;
      if (frame.mask) {
        context.beginPath();
        frame.mask.forEach(([mx, my], index) =>
          index === 0 ? context.moveTo(mx, my) : context.lineTo(mx, my),
        );
        context.closePath();
        context.clip();
      }
      context.drawImage(image!, x, y, width, height, 0, 0, canvas.width, canvas.height);
      staged.set(key(frame), {
        url: canvas.toDataURL("image/png"),
        width: canvas.width,
        height: canvas.height,
        pixel: frame.pixel,
      });
    }
  } catch {
    return false;
  }
  for (const [frame, image] of staged) images.set(frame, image);
  return true;
}
