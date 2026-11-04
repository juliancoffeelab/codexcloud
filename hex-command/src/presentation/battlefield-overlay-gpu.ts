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

const outlineVertex = `#version 300 es
layout(location=0) in vec2 point;
layout(location=1) in vec4 stroke;
layout(location=2) in vec4 ink;
uniform vec2 size;
out vec4 edge;
out vec4 tint;
void main() {
  gl_Position=vec4(point/size*vec2(2.0,-2.0)+vec2(-1.0,1.0),0.0,1.0);
  edge=stroke;tint=ink;
}`;
const outlineFragment = `#version 300 es
precision highp float;
in vec4 edge;
in vec4 tint;
out vec4 color;
void main() {
  float coverage=clamp(.5+(edge.y-abs(edge.x))/max(fwidth(edge.x),.00001),0.0,1.0);
  coverage*=clamp(.5+edge.z/max(fwidth(edge.z),.00001),0.0,1.0);
  coverage*=clamp(.5+edge.w/max(fwidth(edge.w),.00001),0.0,1.0);
  float alpha=tint.a*coverage;
  color=vec4(tint.rgb*alpha,alpha);
}`;
function program(gl: WebGL2RenderingContext, vertex: string, fragment: string) {
  const shaders = [vertex, fragment].map((source, index) => {
    const shader = gl.createShader(index === 0 ? gl.VERTEX_SHADER : gl.FRAGMENT_SHADER)!;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    return shader;
  });
  const result = gl.createProgram()!;
  for (const shader of shaders) gl.attachShader(result, shader);
  gl.linkProgram(result);
  if (!gl.getProgramParameter(result, gl.LINK_STATUS))
    throw new Error(`Battlefield shader failed: ${gl.getProgramInfoLog(result)}`);
  for (const shader of shaders) gl.deleteShader(shader);
  return result;
}

function rgba(hex: string, alpha = 1): Color {
  return [
    parseInt(hex.slice(1, 3), 16) / 255,
    parseInt(hex.slice(3, 5), 16) / 255,
    parseInt(hex.slice(5, 7), 16) / 255,
    alpha,
  ];
}
const white: Color = [1, 1, 1, 1];

/** Native artwork atlas plus a cached, analytic outline pass. */
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
  private focused: string | null = null;
  private changedInspection = new Set<string>();
  private batches: Batch[] = [];
  private staticBatches?: Batch[];
  private light: readonly number[] = [0, 0, 0];
  private atlas = 0;
  private outlineProgram: WebGLProgram;
  private outlineVao: WebGLVertexArrayObject;
  private outlineBuffer: WebGLBuffer;
  private outlineSize: WebGLUniformLocation | null;
  private outlineValues: number[] = [];
  private outlineData?: Float32Array;
  private uploadedOutline?: Float32Array;
  private fringe = 0;

  constructor(private gl: WebGL2RenderingContext) {
    this.program = program(gl, vertex, fragment);
    this.outlineProgram = program(gl, outlineVertex, outlineFragment);
    this.outlineSize = gl.getUniformLocation(this.outlineProgram, "size");
    this.outlineVao = gl.createVertexArray()!;
    this.outlineBuffer = gl.createBuffer()!;
    gl.bindVertexArray(this.outlineVao);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.outlineBuffer);
    let outlineOffset = 0;
    for (const [index, count] of [2, 4, 4].entries()) {
      gl.enableVertexAttribArray(index);
      gl.vertexAttribPointer(index, count, gl.FLOAT, false, 40, outlineOffset * 4);
      outlineOffset += count;
    }
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
      this.focused = null;
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
  inspect(_hovered: string | null, focused: string | null) {
    if (focused === this.focused) return;
    for (const id of [this.focused, focused]) if (id) this.changedInspection.add(id);
    this.staticBatches = undefined;
    this.focused = focused;
  }
  get statistics() {
    return {
      units: this.scene?.units.length ?? 0,
      items: this.scene?.items.length ?? 0,
      tiles: this.scene?.tiles.length ?? 0,
      batches: this.batches.length + (this.outlineData?.length ? 1 : 0),
      vertices:
        (this.outlineData?.length ?? 0) / 10 +
        this.batches.reduce((sum, batch) => sum + batch.values.length / stride, 0),
    };
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
      if (now >= pulse.start + (pulse.kind === "select" ? 180 : 300)) this.pulses.delete(tile);
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
  private outline(
    points: readonly (readonly [number, number])[],
    width: number,
    color: Color,
    dash?: readonly [number, number],
  ) {
    const half = width / 2,
      expanded = half + this.fringe;
    const edges = points.map((point, i) => {
      const next = points[(i + 1) % points.length]!;
      const dx = next[0] - point[0],
        dy = next[1] - point[1],
        length = Math.hypot(dx, dy);
      return { dx: dx / length, dy: dy / length, length, nx: dy / length, ny: -dx / length };
    });
    const miters = points.map((point, i) => {
      const before = edges[(i + edges.length - 1) % edges.length]!,
        after = edges[i]!;
      const nx = before.nx + after.nx,
        ny = before.ny + after.ny;
      const factor = expanded / (nx * after.nx + ny * after.ny);
      return [
        point[0] + nx * factor,
        point[1] + ny * factor,
        point[0] - nx * factor,
        point[1] - ny * factor,
      ] as const;
    });
    const perimeter = edges.reduce((sum, edge) => sum + edge.length, 0);
    const on = (distance: number) => {
      distance = ((distance % perimeter) + perimeter) % perimeter;
      return (
        !dash ||
        ((distance % (dash[0] + dash[1])) + dash[0] + dash[1]) % (dash[0] + dash[1]) < dash[0]
      );
    };
    let distance = 0;
    for (const [i, edge] of edges.entries()) {
      const origin = points[i]!,
        next = (i + 1) % points.length;
      const cross = (at: number, join: boolean, vertex: number) =>
        join
          ? miters[vertex]!
          : [
              origin[0] + edge.dx * at + edge.nx * expanded,
              origin[1] + edge.dy * at + edge.ny * expanded,
              origin[0] + edge.dx * at - edge.nx * expanded,
              origin[1] + edge.dy * at - edge.ny * expanded,
            ];
      let at = 0;
      while (at < edge.length - 1e-6) {
        const phase = dash ? (distance + at) % (dash[0] + dash[1]) : 0;
        const visible = !dash || phase < dash[0];
        const length = Math.min(
          edge.length - at,
          dash ? (visible ? dash[0] - phase : dash[0] + dash[1] - phase) : edge.length,
        );
        const end = at + Math.max(1e-6, length);
        if (visible) {
          const joinStart = at === 0 && on(distance - 1e-6);
          const joinEnd = end >= edge.length - 1e-6 && on(distance + edge.length + 1e-6);
          const a = cross(joinStart ? at : at - this.fringe, joinStart, i);
          const b = cross(joinEnd ? end : end + this.fringe, joinEnd, next);
          const dashStart = distance + at - phase,
            dashEnd = dashStart + (dash?.[0] ?? 0);
          const point = (x: number, y: number, side: number) => {
            const along = distance + (x - origin[0]) * edge.dx + (y - origin[1]) * edge.dy;
            this.outlineValues.push(
              x,
              y,
              side * expanded,
              half,
              dash ? along - dashStart : 1e6,
              dash ? dashEnd - along : 1e6,
              ...color,
            );
          };
          point(a[0]!, a[1]!, 1);
          point(b[0]!, b[1]!, 1);
          point(b[2]!, b[3]!, -1);
          point(a[0]!, a[1]!, 1);
          point(b[2]!, b[3]!, -1);
          point(a[2]!, a[3]!, -1);
        }
        at = end;
      }
      distance += edge.length;
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
    const fringe =
      Math.max(width / this.gl.drawingBufferWidth, height / this.gl.drawingBufferHeight) * 1.5;
    if (fringe !== this.fringe) {
      this.staticBatches = undefined;
      this.fringe = fringe;
    }
    this.batches = [];
    this.light = [0, 0, 0];
    if (this.staticBatches) this.batches = [...this.staticBatches];
    else {
      this.outlineValues = [];
      for (const tile of scene.tiles) {
        const classes = new Set(tile.classes.split(" "));
        const focus = tile.id === this.focused;
        let color = "#39472b",
          alpha = 0.3,
          thickness = 0.7;
        if (classes.has("selected") || focus) {
          color = "#ffe5a1";
          thickness = 3;
        }
        if (classes.has("reachable")) {
          color = "#95bfd2";
          alpha = 1;
          thickness = 2.4;
        }
        if (classes.has("target")) {
          color = "#db8980";
          alpha = 1;
          thickness = 3;
        }
        const recruitable = classes.has("recruitable");
        if (recruitable) {
          color = "#8ec7a2";
          alpha = 1;
          thickness = 3;
        }
        const pulse = this.pulses.get(tile.id);
        if (pulse?.kind === "select" && now - pulse.start < 180) {
          const t = Math.max(0, (now - pulse.start) / 180);
          alpha = t < 0.55 ? 0.45 + (1 - 0.45) * (t / 0.55) : 1 - (1 - 0.8) * ((t - 0.55) / 0.45);
        }
        const points = Array.from({ length: 6 }, (_, i) => {
          const angle = ((60 * i - 30) * Math.PI) / 180;
          return [
            tile.x + (hexColumnSpacing / Math.sqrt(3)) * Math.cos(angle),
            tile.y + (battlefieldSpriteSize / 2) * Math.sin(angle),
          ] as const;
        });
        if (classes.has("objective") || recruitable)
          this.outline(points, 7, rgba("#172015"), recruitable ? [4, 3] : [9, 5]);
        this.outline(points, thickness, rgba(color, alpha), recruitable ? [4, 3] : undefined);
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
      this.outlineData = new Float32Array(this.outlineValues);
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
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    if (this.outlineData?.length) {
      gl.useProgram(this.outlineProgram);
      gl.uniform2f(this.outlineSize, width, height);
      gl.bindVertexArray(this.outlineVao);
      gl.bindBuffer(gl.ARRAY_BUFFER, this.outlineBuffer);
      if (this.uploadedOutline !== this.outlineData) {
        gl.bufferData(gl.ARRAY_BUFFER, this.outlineData, gl.DYNAMIC_DRAW);
        this.uploadedOutline = this.outlineData;
      }
      gl.drawArrays(gl.TRIANGLES, 0, this.outlineData.length / 10);
    }
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
