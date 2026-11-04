import { hexColumnSpacing, battlefieldSpriteSize } from "../ui/geometry";
import type { BattlefieldScene, BattlefieldUnit } from "./battlefield-scene";
import type { MotionKind } from "./motion";
import { lightSampling } from "./map-shaders";

type Region = { left: number; top: number; right: number; bottom: number };
type Color = readonly [number, number, number, number];
type Batch = { page: number; values: number[]; data?: Float32Array };
type Sprite = { page: number; u: number; v: number };
type Transition = {
  from: BattlefieldUnit;
  to: BattlefieldUnit;
  start: number;
  duration: number;
  kind: "move" | "hit" | "spawn" | "death";
};
const atlasSize = 1024,
  cells = atlasSize / 64,
  stride = 11;
const vertex = `#version 300 es
layout(location=0) in vec2 point;
layout(location=1) in vec2 texcoord;
layout(location=2) in vec4 ink;
layout(location=3) in vec3 lighting;
uniform vec2 size;
out vec2 uv;
out vec4 tint;
out vec3 lightPoint;
void main() {
  gl_Position = vec4(point / size * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
  uv = texcoord; tint = ink; lightPoint = lighting;
}`;
const fragment = `#version 300 es
precision highp float;
uniform sampler2D image;
in vec2 uv;
in vec4 tint;
in vec3 lightPoint;
${lightSampling}
out vec4 color;
void main() {
  vec4 source = texture(image, uv) * tint;
  vec3 rgb = lightPoint.z > .5 ? shadeColor(source.rgb, lightPoint.xy) : source.rgb;
  color = vec4(rgb * source.a, source.a);
}`;

function rgba(hex: string, alpha = 1): Color {
  return [
    parseInt(hex.slice(1, 3), 16) / 255,
    parseInt(hex.slice(3, 5), 16) / 255,
    parseInt(hex.slice(5, 7), 16) / 255,
    alpha,
  ];
}
const white: Color = [1, 1, 1, 1];

/** One batched atlas pipeline for live battlefield visuals. No DOM sprite filters. */
export class BattlefieldOverlayGpu {
  readonly uniforms: Record<string, WebGLUniformLocation | null>;
  private program: WebGLProgram;
  private vao: WebGLVertexArrayObject;
  private buffer: WebGLBuffer;
  private pages: WebGLTexture[] = [];
  private sprites = new Map<string, Sprite>();
  private scene?: BattlefieldScene;
  private transitions = new Map<string, Transition>();
  private pulses = new Map<string, { kind: MotionKind; start: number }>();
  private invalid = false;
  private hovered: string | null = null;
  private focused: string | null = null;
  private changedInspection = new Set<string>();
  private batches: Batch[] = [];
  private staticBatches?: Batch[];
  private light: readonly number[] = [0, 0, 0];
  private atlas = 0;

  constructor(private gl: WebGL2RenderingContext) {
    const shaders = [vertex, fragment].map((source, i) => {
      const shader = gl.createShader(i === 0 ? gl.VERTEX_SHADER : gl.FRAGMENT_SHADER)!;
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      return shader;
    });
    this.program = gl.createProgram()!;
    for (const shader of shaders) gl.attachShader(this.program, shader);
    gl.linkProgram(this.program);
    if (!gl.getProgramParameter(this.program, gl.LINK_STATUS))
      throw new Error(`Battlefield sprite shader failed: ${gl.getProgramInfoLog(this.program)}`);
    for (const shader of shaders) gl.deleteShader(shader);
    this.uniforms = Object.fromEntries(
      ["size", "image", "lights", "lightCells", "lightIndices", "night", "scale"].map((name) => [
        name,
        gl.getUniformLocation(this.program, name),
      ]),
    );
    this.vao = gl.createVertexArray()!;
    this.buffer = gl.createBuffer()!;
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
    let offset = 0;
    for (const [index, count] of [2, 2, 4, 3].entries()) {
      gl.enableVertexAttribArray(index);
      gl.vertexAttribPointer(index, count, gl.FLOAT, false, stride * 4, offset * 4);
      offset += count;
    }
    gl.bindVertexArray(null);
    this.addPage();
  }

  private addPage() {
    const gl = this.gl,
      texture = gl.createTexture()!;
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA8,
      atlasSize,
      atlasSize,
      0,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      null,
    );
    gl.texSubImage2D(
      gl.TEXTURE_2D,
      0,
      0,
      0,
      1,
      1,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      new Uint8Array([255, 255, 255, 255]),
    );
    this.pages.push(texture);
  }

  resetImages() {
    for (const texture of this.pages) this.gl.deleteTexture(texture);
    this.pages = [];
    this.sprites.clear();
    this.staticBatches = undefined;
    this.addPage();
  }

  hasImage(url: string) {
    return this.sprites.has(url);
  }
  addImage(url: string, image: ImageBitmap) {
    if (this.sprites.has(url)) return;
    if (image.width !== 64 || image.height !== 64)
      throw new Error(`Battlefield sprites must be exactly 64×64: ${url}`);
    const index = this.sprites.size,
      page = Math.floor(index / (cells * cells - 1));
    if (page === this.pages.length) this.addPage();
    const slot = (index % (cells * cells - 1)) + 1,
      x = (slot % cells) * 64,
      y = Math.floor(slot / cells) * 64;
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.pages[page]!);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, x, y, gl.RGBA, gl.UNSIGNED_BYTE, image);
    this.sprites.set(url, { page, u: x / atlasSize, v: y / atlasSize });
  }

  setScene(scene: BattlefieldScene, now: number) {
    const previous = this.scene;
    if (previous?.key !== scene.key) {
      this.transitions.clear();
      this.pulses.clear();
      this.hovered = this.focused = null;
    } else if (!scene.reducedMotion) {
      const old = new Map(previous.units.map((unit) => [unit.id, unit]));
      for (const unit of scene.units) {
        const before = old.get(unit.id);
        old.delete(unit.id);
        if (before && (before.x !== unit.x || before.y !== unit.y)) {
          // Start from the current visual position when fast AI updates interrupt a tween.
          const current = this.position(before, now);
          this.transitions.set(unit.id, {
            from: { ...before, ...current },
            to: unit,
            start: now,
            duration: 240,
            kind: "move",
          });
        } else if (before && unit.hp < before.hp) {
          this.transitions.set(unit.id, {
            from: before,
            to: unit,
            start: now,
            duration: 260,
            kind: "hit",
          });
        } else if (!before)
          this.transitions.set(unit.id, {
            from: unit,
            to: unit,
            start: now,
            duration: 220,
            kind: "spawn",
          });
      }
      for (const unit of old.values()) {
        // Fog concealment is immediate; never leak a disappearing enemy through a death effect.
        if (!scene.fog.includes(unit.tile))
          this.transitions.set(unit.id, {
            from: unit,
            to: unit,
            start: now,
            duration: 300,
            kind: "death",
          });
        else this.transitions.delete(unit.id);
      }
    }
    if (scene.reducedMotion) {
      this.transitions.clear();
      this.pulses.clear();
    }
    this.staticBatches = undefined;
    this.scene = scene;
    this.invalid = true;
  }

  pulse(tile: string, kind: MotionKind, now: number) {
    if (this.scene?.reducedMotion) return;
    this.staticBatches = undefined;
    this.pulses.set(tile, { kind, start: now });
    this.invalid = true;
  }
  inspect(hovered: string | null, focused: string | null) {
    if (hovered === this.hovered && focused === this.focused) return;
    for (const id of [this.hovered, this.focused, hovered, focused])
      if (id) this.changedInspection.add(id);
    this.staticBatches = undefined;
    this.hovered = hovered;
    this.focused = focused;
  }
  get animating() {
    return this.transitions.size > 0 || this.pulses.size > 0;
  }
  dirty(now: number, scale: number, width: number, height: number): Region | null {
    if (!this.scene) return null;
    let result: Region | null = this.invalid
      ? { left: 0, top: 0, right: width, bottom: height }
      : null;
    const include = (x: number, y: number) => {
      const rect = {
        left: (x - 60) * scale,
        top: (y - 90) * scale,
        right: (x + 60) * scale,
        bottom: (y + 60) * scale,
      };
      result = result
        ? {
            left: Math.min(result.left, rect.left),
            top: Math.min(result.top, rect.top),
            right: Math.max(result.right, rect.right),
            bottom: Math.max(result.bottom, rect.bottom),
          }
        : rect;
    };
    for (const tile of this.scene.tiles)
      if (this.changedInspection.has(tile.id)) include(tile.x, tile.y);
    this.changedInspection.clear();
    for (const [id, transition] of this.transitions) {
      include(transition.from.x, transition.from.y);
      include(transition.to.x, transition.to.y);
      if (now >= transition.start + transition.duration) this.transitions.delete(id);
    }
    for (const [tile, pulse] of this.pulses) {
      const center = this.scene.tiles.find((t) => t.id === tile);
      if (center) include(center.x, center.y);
      if (now >= pulse.start + 300) this.pulses.delete(tile);
    }
    this.invalid = false;
    return result;
  }

  private position(unit: BattlefieldUnit, now: number) {
    const transition = this.transitions.get(unit.id);
    if (!transition || transition.kind !== "move") return { x: unit.x, y: unit.y };
    const t = Math.max(0, Math.min(1, (now - transition.start) / transition.duration));
    const eased = t * t * (3 - 2 * t);
    return {
      x: transition.from.x + (transition.to.x - transition.from.x) * eased,
      y:
        transition.from.y +
        (transition.to.y - transition.from.y) * eased -
        Math.sin(t * Math.PI) * 5,
    };
  }

  private point(x: number, y: number, color: Color, u = 0.5 / atlasSize, v = 0.5 / atlasSize) {
    let batch = this.batches[this.batches.length - 1];
    if (!batch || batch.page !== this.atlas) {
      batch = { page: this.atlas, values: [] };
      this.batches.push(batch);
    }
    batch.values.push(x, y, u, v, ...color, ...this.light);
  }
  private triangle(points: number[], color: Color) {
    this.atlas = 0;
    for (let i = 0; i < 6; i += 2) this.point(points[i]!, points[i + 1]!, color);
  }
  private line(x: number, y: number, x2: number, y2: number, width: number, color: Color) {
    const length = Math.hypot(x2 - x, y2 - y);
    if (!length) return;
    const dx = (((y2 - y) / length) * width) / 2,
      dy = (((x - x2) / length) * width) / 2;
    this.triangle([x + dx, y + dy, x2 + dx, y2 + dy, x2 - dx, y2 - dy], color);
    this.triangle([x + dx, y + dy, x2 - dx, y2 - dy, x - dx, y - dy], color);
  }
  private rect(x: number, y: number, w: number, h: number, color: Color) {
    this.triangle([x, y, x + w, y, x + w, y + h], color);
    this.triangle([x, y, x + w, y + h, x, y + h], color);
  }
  private ellipse(x: number, y: number, rx: number, ry: number, color: Color) {
    for (let i = 0; i < 20; i++) {
      const a = (i * Math.PI) / 10,
        b = ((i + 1) * Math.PI) / 10;
      this.triangle(
        [
          x,
          y,
          x + Math.cos(a) * rx,
          y + Math.sin(a) * ry,
          x + Math.cos(b) * rx,
          y + Math.sin(b) * ry,
        ],
        color,
      );
    }
  }
  private image(url: string | undefined, x: number, y: number, size: number, color = white) {
    if (!url) return;
    const sprite = this.sprites.get(url);
    if (!sprite) return;
    this.atlas = sprite.page;
    for (const [px, py] of [
      [0, 0],
      [1, 0],
      [1, 1],
      [0, 0],
      [1, 1],
      [0, 1],
    ])
      this.point(
        x + px! * size,
        y + py! * size,
        color,
        sprite.u + (px! * 64) / atlasSize,
        sprite.v + (py! * 64) / atlasSize,
      );
  }

  draw(
    now: number,
    width: number,
    height: number,
    configureLighting: (uniforms: typeof this.uniforms) => void,
  ) {
    const scene = this.scene;
    if (!scene) return;
    this.batches = [];
    this.light = [0, 0, 0];
    if (this.staticBatches) this.batches = [...this.staticBatches];
    else {
      for (const tile of scene.tiles) {
        const classes = new Set(tile.classes.split(" "));
        const focus = tile.id === this.focused,
          hover = tile.id === this.hovered;
        let color = rgba("#39472b", 0.3),
          thickness = 0.7;
        if (classes.has("selected") || focus) {
          color = rgba("#ffe5a1");
          thickness = 3;
        }
        if (classes.has("reachable")) {
          color = rgba("#95bfd2");
          thickness = 2.4;
        }
        if (classes.has("target")) {
          color = rgba("#ed8f82");
          thickness = 3;
        }
        if (classes.has("recruitable")) {
          color = rgba("#6fdec0");
          thickness = 3;
        }
        if (hover) {
          color = rgba("#ffe5a1");
          thickness = 2;
        }
        const pulse = this.pulses.get(tile.id);
        if (pulse?.kind === "select")
          thickness += Math.sin(((now - pulse.start) / 300) * Math.PI) * 2;
        for (let i = 0; i < 6; i++) {
          const a = ((60 * i - 30) * Math.PI) / 180,
            b = ((60 * (i + 1) - 30) * Math.PI) / 180;
          const x = tile.x + (hexColumnSpacing / Math.sqrt(3)) * Math.cos(a),
            y = tile.y + (battlefieldSpriteSize / 2) * Math.sin(a);
          const x2 = tile.x + (hexColumnSpacing / Math.sqrt(3)) * Math.cos(b),
            y2 = tile.y + (battlefieldSpriteSize / 2) * Math.sin(b);
          if (classes.has("objective") || classes.has("recruitable")) {
            // Explicit alternating segments retain objective/recruitment distinction without SVG dashes.
            for (let t = 0; t < 1; t += 0.25)
              this.line(
                x + (x2 - x) * t,
                y + (y2 - y) * t,
                x + (x2 - x) * Math.min(1, t + 0.15),
                y + (y2 - y) * Math.min(1, t + 0.15),
                7,
                rgba("#172015"),
              );
          }
          this.line(x, y, x2, y2, thickness, color);
        }
      }
      for (const item of scene.items) {
        this.light = [item.x, item.y, 1];
        this.ellipse(item.x, item.y + 14, 12, 5, rgba("#d5bc68", 0.4));
        if (item.url) this.image(item.url, item.x - 32, item.y - 32, 64);
        else {
          this.triangle(
            [item.x, item.y - 6, item.x + 8, item.y + 2, item.x, item.y + 10],
            rgba("#e2c778"),
          );
          this.triangle(
            [item.x, item.y - 6, item.x, item.y + 10, item.x - 8, item.y + 2],
            rgba("#e2c778"),
          );
        }
      }
      if (![...this.pulses.values()].some((pulse) => pulse.kind === "select"))
        this.staticBatches = [...this.batches];
    }
    // Never append dynamic vertices to cached static arrays.
    this.batches.push({ page: 0, values: [] });
    const units = [...scene.units];
    for (const transition of this.transitions.values())
      if (transition.kind === "death") units.push(transition.from);
    const indicators: { unit: BattlefieldUnit; x: number; y: number; alpha: number }[] = [];
    for (const unit of units) {
      const transition = this.transitions.get(unit.id),
        t = transition
          ? Math.max(0, Math.min(1, (now - transition.start) / transition.duration))
          : 1;
      let { x, y } = this.position(unit, now),
        alpha = 1;
      if (transition?.kind === "death") {
        alpha = 1 - t;
        y += t * 10;
      }
      if (transition?.kind === "spawn") {
        alpha = t;
        y -= (1 - t) * 10;
      }
      if (transition?.kind === "hit") x += Math.sin(t * Math.PI * 6) * (1 - t) * 4;
      const pulse = this.pulses.get(unit.tile);
      if (pulse?.kind === "attack") x += Math.sin(((now - pulse.start) / 300) * Math.PI) * 7;
      this.light = [0, 0, 0];
      this.ellipse(x, y + 19, 19, 7, rgba(unit.color, 0.5 * alpha));
      this.light = [x, y, 1];
      const flash = transition?.kind === "hit" ? Math.sin(t * Math.PI) * 0.5 : 0;
      this.image(unit.url, x - unit.size / 2, y + 20 - unit.size, unit.size, [
        1,
        1 - flash,
        1 - flash,
        (unit.spent ? 0.65 : 1) * alpha,
      ]);
      if (transition?.kind !== "death") indicators.push({ unit, x, y, alpha });
    }
    this.light = [0, 0, 0];
    for (const { unit, x, y, alpha } of indicators) {
      this.rect(x - 19, y + 27, 38, 4, rgba("#18211b", alpha));
      this.rect(x - 19, y + 27, Math.max(0, Math.min(38, unit.health)), 4, rgba("#a9d0a5", alpha));
      if (unit.spent) {
        this.line(x + 21, y - 20, x + 24, y - 17, 2, rgba("#dbe8d7", alpha));
        this.line(x + 24, y - 17, x + 30, y - 24, 2, rgba("#dbe8d7", alpha));
      }
      if (unit.retreating) {
        this.line(x - 22, y - 21, x - 30, y - 21, 2, rgba("#fff1c1", alpha));
        this.line(x - 30, y - 21, x - 26, y - 25, 2, rgba("#fff1c1", alpha));
        this.line(x - 30, y - 21, x - 26, y - 17, 2, rgba("#fff1c1", alpha));
      }
    }
    for (const tile of scene.tiles)
      if (tile.owner) {
        const pulse = this.pulses.get(tile.id),
          radius =
            pulse?.kind === "capture" ? 5 + Math.sin(((now - pulse.start) / 300) * Math.PI) * 3 : 5;
        this.ellipse(tile.x + 23, tile.y + 17, radius, radius, rgba(tile.owner));
      }
    const gl = this.gl;
    gl.useProgram(this.program);
    gl.uniform2f(this.uniforms.size!, width, height);
    gl.uniform1i(this.uniforms.image!, 0);
    configureLighting(this.uniforms);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    gl.bindVertexArray(this.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.buffer);
    for (const batch of this.batches) {
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, this.pages[batch.page]!);
      gl.bufferData(
        gl.ARRAY_BUFFER,
        (batch.data ??= new Float32Array(batch.values)),
        gl.DYNAMIC_DRAW,
      );
      gl.drawArrays(gl.TRIANGLES, 0, batch.values.length / stride);
    }
    gl.bindVertexArray(null);
    gl.disable(gl.BLEND);
  }
}
