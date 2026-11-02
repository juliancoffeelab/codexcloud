/** Render targets use premultiplied RGBA; uploaded artwork uses straight RGBA and top-first rows. */
export const fullscreenVertex = `#version 300 es
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

export const quadVertex = `#version 300 es
uniform vec2 size;
uniform vec4 rect;
out vec2 uv;
void main() {
  vec2 p = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1));
  uv = p;
  vec2 point = rect.xy + p * rect.zw;
  gl_Position = vec4(point / size * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
}`;

export const spriteFragment = `#version 300 es
precision highp float;
uniform sampler2D image;
uniform float brightness;
in vec2 uv;
out vec4 color;
void main() {
  vec4 source = texture(image, uv);
  color = vec4(source.rgb * source.a * brightness, source.a);
}`;

export const copyFragment = `#version 300 es
precision highp float;
uniform sampler2D terrain;
uniform bool straight;
out vec4 color;
void main() {
  ivec2 point = ivec2(gl_FragCoord.xy);
  if (straight) point.y = textureSize(terrain, 0).y - 1 - point.y;
  color = texelFetch(terrain, point, 0);
  if (straight) color.rgb *= color.a;
}`;

export const lightingFragment = `#version 300 es
precision highp float;
precision highp sampler2D;
uniform sampler2D terrain;
uniform sampler2D lights;
uniform bool straight;
uniform vec2 size;
uniform float scale;
uniform int count;
uniform bool night;
out vec4 color;
void main() {
  ivec2 pixel = ivec2(gl_FragCoord.xy);
  if (straight) pixel.y = textureSize(terrain, 0).y - 1 - pixel.y;
  vec4 source = texelFetch(terrain, pixel, 0);
  if (straight) source.rgb *= source.a;
  if (!night || source.a == 0.0) { color = source; return; }
  vec2 point = vec2(floor(gl_FragCoord.x), size.y - ceil(gl_FragCoord.y)) / scale;
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
  float amount = min(1.0, total) * .88;
  vec3 rgb = source.rgb / source.a * 255.0;
  float lum = dot(rgb, vec3(.2126, .7152, .0722));
  vec3 dark = (rgb * .8 + lum * .2) * vec3(.28, .41, .63) + vec3(4.0, 8.0, 17.0);
  color = vec4(clamp(mix(dark, rgb * warm + offset, amount) / 255.0, 0.0, 1.0) * source.a, source.a);
}`;

// Analytic pixel coverage preserves Canvas fillRect edges at fractional native positions.
export const solidVertex = quadVertex.replace(
  "rect.xy + p * rect.zw",
  "rect.xy - vec2(1.0) + p * (rect.zw + vec2(2.0))",
);

export const solidFragment = `#version 300 es
precision highp float;
uniform vec4 ink;
uniform vec4 rect;
uniform vec2 size;
out vec4 color;
void main() {
  vec2 point = vec2(gl_FragCoord.x, size.y - gl_FragCoord.y);
  vec2 coverage = clamp(min(point + .5, rect.xy + rect.zw) - max(point - .5, rect.xy), 0.0, 1.0);
  float alpha = ink.a * coverage.x * coverage.y;
  color = vec4(ink.rgb * alpha, alpha);
}
`;

export const haloFragment = `#version 300 es
precision highp float;
uniform vec3 glow;
uniform bool fixture;
in vec2 uv;
out vec4 color;
void main() {
  float distance = length(uv * 2.0 - 1.0);
  if (distance >= 1.0) discard;
  vec3 rgb;
  float alpha;
  if (fixture) {
    float t = min(distance / .25, 1.0);
    rgb = mix(vec3(255.0, 204.0, 110.0), vec3(255.0, 169.0, 65.0), t) / 255.0;
    alpha = distance < .25 ? mix(.65, .24, t) : .24 * (1.0 - distance) / .75;
  } else {
    rgb = glow;
    alpha = distance < .25 ? mix(.95, .45, distance / .25) : .45 * (1.0 - distance) / .75;
  }
  color = vec4(rgb * alpha, alpha);
}`;

export const fogVertex = `#version 300 es
layout(location=0) in vec2 point;
uniform vec2 size;
void main() {
  gl_Position = vec4(point / size * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
}`;
export const maskFragment = `#version 300 es
precision highp float;
out vec4 color;
void main() { color = vec4(1.0); }
`;

// Half-resolution, separable Gaussian blur. Paired taps use texture interpolation.
export const blurFragment = `#version 300 es
precision highp float;
uniform sampler2D image;
uniform vec2 size;
uniform vec2 stepSize;
uniform float weights[7];
uniform float offsets[7];
out vec4 color;
void main() {
  vec2 uv = gl_FragCoord.xy / size;
  float value = texture(image, uv).r * weights[0];
  for (int i = 1; i < 7; i++) {
    vec2 delta = stepSize * offsets[i];
    // Zero beyond the canvas, rather than extending edge hexes into the blur.
    vec2 a = uv - delta, b = uv + delta;
    if (all(greaterThanEqual(a, vec2(0.0))) && all(lessThanEqual(a, vec2(1.0))))
      value += texture(image, a).r * weights[i];
    if (all(greaterThanEqual(b, vec2(0.0))) && all(lessThanEqual(b, vec2(1.0))))
      value += texture(image, b).r * weights[i];
  }
  color = vec4(value, 0.0, 0.0, 1.0);
}`;

export const compositeFragment = `#version 300 es
precision highp float;
uniform sampler2D image;
uniform sampler2D fog;
uniform vec2 size;
uniform bool hasFog;
out vec4 color;
void main() {
  vec2 uv = gl_FragCoord.xy / size;
  vec4 source = texture(image, uv);
  float cover = hasFog ? texture(fog, uv).r : 0.0;
  color = vec4(vec3(12.0, 19.0, 32.0) / 255.0 * cover + source.rgb * (1.0 - cover),
               cover + source.a * (1.0 - cover));
}`;
