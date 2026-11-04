import { workerTraceStart, workerTraceEnd } from "./worker-trace";
import {
  daylight,
  lightingKey,
  lightPosition,
  type BattlefieldLighting,
} from "./battlefield-lighting";
import { terrainDrawingOrder, terrainMapDimensions, type TerrainMapPlan } from "./terrain-painter";
import { battlefieldSpriteSize, hexColumnSpacing } from "../ui/geometry";
import * as shaders from "./map-shaders";
import { lightGrid } from "./light-grid";

type Target = {
  texture: WebGLTexture;
  framebuffer: WebGLFramebuffer;
  width: number;
  height: number;
};
type Region = { left: number; top: number; right: number; bottom: number };

type Program = { program: WebGLProgram; uniforms: Record<string, WebGLUniformLocation | null> };

const programDefinitions = {
  sprite: [shaders.quadVertex, shaders.spriteFragment, ["size", "rect", "image", "brightness"]],
  copy: [shaders.fullscreenVertex, shaders.copyFragment, ["terrain", "straight"]],
  lighting: [
    shaders.fullscreenVertex,
    shaders.lightingFragment,
    ["size", "terrain", "lights", "straight", "scale", "lightCells", "lightIndices", "night"],
  ],
  solid: [shaders.solidVertex, shaders.solidFragment, ["size", "rect", "ink"]],
  halo: [shaders.quadVertex, shaders.haloFragment, ["size", "rect", "glow", "fixture"]],
  mask: [shaders.fogVertex, shaders.maskFragment, ["size"]],
  blur: [
    shaders.fullscreenVertex,
    shaders.blurFragment,
    ["size", "image", "stepSize", "weights[0]", "offsets[0]"],
  ],
  composite: [
    shaders.fullscreenVertex,
    shaders.compositeFragment,
    ["size", "image", "fog", "hasFog"],
  ],
} as const;
type ProgramName = keyof typeof programDefinitions;
type PendingProgram = { program: WebGLProgram; fragment: WebGLShader };

/** Native terrain/light/fog passes invoked by the Phaser scene on the same worker/context. */
export class MapRendererGpu {
  private gl: WebGL2RenderingContext;
  private programs: Partial<Record<ProgramName, Program>> = {};
  private pendingPrograms = new Map<ProgramName, PendingProgram>();
  private vertexShaders = new Map<string, WebGLShader>();
  private parallelCompile: { COMPLETION_STATUS_KHR: number } | null;
  private straightTerrain = false;
  private terrain!: Target;
  private lit!: Target;
  private compositionInvalid = true;
  private previousLighting?: BattlefieldLighting;
  private previousFog?: ReadonlySet<string>;
  private tileCenters = new Map<string, { x: number; y: number }>();
  private fogVertices = new Map<string, Float32Array>();
  private lightCapacity = 0;
  private fogMask?: Target;
  private fogScratch?: Target;
  private fog?: Target;
  private lights: WebGLTexture;
  private lightCells: WebGLTexture;
  private lightIndices: WebGLTexture;
  private emptyFog: WebGLTexture;
  private fogBuffer: WebGLBuffer;
  private fogVao: WebGLVertexArrayObject;
  private maxTextureSize: number;
  private lightKey?: string;
  private fogKey?: string;
  private hasFog = false;
  private scale = 1;
  private width = 1;
  private height = 1;
  private presentationSize?: { width: number; height: number };

  constructor(readonly canvas: OffscreenCanvas = new OffscreenCanvas(1, 1)) {
    const gl = this.canvas.getContext("webgl2", {
      alpha: true,
      premultipliedAlpha: true,
      antialias: true,
      depth: false,
      stencil: false,
      // This canvas renders on demand, so its visible pixels must survive idle
      // tasks, scrolling and recoverable load failures between updates.
      preserveDrawingBuffer: true,
    });
    if (!gl)
      throw new Error("Map rendering requires WebGL 2. Enable browser graphics acceleration.");
    this.gl = gl;
    this.maxTextureSize = gl.getParameter(gl.MAX_TEXTURE_SIZE) as number;
    gl.disable(gl.DITHER);
    this.parallelCompile = gl.getExtension("KHR_parallel_shader_compile");
    this.lights = this.texture();
    this.lightCells = this.texture();
    this.lightIndices = this.texture();
    this.emptyFog = this.texture(true);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, 1, 1, 0, gl.RED, gl.UNSIGNED_BYTE, new Uint8Array(1));
    this.fogBuffer = gl.createBuffer()!;
    this.fogVao = gl.createVertexArray()!;
    gl.bindVertexArray(this.fogVao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.fogBuffer);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    gl.bindVertexArray(null);
    this.check();
  }

  setPresentationSize(size: { width: number; height: number }) {
    this.presentationSize = size;
    this.resizePresentation();
  }
  private resizePresentation() {
    const gl = this.gl;
    const requested = this.presentationSize ?? { width: this.width, height: this.height };
    const viewport = gl.getParameter(gl.MAX_VIEWPORT_DIMS) as Int32Array;
    const limit = gl.getParameter(gl.MAX_RENDERBUFFER_SIZE) as number;
    // Bound the multisampled display buffer; native lighting/fog textures do not grow.
    const factor = Math.min(
      1,
      Math.sqrt(4_194_304 / (requested.width * requested.height)),
      Math.min(limit, viewport[0]!) / requested.width,
      Math.min(limit, viewport[1]!) / requested.height,
    );
    const width = Math.max(1, Math.floor(requested.width * factor)),
      height = Math.max(1, Math.floor(requested.height * factor));
    if (this.canvas.width === width && this.canvas.height === height) return;
    this.canvas.width = width;
    this.canvas.height = height;
    this.compositionInvalid = true;
  }

  get context() {
    return this.gl;
  }

  get dimensions() {
    return { width: this.width / this.scale, height: this.height / this.scale };
  }
  get statistics() {
    return {
      nativeWidth: this.width,
      nativeHeight: this.height,
      displayWidth: this.canvas.width,
      displayHeight: this.canvas.height,
    };
  }

  private requestProgram(name: ProgramName) {
    if (this.programs[name] || this.pendingPrograms.has(name)) return;
    const gl = this.gl,
      [vertex, fragment] = programDefinitions[name];
    let vs = this.vertexShaders.get(vertex);
    if (!vs) {
      vs = gl.createShader(gl.VERTEX_SHADER)!;
      gl.shaderSource(vs, vertex);
      gl.compileShader(vs);
      this.vertexShaders.set(vertex, vs);
    }
    const fs = gl.createShader(gl.FRAGMENT_SHADER)!;
    gl.shaderSource(fs, fragment);
    gl.compileShader(fs);
    const program = gl.createProgram()!;
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    this.pendingPrograms.set(name, { program, fragment: fs });
  }

  private finishProgram(name: ProgramName): Program {
    if (this.programs[name]) return this.programs[name]!;
    const gl = this.gl,
      pending = this.pendingPrograms.get(name)!;
    if (!gl.getProgramParameter(pending.program, gl.LINK_STATUS)) {
      const error = gl.getProgramInfoLog(pending.program) || gl.getShaderInfoLog(pending.fragment);
      throw new Error(`Map GPU program failed (${name}): ${error}`);
    }
    gl.deleteShader(pending.fragment);
    const program = {
      program: pending.program,
      uniforms: Object.fromEntries(
        programDefinitions[name][2].map((key) => [
          key,
          gl.getUniformLocation(pending.program, key),
        ]),
      ),
    };
    this.programs[name] = program;
    this.pendingPrograms.delete(name);
    if (name === "blur") {
      const kernel = Array.from({ length: 13 }, (_, i) => Math.exp(-(i * i) / 32));
      const sum = kernel[0]! + 2 * kernel.slice(1).reduce((a, b) => a + b, 0);
      const weights = [kernel[0]! / sum],
        offsets = [0];
      for (let i = 1; i < 13; i += 2) {
        const weight = kernel[i]! + kernel[i + 1]!;
        weights.push(weight / sum);
        offsets.push(i + kernel[i + 1]! / weight);
      }
      gl.useProgram(program.program);
      gl.uniform1fv(program.uniforms["weights[0]"]!, weights);
      gl.uniform1fv(program.uniforms["offsets[0]"]!, offsets);
    }
    return program;
  }

  /** Queue independent compilations together, overlapping image fetching/decoding. */
  async prepare(lighting: BattlefieldLighting = daylight, hasFog: boolean, prebuilt: boolean) {
    const names: ProgramName[] = ["composite", lighting.time === "day" ? "copy" : "lighting"];
    // Night fixtures are prepared even before activation so the first blood light
    // or lantern does not pay a new shader compilation cost during play.
    if (lighting.time === "night") names.push("solid", "halo");
    if (hasFog) names.push("mask", "blur");
    if (!prebuilt) names.push("sprite");
    for (const name of names) this.requestProgram(name);
    if (this.parallelCompile) {
      const completion = this.parallelCompile.COMPLETION_STATUS_KHR;
      while (
        !names.every(
          (name) =>
            this.programs[name] ||
            this.gl.getProgramParameter(this.pendingPrograms.get(name)!.program, completion),
        )
      ) {
        if (this.gl.isContextLost())
          throw new Error("Map GPU context was lost. Retry map rendering.");
        await new Promise((resolve) => setTimeout(resolve, 1));
      }
    }
    for (const name of names) this.finishProgram(name);
  }

  private use(name: ProgramName) {
    this.requestProgram(name);
    const program = this.finishProgram(name);
    this.gl.useProgram(program.program);
    return program.uniforms;
  }

  private texture(linear = false) {
    const gl = this.gl,
      texture = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, linear ? gl.LINEAR : gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, linear ? gl.LINEAR : gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return texture;
  }

  private target(width: number, height: number, mask = false): Target {
    const gl = this.gl,
      texture = this.texture(mask);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      mask ? gl.R8 : gl.RGBA8,
      width,
      height,
      0,
      mask ? gl.RED : gl.RGBA,
      gl.UNSIGNED_BYTE,
      null,
    );
    const framebuffer = gl.createFramebuffer()!;
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE)
      throw new Error("Map GPU render target is unavailable");
    return { texture, framebuffer, width, height };
  }

  private bind(target: Target | null) {
    const gl = this.gl;
    gl.bindFramebuffer(gl.FRAMEBUFFER, target?.framebuffer ?? null);
    gl.viewport(0, 0, target?.width ?? this.canvas.width, target?.height ?? this.canvas.height);
  }

  private sample(texture: WebGLTexture, unit: number) {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
  }

  private clear() {
    this.gl.clearColor(0, 0, 0, 0);
    this.gl.clear(this.gl.COLOR_BUFFER_BIT);
  }

  private quad(uniforms: Program["uniforms"], x: number, y: number, width: number, height: number) {
    const gl = this.gl;
    gl.uniform2f(uniforms.size!, this.width, this.height);
    gl.uniform4f(uniforms.rect!, x, y, width, height);
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  }

  /** Upload complete images only; terrain composition uses authored anchors and depth. */
  setTerrain(
    plan: TerrainMapPlan,
    images: ReadonlyMap<string, ImageBitmap>,
    prebuilt?: ImageBitmap,
  ) {
    const gl = this.gl,
      { width, height } = terrainMapDimensions(plan);
    if (gl.isContextLost()) throw new Error("Map GPU context was lost. Retry map rendering.");
    const viewport = gl.getParameter(gl.MAX_VIEWPORT_DIMS) as Int32Array;
    if (
      width > Math.min(this.maxTextureSize, viewport[0]!) ||
      height > Math.min(this.maxTextureSize, viewport[1]!)
    )
      throw new Error("Map exceeds this GPU's texture or viewport limits");
    if (!this.terrain || this.width !== width || this.height !== height) {
      for (const target of [this.terrain, this.lit, this.fogMask, this.fogScratch, this.fog]) {
        if (target) {
          gl.deleteTexture(target.texture);
          gl.deleteFramebuffer(target.framebuffer);
        }
      }
      this.width = width;
      this.height = height;
      this.terrain = this.target(width, height);
      this.lit = this.target(width, height);
      this.fogMask = this.fogScratch = this.fog = undefined;
    }
    this.resizePresentation();
    this.tileCenters = new Map(plan.tiles.map((tile) => [tile.id, { x: tile.x, y: tile.y }]));
    this.straightTerrain = !!prebuilt;
    this.scale = plan.pixelScale;
    this.compositionInvalid = true;
    this.previousLighting = undefined;
    this.previousFog = undefined;
    this.fogVertices.clear();
    this.lightKey = undefined;
    this.fogKey = undefined;
    gl.activeTexture(gl.TEXTURE0);
    if (prebuilt) {
      if (prebuilt.width !== width || prebuilt.height !== height)
        throw new Error("Prebuilt map dimensions do not match the native scene");
      gl.bindTexture(gl.TEXTURE_2D, this.terrain.texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, prebuilt);
      this.check();
      return;
    }
    const textures = new Map<string | ImageBitmap, WebGLTexture>();
    try {
      for (const [key, image] of images) {
        const texture = this.texture();
        textures.set(key, texture);
        if (image.width > this.maxTextureSize || image.height > this.maxTextureSize)
          throw new Error("Terrain artwork exceeds this GPU's texture limit");
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
      }
      this.bind(this.terrain);
      this.clear();
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
      const u = this.use("sprite");
      gl.uniform1i(u.image!, 0);
      for (const tile of terrainDrawingOrder(plan.tiles)) {
        const image = tile.terrainFrame && textures.get(tile.terrainFrame.url);
        if (!image) throw new Error("Missing complete terrain frame");
        this.sample(image, 0);
        gl.uniform1f(u.brightness!, tile.role === "forest" ? 0.93 : 1);
        this.quad(
          u,
          (tile.spriteX ?? tile.x - tile.spriteSize / 2) * this.scale,
          (tile.spriteY ?? tile.y - tile.spriteSize / 2) * this.scale,
          (tile.spriteWidth ?? tile.spriteSize) * this.scale,
          (tile.spriteHeight ?? tile.spriteSize) * this.scale,
        );
      }
      gl.disable(gl.BLEND);
      this.check();
    } finally {
      for (const texture of textures.values()) gl.deleteTexture(texture);
    }
  }

  private fullRegion(): Region {
    return { left: 0, top: 0, right: this.width, bottom: this.height };
  }

  private union(a: Region | null, b: Region | null): Region | null {
    if (!a) return b;
    if (!b) return a;
    return {
      left: Math.min(a.left, b.left),
      top: Math.min(a.top, b.top),
      right: Math.max(a.right, b.right),
      bottom: Math.max(a.bottom, b.bottom),
    };
  }

  private expand(region: Region, x: number, y = x): Region {
    return {
      left: region.left - x,
      top: region.top - y,
      right: region.right + x,
      bottom: region.bottom + y,
    };
  }

  /** Region coordinates are native map pixels, independent of render-target size. */
  private scissor(region: Region, target: Pick<Target, "width" | "height">) {
    const gl = this.gl,
      sx = target.width / this.width,
      sy = target.height / this.height;
    const left = Math.max(0, Math.floor(region.left * sx)),
      top = Math.max(0, Math.floor(region.top * sy));
    const right = Math.min(target.width, Math.ceil(region.right * sx)),
      bottom = Math.min(target.height, Math.ceil(region.bottom * sy));
    gl.enable(gl.SCISSOR_TEST);
    gl.scissor(left, target.height - bottom, Math.max(0, right - left), Math.max(0, bottom - top));
  }

  private lightingRegion(lighting: BattlefieldLighting): Region | null {
    const previous = this.previousLighting;
    if (!previous || previous.time !== lighting.time) return this.fullRegion();
    if (lighting.time === "day") return null;
    const before = previous.lights ?? [],
      after = lighting.lights ?? [];
    let region: Region | null = null;
    for (let i = 0; i < Math.max(before.length, after.length); i++) {
      const old = before[i],
        next = after[i];
      if (
        lightingKey({ time: "night", lights: old ? [old] : [] }) ===
        lightingKey({ time: "night", lights: next ? [next] : [] })
      )
        continue;
      for (const light of [old, next]) {
        if (!light) continue;
        const { x, y } = lightPosition(light);
        // Include both the lighting ellipse and every fixture/halo pixel.
        const radius = Math.max(light.radius, 27);
        region = this.union(region, {
          left: (x - radius) * this.scale - 2,
          top: (y - radius) * this.scale - 2,
          right: (x + radius) * this.scale + 2,
          bottom: (y + radius) * this.scale + 2,
        });
      }
    }
    return region;
  }

  private uploadLights(lighting: BattlefieldLighting) {
    const gl = this.gl,
      lights = lighting.lights ?? [];
    if (lights.length > this.maxTextureSize)
      throw new Error("Map has too many lights for this GPU");
    const table = new Float32Array(Math.max(1, lights.length) * 12);
    lights.forEach((light, index) => {
      const { x, y } = lightPosition(light),
        color = light.color?.map((v) => v / 255);
      table.set(
        [
          x,
          y,
          light.radius,
          light.power,
          ...(color ? color.map((v) => v * 1.06) : [1.02, 0.78, 0.49]),
          0,
          ...(color ? color.map((v) => v * 30) : [22, 14, 7]),
          0,
        ],
        index * 12,
      );
    });
    this.sample(this.lights, 1);
    if (Math.max(1, lights.length) > this.lightCapacity) {
      this.lightCapacity = Math.min(
        this.maxTextureSize,
        Math.max(16, 2 ** Math.ceil(Math.log2(Math.max(1, lights.length)))),
      );
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        gl.RGBA32F,
        3,
        this.lightCapacity,
        0,
        gl.RGBA,
        gl.FLOAT,
        null,
      );
    }
    gl.texSubImage2D(
      gl.TEXTURE_2D,
      0,
      0,
      0,
      3,
      Math.max(1, lights.length),
      gl.RGBA,
      gl.FLOAT,
      table,
    );
    const grid = lightGrid(lighting, this.width, this.height, this.scale);
    this.sample(this.lightCells, 2);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RG32UI,
      grid.cols,
      grid.rows,
      0,
      gl.RG_INTEGER,
      gl.UNSIGNED_INT,
      grid.ranges,
    );
    const width = Math.min(this.maxTextureSize, grid.indices.length);
    const height = Math.ceil(grid.indices.length / width);
    if (height > this.maxTextureSize)
      throw new Error("Map light index exceeds this GPU's texture limit");
    const indices = new Uint32Array(width * height);
    indices.set(grid.indices);
    this.sample(this.lightIndices, 3);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.R32UI,
      width,
      height,
      0,
      gl.RED_INTEGER,
      gl.UNSIGNED_INT,
      indices,
    );
  }

  configureLighting(u: Program["uniforms"], lighting: BattlefieldLighting, unit = 1) {
    const gl = this.gl;
    gl.uniform1i(u.night!, Number(lighting.time === "night"));
    gl.uniform1f(u.scale!, this.scale);
    this.sample(this.lights, unit);
    this.sample(this.lightCells, unit + 1);
    this.sample(this.lightIndices, unit + 2);
    gl.uniform1i(u.lights!, unit);
    gl.uniform1i(u.lightCells!, unit + 1);
    gl.uniform1i(u.lightIndices!, unit + 2);
  }

  private shade(lighting: BattlefieldLighting, region: Region) {
    const gl = this.gl,
      lights = lighting.lights ?? [];
    if (lighting.time === "day") {
      this.bind(this.lit);
      this.scissor(region, this.lit);
      const u = this.use("copy");
      this.sample(this.terrain.texture, 0);
      gl.uniform1i(u.terrain!, 0);
      gl.uniform1i(u.straight!, Number(this.straightTerrain));
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      gl.disable(gl.SCISSOR_TEST);
      return;
    }
    this.bind(this.lit);
    this.scissor(region, this.lit);
    const u = this.use("lighting");
    this.sample(this.terrain.texture, 0);
    gl.uniform1i(u.terrain!, 0);
    gl.uniform1i(u.straight!, Number(this.straightTerrain));
    gl.uniform2f(u.size!, this.width, this.height);
    this.configureLighting(u, lighting);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.enable(gl.BLEND);
    for (const light of lights) {
      const { x, y } = lightPosition(light),
        scale = this.scale;
      const fixtureRadius = light.fixture === false ? 25 : 20;
      const haloY = y - (light.fixture === false ? 0 : 5);
      if (
        (x + fixtureRadius) * scale + 1 < region.left ||
        (x - fixtureRadius) * scale - 1 > region.right ||
        (haloY + fixtureRadius) * scale + 1 < region.top ||
        (haloY - fixtureRadius) * scale - 1 > region.bottom
      )
        continue;
      if (light.fixture !== false) {
        gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
        const solid = this.use("solid");
        for (const [dx, dy, w, h, r, g, b] of [
          [-3, -9, 6, 12, 73, 54, 31],
          [-2, -7, 4, 5, 246, 170, 70],
          [-1, -6, 2, 3, 255, 240, 176],
          [-1, 3, 2, 8, 80, 64, 46],
        ]) {
          gl.uniform4f(solid.ink!, r! / 255, g! / 255, b! / 255, 1);
          this.quad(solid, (x + dx!) * scale, (y + dy!) * scale, w! * scale, h! * scale);
        }
      }
      // Screen composition in premultiplied RGBA, including transparent map edges.
      gl.blendFuncSeparate(gl.ONE, gl.ONE_MINUS_SRC_COLOR, gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
      const halo = this.use("halo"),
        fixture = light.fixture !== false,
        radius = fixture ? 20 : 25;
      const color = light.color ?? [255, 170, 70];
      gl.uniform1i(halo.fixture!, Number(fixture));
      gl.uniform3f(halo.glow!, color[0]! / 255, color[1]! / 255, color[2]! / 255);
      this.quad(
        halo,
        (x - radius) * scale,
        (y - (fixture ? 5 : 0) - radius) * scale,
        radius * 2 * scale,
        radius * 2 * scale,
      );
    }
    gl.disable(gl.BLEND);
    gl.disable(gl.SCISSOR_TEST);
  }

  private paintFog(ids: readonly string[]): Region | null {
    const gl = this.gl,
      previous = this.previousFog,
      next = new Set(ids);
    this.hasFog = next.size > 0;
    if (!this.hasFog) {
      this.previousFog = undefined;
      return previous?.size ? this.fullRegion() : null;
    }
    if (!this.fog) {
      const width = Math.ceil(this.width / 2),
        height = Math.ceil(this.height / 2);
      this.fogMask = this.target(width, height, true);
      this.fogScratch = this.target(width, height, true);
      this.fog = this.target(width, height, true);
    }
    const maskTarget = this.fogMask!,
      scratchTarget = this.fogScratch!,
      fogTarget = this.fog;
    let changed: Region | null = previous ? null : this.fullRegion();
    if (previous) {
      const delta = [...next]
        .filter((id) => !previous.has(id))
        .concat([...previous].filter((id) => !next.has(id)));
      for (const id of delta) {
        const point = this.tileCenters.get(id);
        if (!point) throw new Error(`Unknown fog tile ${id}`);
        changed = this.union(changed, {
          left: (point.x - hexColumnSpacing / 2) * this.scale - 2,
          top: (point.y - battlefieldSpriteSize / 2) * this.scale - 2,
          right: (point.x + hexColumnSpacing / 2) * this.scale + 2,
          bottom: (point.y + battlefieldSpriteSize / 2) * this.scale + 2,
        });
      }
    }
    this.previousFog = next;
    if (!changed) return null;
    const vertices = new Float32Array(next.size * 36);
    let offset = 0;
    for (const id of next) {
      let hex = this.fogVertices.get(id);
      if (!hex) {
        const center = this.tileCenters.get(id);
        if (!center) throw new Error(`Unknown fog tile ${id}`);
        hex = new Float32Array(36);
        let at = 0;
        for (let i = 0; i < 6; i++) {
          hex.set([center.x, center.y], at);
          at += 2;
          for (const edge of [i, i + 1]) {
            const angle = ((60 * edge - 30) * Math.PI) / 180;
            hex.set(
              [
                center.x + (hexColumnSpacing / Math.sqrt(3)) * Math.cos(angle),
                center.y + (battlefieldSpriteSize / 2) * Math.sin(angle),
              ],
              at,
            );
            at += 2;
          }
        }
        this.fogVertices.set(id, hex);
      }
      vertices.set(hex, offset);
      offset += hex.length;
    }
    this.bind(maskTarget);
    this.scissor(changed, maskTarget);
    this.clear();
    const mask = this.use("mask");
    gl.uniform2f(mask.size!, this.width / this.scale, this.height / this.scale);
    gl.bindVertexArray(this.fogVao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.fogBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, vertices, gl.DYNAMIC_DRAW);
    gl.drawArrays(gl.TRIANGLES, 0, vertices.length / 2);
    gl.bindVertexArray(null);
    const u = this.use("blur");
    gl.uniform1i(u.image!, 0);
    gl.uniform2f(u.size!, fogTarget.width, fogTarget.height);
    // Each paired Gaussian kernel reaches at most 13 half-resolution texels.
    const horizontal = this.expand(changed, (13 * this.width) / fogTarget.width, 0);
    this.bind(scratchTarget);
    this.scissor(horizontal, scratchTarget);
    this.sample(maskTarget.texture, 0);
    gl.uniform2f(u.stepSize!, 1 / fogTarget.width, 0);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    const vertical = this.expand(horizontal, 0, (13 * this.height) / fogTarget.height);
    this.bind(fogTarget);
    this.scissor(vertical, fogTarget);
    this.sample(scratchTarget.texture, 0);
    gl.uniform2f(u.stepSize!, 0, 1 / fogTarget.height);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.disable(gl.SCISSOR_TEST);
    return vertical;
  }

  private compose(region: Region) {
    const gl = this.gl;
    // Compose changed pixels straight into the retained visible drawing buffer.
    this.bind(null);
    this.scissor(region, this.canvas);
    const u = this.use("composite");
    this.sample(this.lit.texture, 0);
    this.sample(this.fog?.texture ?? this.emptyFog, 1);
    gl.uniform1i(u.image!, 0);
    gl.uniform1i(u.fog!, 1);
    gl.uniform1i(u.hasFog!, Number(this.hasFog));
    gl.uniform2f(u.size!, this.canvas.width, this.canvas.height);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    gl.disable(gl.SCISSOR_TEST);
  }

  private present() {
    // Automatic worker-canvas presentation: no full-map blit or bitmap export.
    this.gl.flush();
  }

  render(lighting: BattlefieldLighting = daylight, fog: readonly string[]) {
    const gl = this.gl;
    if (gl.isContextLost()) throw new Error("Map GPU context was lost. Retry map rendering.");
    const lightKey = lightingKey(lighting),
      fogKey = JSON.stringify(fog);
    let dirty = this.compositionInvalid ? this.fullRegion() : null;
    if (this.lightKey !== lightKey) {
      const uploadStarted = workerTraceStart();
      this.uploadLights(lighting);
      workerTraceEnd("Light spatial index upload", uploadStarted, {
        lights: lighting.lights?.length ?? 0,
      });
      const region = this.lightingRegion(lighting);
      if (region) {
        const started = workerTraceStart();
        this.shade(lighting, region);
        workerTraceEnd("Terrain lighting", started, { ...region });
      }
      dirty = this.union(dirty, region);
      this.previousLighting = lighting;
      this.lightKey = lightKey;
    }
    if (this.fogKey !== fogKey) {
      const started = workerTraceStart();
      dirty = this.union(dirty, this.paintFog(fog));
      workerTraceEnd("Fog mask and blur", started, { tiles: fog.length });
      this.fogKey = fogKey;
    }
    // Phaser owns presentation. This Extern pass composites cached terrain/fog
    // behind its display list; light/fog caches above update only when changed.
    const composeStarted = workerTraceStart();
    this.compose(this.fullRegion());
    workerTraceEnd("Compose cached terrain", composeStarted, { cachesChanged: !!dirty });
    gl.bindVertexArray(null);
    this.compositionInvalid = false;
    return this.present();
  }

  private check() {
    const error = this.gl.getError();
    if (error !== this.gl.NO_ERROR) throw new Error(`Map GPU operation failed (${error})`);
  }
}
