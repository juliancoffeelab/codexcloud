import Phaser from "phaser";
import { traceInstant } from "./performance-trace";

/** Device limits are allocation constraints, never an estimate of free VRAM. */
export class BattlefieldResources {
  readonly texture: number;
  readonly target: number;
  readonly limits: Record<string, number | number[]>;
  private owned = new Map<string, { kind: string; cpu: number; gpu: number }>();
  private cpu = 0;
  private gpu = 0;
  private peakCPU = 0;
  private peakGPU = 0;
  constructor(private game: Phaser.Game) {
    const gl = (game.renderer as Phaser.Renderer.WebGL.WebGLRenderer).gl;
    const viewport = Array.from(gl.getParameter(gl.MAX_VIEWPORT_DIMS) as Int32Array);
    this.texture = gl.getParameter(gl.MAX_TEXTURE_SIZE) as number;
    this.target = Math.min(
      this.texture,
      gl.getParameter(gl.MAX_RENDERBUFFER_SIZE) as number,
      ...viewport,
    );
    this.limits = {
      texture: this.texture,
      target: this.target,
      viewport,
      attributes: gl.getParameter(gl.MAX_VERTEX_ATTRIBS) as number,
      varyings: gl.getParameter(gl.MAX_VARYING_VECTORS) as number,
      textures: gl.getParameter(gl.MAX_TEXTURE_IMAGE_UNITS) as number,
    };
    if (
      Number(this.limits.attributes) < 7 ||
      Number(this.limits.varyings) < 6 ||
      Number(this.limits.textures) < 3
    )
      throw new Error(`Unsupported battlefield WebGL limits: ${JSON.stringify(this.limits)}`);
    traceInstant("Battlefield device limits", this.limits);
  }
  area(label: string, width: number, height: number) {
    if (
      !Number.isSafeInteger(width) ||
      !Number.isSafeInteger(height) ||
      width <= 0 ||
      height <= 0 ||
      !Number.isSafeInteger(width * height * 28)
    )
      throw new Error(`Invalid battlefield dimensions ${label}: ${width}×${height}`);
    return width * height;
  }
  check(label: string, width: number, height: number, target = false) {
    this.area(label, width, height);
    const limit = target ? this.target : this.texture;
    if (width > limit || height > limit)
      throw new Error(
        `Unsupported battlefield allocation ${label}: ${width}×${height}; device limit ${limit}`,
      );
    return width * height;
  }
  image(key: string, image: HTMLImageElement) {
    const area = this.check(key, image.naturalWidth, image.naturalHeight);
    this.set(key, "decoded image + source texture", area * 4, area * 4);
  }
  set(key: string, kind: string, cpu: number, gpu: number) {
    const old = this.owned.get(key);
    this.cpu += cpu - (old?.cpu ?? 0);
    this.gpu += gpu - (old?.gpu ?? 0);
    this.peakCPU = Math.max(this.peakCPU, this.cpu);
    this.peakGPU = Math.max(this.peakGPU, this.gpu);
    this.owned.set(key, { kind, cpu, gpu });
  }
  remove(key: string) {
    const old = this.owned.get(key);
    if (old) {
      this.cpu -= old.cpu;
      this.gpu -= old.gpu;
    }
    this.owned.delete(key);
  }
  snapshot() {
    const renderer = this.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer;
    this.set(
      "main-color",
      "main drawing buffer",
      0,
      this.game.canvas.width * this.game.canvas.height * 4,
    );
    const capture = renderer.renderTarget;
    if (capture)
      this.set(
        "native-capture",
        "native Phaser capture target",
        0,
        capture.width * capture.height * (capture.hasDepthBuffer ? 8 : 4),
      );
    const groups: Record<string, { cpu: number; gpu: number; count: number }> = {};
    for (const value of this.owned.values()) {
      const group = (groups[value.kind] ??= { cpu: 0, gpu: 0, count: 0 });
      group.cpu += value.cpu;
      group.gpu += value.gpu;
      group.count++;
    }
    return {
      groups,
      totals: { cpu: this.cpu, gpu: this.gpu },
      peaks: { cpu: this.peakCPU, gpu: this.peakGPU },
    };
  }
  publish() {
    traceInstant("Battlefield owned resource estimates", this.snapshot());
  }
}
