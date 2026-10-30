import { tileCenter } from "../ui/geometry";

export interface BattlefieldLight {
  readonly col: number;
  readonly row: number;
  readonly dx?: number;
  readonly dy?: number;
  readonly radius: number;
  readonly power: number;
  readonly whenOwner?: "player" | "enemy";
}

/** Presentation only: lights never grant sight, targeting or movement. */
export interface BattlefieldLighting {
  readonly time: "day" | "night";
  readonly lights?: readonly BattlefieldLight[];
}

export const daylight: BattlefieldLighting = Object.freeze({ time: "day" });

/** Resolve ownership against the current snapshot, including restored saves and undo. */
export function resolveLighting(
  lighting: BattlefieldLighting,
  tiles: readonly { col: number; row: number; owner: string | null }[],
): BattlefieldLighting {
  return {
    time: lighting.time,
    lights: lighting.lights?.filter(
      (light) =>
        !light.whenOwner ||
        tiles.some(
          (tile) =>
            tile.col === light.col && tile.row === light.row && tile.owner === light.whenOwner,
        ),
    ),
  };
}

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

/** Linear color transform shared by terrain pixels and SVG unit sprites. */
export function lightingMatrix(x: number, y: number, lighting: BattlefieldLighting) {
  if (lighting.time === "day") return [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0];
  const mix = lightStrength(x, y, lighting);
  const luminance = [0.2126, 0.7152, 0.0722];
  const night = [0.28, 0.41, 0.63];
  const warm = [1.02, 0.78, 0.49];
  const nightOffset = [4, 8, 17];
  const warmOffset = [22, 14, 7];
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

/** Apply a bounded row range so the main-thread fallback can yield between batches. */
export function shadePixels(
  data: Uint8ClampedArray,
  width: number,
  start: number,
  end: number,
  pixelScale: number,
  lighting: BattlefieldLighting,
) {
  if (lighting.time === "day") return;
  for (let y = start; y < end; y++)
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      if (!data[i + 3]) continue;
      const r = data[i]!,
        g = data[i + 1]!,
        b = data[i + 2]!;
      const mix = lightStrength(x / pixelScale, y / pixelScale, lighting);
      const lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      data[i] = (r * 0.8 + lum * 0.2) * 0.28 * (1 - mix) + 4 * (1 - mix) + (r * 1.02 + 22) * mix;
      data[i + 1] =
        (g * 0.8 + lum * 0.2) * 0.41 * (1 - mix) + 8 * (1 - mix) + (g * 0.78 + 14) * mix;
      data[i + 2] =
        (b * 0.8 + lum * 0.2) * 0.63 * (1 - mix) + 17 * (1 - mix) + (b * 0.49 + 7) * mix;
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

/** Fail authored invalid settings at build time, before shipping a mismatched cache. */
export function readLighting(value: unknown): BattlefieldLighting {
  const lighting = value as BattlefieldLighting;
  if (
    !lighting ||
    !["day", "night"].includes(lighting.time) ||
    (lighting.lights !== undefined &&
      (!Array.isArray(lighting.lights) ||
        lighting.lights.some(
          (light) =>
            !light ||
            !Number.isInteger(light.col) ||
            !Number.isInteger(light.row) ||
            !Number.isFinite(light.radius) ||
            light.radius <= 0 ||
            !Number.isFinite(light.power) ||
            light.power < 0 ||
            (light.whenOwner !== undefined && !["player", "enemy"].includes(light.whenOwner)) ||
            (light.dx !== undefined && !Number.isFinite(light.dx)) ||
            (light.dy !== undefined && !Number.isFinite(light.dy)),
        )))
  )
    throw new Error("Invalid battlefield presentation lighting");
  return lighting;
}
