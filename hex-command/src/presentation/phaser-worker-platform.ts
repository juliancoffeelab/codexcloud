/** Phaser 3.90 customEnvironment host for a module worker.
 * Input, audio, DOM UI and browser scale management are disabled in game config.
 * Canvas/Image support is deliberately limited to native OffscreenCanvas and
 * Phaser's built-in textures; application artwork still uses fetch/ImageBitmap.
 */
export function installPhaserWorkerPlatform() {
  const scope = globalThis as any;
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
  const decorate = (canvas: any) =>
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
  scope.window = scope;
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
  scope.Image = function () {
    const canvas: any = decorate(new OffscreenCanvas(1, 1));
    Object.defineProperty(canvas, "src", {
      set(url: string) {
        fetch(url)
          .then((r) => r.blob())
          .then((b) => createImageBitmap(b))
          .then((image) => {
            canvas.width = image.width;
            canvas.height = image.height;
            canvas.getContext("2d").drawImage(image, 0, 0);
            image.close();
            canvas.onload?.();
          })
          .catch((e) => canvas.onerror?.(e));
      },
    });
    return canvas;
  };
  scope.HTMLImageElement = scope.Image;
  return decorate;
}
