/** Phaser 3.90 customEnvironment host for a dedicated module worker.
 * DOM input/audio/scale features are disabled in the game configuration. The
 * adapter supplies only startup canvas/Image support, never main-thread work.
 * Revalidate this boundary when upgrading the pinned Phaser version.
 */
export function installPhaserWorkerPlatform() {
  const scope = globalThis as unknown as Record<string, unknown>;
  if (scope.document) throw new Error("Phaser battlefield requires a dedicated worker");
  const box = () => ({ x: 0, y: 0, left: 0, top: 0, right: 1, bottom: 1, width: 1, height: 1 });
  const node = () => ({
    nodeType: 1,
    style: {},
    clientWidth: 1,
    clientHeight: 1,
    appendChild() {},
    removeChild() {},
    addEventListener() {},
    removeEventListener() {},
    getBoundingClientRect: box,
  });
  const decorate = (canvas: OffscreenCanvas) =>
    Object.assign(canvas, {
      style: {},
      getBoundingClientRect: () => ({
        ...box(),
        width: canvas.width,
        height: canvas.height,
        right: canvas.width,
        bottom: canvas.height,
      }),
      parentElement: null,
      parentNode: null,
      ownerDocument: scope.document,
    });
  scope.window = globalThis;
  scope.screen = { orientation: { addEventListener() {}, removeEventListener() {} } };
  scope.HTMLCanvasElement = OffscreenCanvas;
  scope.HTMLVideoElement = class {};
  scope.document = {
    readyState: "complete",
    body: node(),
    documentElement: node(),
    hidden: false,
    addEventListener() {},
    removeEventListener() {},
    createElement: (tag: string) =>
      tag === "canvas" ? decorate(new OffscreenCanvas(1, 1)) : node(),
    getElementById: () => null,
  };
  // Phaser boots its built-in default/white textures through Image.src. The
  // application sprite path remains fetch -> ImageBitmap -> direct GPU upload.
  scope.Image = function () {
    const canvas = decorate(new OffscreenCanvas(1, 1)) as ReturnType<typeof decorate> & {
      onload?: () => void;
      onerror?: (error: unknown) => void;
    };
    Object.defineProperty(canvas, "src", {
      set(url: string) {
        void fetch(url)
          .then((response) => {
            if (!response.ok)
              throw new Error(`Phaser built-in texture failed (${response.status})`);
            return response.blob();
          })
          .then((blob) => createImageBitmap(blob))
          .then((image) => {
            try {
              canvas.width = image.width;
              canvas.height = image.height;
              canvas.getContext("2d")!.drawImage(image, 0, 0);
            } finally {
              image.close();
            }
            canvas.onload?.();
          })
          .catch((error) => canvas.onerror?.(error));
      },
    });
    return canvas;
  };
  scope.HTMLImageElement = scope.Image;
  return decorate;
}
