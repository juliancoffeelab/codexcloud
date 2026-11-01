import { lightPosition, type BattlefieldLighting } from "./battlefield-lighting";

const vertex = `#version 300 es
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;
const fragment = `#version 300 es
precision highp float;
precision highp sampler2D;
uniform sampler2D terrain;
uniform sampler2D lights;
uniform vec2 size;
uniform float scale;
uniform int count;
uniform bool night;
out vec4 color;
void main() {
  // Match the CPU painter's integer, top-left pixel coordinates exactly.
  vec2 pixel = vec2(floor(gl_FragCoord.x), size.y - ceil(gl_FragCoord.y));
  vec4 source = texelFetch(terrain, ivec2(pixel), 0);
  if (!night || source.a == 0.0) { color = source; return; }
  vec2 point = pixel / scale;
  float total = 0.0;
  vec3 warm = vec3(0.0), offset = vec3(0.0);
  for (int i = 0; i < count; i++) {
    vec4 light = texelFetch(lights, ivec2(0, i), 0);
    vec2 delta = (point - light.xy) * vec2(1.0, 1.12);
    float distance2 = dot(delta, delta) / (light.z * light.z);
    if (distance2 >= 1.0) continue;
    float strength = light.w * (1.0 - distance2) * (1.0 - distance2);
    total += strength;
    warm += strength * texelFetch(lights, ivec2(1, i), 0).rgb;
    offset += strength * texelFetch(lights, ivec2(2, i), 0).rgb;
  }
  if (total > 0.0) { warm /= total; offset /= total; }
  else { warm = vec3(1.02, .78, .49); offset = vec3(22.0, 14.0, 7.0); }
  float mixAmount = min(1.0, total) * .88;
  vec3 rgb = source.rgb * 255.0;
  float lum = dot(rgb, vec3(.2126, .7152, .0722));
  vec3 dark = (rgb * .8 + lum * .2) * vec3(.28, .41, .63) + vec3(4.0, 8.0, 17.0);
  color = vec4(mix(dark, rgb * warm + offset, mixAmount) / 255.0, source.a);
}`;

/** Persistent GPU resources: lighting changes upload only a small light table. */
export class MapLightingGpu {
  readonly canvas = new OffscreenCanvas(1, 1);
  private gl: WebGL2RenderingContext;
  private program: WebGLProgram;
  private terrain: WebGLTexture;
  private lights: WebGLTexture;

  constructor() {
    const gl = this.canvas.getContext("webgl2", {
      alpha: true,
      premultipliedAlpha: false,
      antialias: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: true,
    });
    if (!gl)
      throw new Error("Map lighting requires WebGL 2. Enable browser graphics acceleration.");
    this.gl = gl;
    const compile = (type: number, source: string) => {
      const shader = gl.createShader(type)!;
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        const error = gl.getShaderInfoLog(shader);
        gl.deleteShader(shader);
        throw new Error(`Map lighting shader failed: ${error}`);
      }
      return shader;
    };
    const vs = compile(gl.VERTEX_SHADER, vertex);
    const fs = compile(gl.FRAGMENT_SHADER, fragment);
    this.program = gl.createProgram()!;
    gl.attachShader(this.program, vs);
    gl.attachShader(this.program, fs);
    gl.linkProgram(this.program);
    gl.deleteShader(vs);
    gl.deleteShader(fs);
    if (!gl.getProgramParameter(this.program, gl.LINK_STATUS))
      throw new Error(`Map lighting program failed: ${gl.getProgramInfoLog(this.program)}`);
    gl.useProgram(this.program);
    gl.disable(gl.DITHER);
    this.terrain = this.texture(0);
    this.lights = this.texture(1);
    gl.uniform1i(gl.getUniformLocation(this.program, "terrain"), 0);
    gl.uniform1i(gl.getUniformLocation(this.program, "lights"), 1);
  }

  private texture(unit: number) {
    const gl = this.gl;
    const texture = gl.createTexture()!;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return texture;
  }

  setTerrain(base: OffscreenCanvas) {
    const gl = this.gl;
    const limit = gl.getParameter(gl.MAX_TEXTURE_SIZE) as number;
    if (base.width > limit || base.height > limit)
      throw new Error(`Map exceeds GPU texture limit (${limit} pixels)`);
    this.canvas.width = base.width;
    this.canvas.height = base.height;
    gl.viewport(0, 0, base.width, base.height);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.terrain);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, base);
    gl.uniform2f(gl.getUniformLocation(this.program, "size"), base.width, base.height);
    this.check();
  }

  render(lighting: BattlefieldLighting, pixelScale: number) {
    const gl = this.gl;
    if (gl.isContextLost()) throw new Error("Map GPU context was lost. Retry map rendering.");
    const lights = lighting.lights ?? [];
    if (lights.length > (gl.getParameter(gl.MAX_TEXTURE_SIZE) as number))
      throw new Error("Map has too many lights for this GPU");
    const data = new Float32Array(Math.max(1, lights.length) * 12);
    lights.forEach((light, index) => {
      const { x, y } = lightPosition(light);
      const color = light.color?.map((channel) => channel / 255);
      data.set(
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
    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.lights);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA32F,
      3,
      Math.max(1, lights.length),
      0,
      gl.RGBA,
      gl.FLOAT,
      data,
    );
    gl.uniform1i(gl.getUniformLocation(this.program, "count"), lights.length);
    gl.uniform1i(gl.getUniformLocation(this.program, "night"), Number(lighting.time === "night"));
    gl.uniform1f(gl.getUniformLocation(this.program, "scale"), pixelScale);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    this.check();
    return this.canvas;
  }

  private check() {
    const error = this.gl.getError();
    if (error !== this.gl.NO_ERROR) throw new Error(`Map GPU operation failed (${error})`);
  }
}
