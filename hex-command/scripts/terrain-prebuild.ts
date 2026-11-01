/** Production-only assets; map JSON and pixel matrices remain the tracked sources. */
import {
  daylight,
  paintLights,
  shadePixels,
  type BattlefieldLighting,
} from "../src/presentation/battlefield-lighting";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import type { Canvas } from "@napi-rs/canvas";
import type { Plugin } from "vite";
import { currentArtwork } from "../src/presentation/art";
import {
  terrainMapKey,
  terrainAssetKey,
  terrainMapDimensions,
  terrainMapTiles,
  paintTerrainTiles,
} from "../src/presentation/terrain-map";
import { battlefieldSpriteSize, mapDimensions } from "../src/ui/geometry";
import { pixelSprites } from "../src/presentation/pixel-art";
import type { PresentationInput } from "../rust/pkg/hex_command";
import type { GameView } from "../src/game/view";

const virtualId = "virtual:terrain-prebuilt";

function lightingVariants(lighting: BattlefieldLighting, initial: BattlefieldLighting) {
  const controlled = (lighting.lights ?? []).filter((light) => light.when);
  // Bound build output. Larger authored combinations use the existing worker cache.
  if (controlled.length > 4) return [initial];
  return [
    initial,
    ...Array.from({ length: 2 ** controlled.length }, (_, mask) => ({
      time: lighting.time,
      lights: lighting.lights?.filter(
        (light) => !light.when || mask & (1 << controlled.indexOf(light)),
      ),
    })),
  ];
}

export function terrainPrebuild(): Plugin {
  let root: string;
  let production = false;
  return {
    name: "terrain-prebuild",
    configResolved(config) {
      root = config.root;
      production = config.command === "build";
    },
    resolveId(id) {
      if (id === virtualId) return "\0" + virtualId;
    },
    async load(id) {
      if (id !== "\0" + virtualId) return;
      // Development uses the same authored settings with the asynchronous fallback.
      const { createCanvas, loadImage } = await import("@napi-rs/canvas");
      const read = async <T>(path: string): Promise<T> => {
        this.addWatchFile(path);
        return JSON.parse(await readFile(path, "utf8")) as T;
      };
      const { initSync, previewPresentation } = await import("../rust/pkg/hex_command");
      initSync({ module: await readFile(resolve(root, "rust/pkg/hex_command_bg.wasm")) });
      const content = resolve(root, "rust/data");
      const manifest = await read<{ terrain: string; scenarios: string[]; campaigns: string[] }>(
        resolve(content, "manifest.json"),
      );
      const terrainDefinitions = await read<
        { id: string; role: import("../rust/pkg/hex_command").TerrainRole }[]
      >(resolve(content, manifest.terrain));
      const terrainRoles = Object.fromEntries(
        terrainDefinitions.map((definition) => [definition.id, definition.role]),
      );
      const descriptors = manifest.scenarios.map((path) => resolve(content, path));
      for (const campaign of manifest.campaigns) {
        const path = resolve(content, campaign);
        const { stages } = await read<{ stages: string[] }>(path);
        descriptors.push(...stages.map((stage) => resolve(dirname(path), stage)));
      }
      this.addWatchFile(resolve(root, "src/presentation/pixel-art.ts"));
      const artwork = currentArtwork();
      const frames = new Map<string, Canvas>();
      const entries: string[] = [];
      const keys = new Set<string>();
      const assetKeys = new Map<string, string>();
      for (const descriptor of descriptors) {
        const { map: mapPath } = await read<{ map: string }>(descriptor);
        const map = await read<
          Pick<GameView, "cols" | "rows" | "mapId" | "tiles"> & { lighting?: BattlefieldLighting }
        >(resolve(dirname(descriptor), mapPath));
        const presentation = map.lighting ?? daylight;
        if (!production) continue;
        for (const lighting of lightingVariants(
          presentation,
          previewPresentation(
            structuredClone({
              tiles: map.tiles,
              units: [],
              lighting: presentation,
              marks: [],
              memory: [],
              round: 1,
            }) as unknown as PresentationInput,
          ).lighting,
        )) {
          const key = terrainMapKey(map, lighting);
          if (keys.has(key)) continue;
          keys.add(key);
          const assetKey = terrainAssetKey(key);
          if (assetKeys.has(assetKey) && assetKeys.get(assetKey) !== key)
            throw new Error(`Terrain image key collision: ${assetKey}`);
          assetKeys.set(assetKey, key);
          const plan = {
            ...mapDimensions(map),
            pixelScale: 64 / battlefieldSpriteSize,
            tiles: terrainMapTiles(map.tiles, map.mapId, artwork, terrainRoles),
          };
          for (const tile of plan.tiles) {
            const name = tile.terrainFrame?.url;
            if (!name) throw new Error(`${map.mapId}/${tile.id}: missing terrain artwork`);
            if (frames.has(name)) continue;
            const path = resolve(root, "public", pixelSprites[tile.frameKey]!);
            this.addWatchFile(path);
            // Keep offline raster clipping identical to the rendering worker.
            const frame = createCanvas(tile.terrainFrame!.width, tile.terrainFrame!.height);
            frame.getContext("2d").drawImage(await loadImage(path), 0, 0);
            frames.set(name, frame);
          }
          const dimensions = terrainMapDimensions(plan);
          const canvas = createCanvas(dimensions.width, dimensions.height);
          const context = canvas.getContext("2d");
          context.imageSmoothingEnabled = false;
          context.scale(plan.pixelScale, plan.pixelScale);
          paintTerrainTiles(context, plan.tiles, frames);
          if (lighting.time !== "day") {
            const pixels = context.getImageData(0, 0, dimensions.width, dimensions.height);
            shadePixels(
              pixels.data,
              dimensions.width,
              0,
              dimensions.height,
              plan.pixelScale,
              lighting,
            );
            context.putImageData(pixels, 0, 0);
            paintLights(context, lighting);
          }
          const reference = this.emitFile({
            type: "asset",
            name: `terrain-${map.mapId}.png`,
            source: canvas.toBuffer("image/png"),
          });
          entries.push(
            `${JSON.stringify(terrainAssetKey(key))}: { ...${JSON.stringify(dimensions)}, url: import.meta.ROLLUP_FILE_URL_${reference} }`,
          );
        }
      }
      console.log(`Prebuilt terrain for ${entries.length} authored maps`);
      return `export default {${entries.join(",\n")}};`;
    },
  };
}
