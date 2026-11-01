import { tileCenter } from "../ui/geometry";

import type {
  BattlefieldLight as AuthoredLight,
  BattlefieldLighting as AuthoredLighting,
} from "../../rust/pkg/hex_command";
import type { Immutable } from "../game/immutable";
export type BattlefieldLight = Immutable<AuthoredLight>;
export type BattlefieldLighting = Immutable<AuthoredLighting>;

export const daylight: BattlefieldLighting = Object.freeze({ time: "day", lights: [] });

export function lightingKey(lighting: BattlefieldLighting = daylight): string {
  return JSON.stringify([
    lighting.time,
    (lighting.lights ?? []).map((light) => [
      light.col,
      light.row,
      light.dx ?? 0,
      light.dy ?? 0,
      light.radius,
      light.power,
      light.color,
      light.fixture,
    ]),
  ]);
}

export function lightPosition(light: BattlefieldLight) {
  const point = tileCenter(light);
  return { x: point.x + (light.dx ?? 0), y: point.y + (light.dy ?? 0) };
}

export function lightStrength(x: number, y: number, lighting: BattlefieldLighting) {
  let strength = 0;
  for (const light of lighting.lights ?? []) {
    const point = lightPosition(light);
    const distance = Math.hypot(x - point.x, (y - point.y) * 1.12) / light.radius;
    if (distance < 1) strength += light.power * (1 - distance * distance) ** 2;
  }
  return Math.min(1, strength) * 0.88;
}

/** Blend authored light colors without changing terrain geometry. */
function lightPalette(x: number, y: number, lighting: BattlefieldLighting) {
  const warm = [0, 0, 0],
    offset = [0, 0, 0];
  let total = 0;
  for (const light of lighting.lights ?? []) {
    const point = lightPosition(light),
      distance = Math.hypot(x - point.x, (y - point.y) * 1.12) / light.radius;
    if (distance >= 1) continue;
    const strength = light.power * (1 - distance * distance) ** 2;
    total += strength;
    for (let c = 0; c < 3; c++) {
      warm[c]! +=
        strength * (light.color ? (light.color[c]! / 255) * 1.06 : [1.02, 0.78, 0.49][c]!);
      offset[c]! += strength * (light.color ? (light.color[c]! / 255) * 30 : [22, 14, 7][c]!);
    }
  }
  return total
    ? { warm: warm.map((v) => v / total), offset: offset.map((v) => v / total) }
    : { warm: [1.02, 0.78, 0.49], offset: [22, 14, 7] };
}

/** Linear color transform shared by terrain pixels and SVG unit sprites. */
export function lightingMatrix(x: number, y: number, lighting: BattlefieldLighting) {
  if (lighting.time === "day") return [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0];
  const mix = lightStrength(x, y, lighting);
  const luminance = [0.2126, 0.7152, 0.0722];
  const night = [0.28, 0.41, 0.63];
  const palette = (lighting.lights ?? []).some((light) => light.color)
    ? lightPalette(x, y, lighting)
    : { warm: [1.02, 0.78, 0.49], offset: [22, 14, 7] };
  const warm = palette.warm;
  const nightOffset = [4, 8, 17];
  const warmOffset = palette.offset;
  return [0, 1, 2]
    .flatMap((channel) => [
      ...[0, 1, 2].map(
        (source) =>
          ((source === channel ? 0.8 : 0) + 0.2 * luminance[source]!) *
            night[channel]! *
            (1 - mix) +
          (source === channel ? warm[channel]! * mix : 0),
      ),
      0,
      (nightOffset[channel]! * (1 - mix) + warmOffset[channel]! * mix) / 255,
    ])
    .concat([0, 0, 0, 1, 0]);
}

/** Apply lighting to a row range in the rendering worker or offline asset builder. */
export function shadePixels(
  data: Uint8ClampedArray,
  width: number,
  start: number,
  end: number,
  pixelScale: number,
  lighting: BattlefieldLighting,
) {
  if (lighting.time === "day") return;
  const colored = (lighting.lights ?? []).some((light) => light.color);
  const fallbackPalette = { warm: [1.02, 0.78, 0.49], offset: [22, 14, 7] };
  for (let y = start; y < end; y++)
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      if (!data[i + 3]) continue;
      const r = data[i]!,
        g = data[i + 1]!,
        b = data[i + 2]!;
      const mix = lightStrength(x / pixelScale, y / pixelScale, lighting);
      const palette = colored
        ? lightPalette(x / pixelScale, y / pixelScale, lighting)
        : fallbackPalette;
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      data[i] =
        (r * 0.8 + lum * 0.2) * 0.28 * (1 - mix) +
        4 * (1 - mix) +
        (r * palette.warm[0]! + palette.offset[0]!) * mix;
      data[i + 1] =
        (g * 0.8 + lum * 0.2) * 0.41 * (1 - mix) +
        8 * (1 - mix) +
        (g * palette.warm[1]! + palette.offset[1]!) * mix;
      data[i + 2] =
        (b * 0.8 + lum * 0.2) * 0.63 * (1 - mix) +
        17 * (1 - mix) +
        (b * palette.warm[2]! + palette.offset[2]!) * mix;
    }
}

type LightContext = Pick<
  CanvasRenderingContext2D,
  | "save"
  | "restore"
  | "fillRect"
  | "createRadialGradient"
  | "fillStyle"
  | "globalCompositeOperation"
>;

/** Emissive fixtures and halos are baked into static terrain too. */
export function paintLights(context: LightContext, lighting: BattlefieldLighting) {
  if (lighting.time === "day") return;
  context.save();
  for (const light of lighting.lights ?? []) {
    const { x, y } = lightPosition(light);
    if (light.fixture === false) {
      const [r, g, b] = light.color ?? [255, 170, 70];
      context.globalCompositeOperation = "screen";
      const halo = context.createRadialGradient(x, y, 0, x, y, 25);
      halo.addColorStop(0, `rgba(${r},${g},${b},.95)`);
      halo.addColorStop(0.25, `rgba(${r},${g},${b},.45)`);
      halo.addColorStop(1, `rgba(${r},${g},${b},0)`);
      context.fillStyle = halo;
      context.fillRect(x - 25, y - 25, 50, 50);
      context.globalCompositeOperation = "source-over";
      continue;
    }
    context.fillStyle = "#49361f";
    context.fillRect(x - 3, y - 9, 6, 12);
    context.fillStyle = "#f6aa46";
    context.fillRect(x - 2, y - 7, 4, 5);
    context.fillStyle = "#fff0b0";
    context.fillRect(x - 1, y - 6, 2, 3);
    context.fillStyle = "#50402e";
    context.fillRect(x - 1, y + 3, 2, 8);
    context.globalCompositeOperation = "screen";
    const halo = context.createRadialGradient(x, y - 5, 0, x, y - 5, 20);
    halo.addColorStop(0, "rgba(255,204,110,.65)");
    halo.addColorStop(0.25, "rgba(255,169,65,.24)");
    halo.addColorStop(1, "rgba(255,145,44,0)");
    context.fillStyle = halo;
    context.fillRect(x - 20, y - 25, 40, 40);
    context.globalCompositeOperation = "source-over";
  }
  context.restore();
}
