/** Production-only assets; map JSON and pixel matrices remain the tracked sources. */
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import type { Canvas } from "@napi-rs/canvas";
import type { Plugin } from "vite";
import { currentArtwork } from "../src/presentation/art";
import {
  terrainMapKey,
  terrainMapDimensions,
  terrainTileView,
  paintTerrainTiles,
} from "../src/presentation/terrain-map";
import { battlefieldSpriteSize, mapDimensions } from "../src/ui/geometry";
import { pixelSprites } from "../src/presentation/pixel-art";
import type { GameView } from "../src/game/view";

const virtualId = "virtual:terrain-prebuilt";

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
      // Development keeps the ordinary renderer; production emits complete map PNGs.
      if (!production) return "export default {};";
      const { createCanvas, loadImage } = await import("@napi-rs/canvas");
      const read = async <T>(path: string): Promise<T> => {
        this.addWatchFile(path);
        return JSON.parse(await readFile(path, "utf8")) as T;
      };
      const content = resolve(root, "rust/data");
      const manifest = await read<{ scenarios: string[]; campaigns: string[] }>(
        resolve(content, "manifest.json"),
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
      for (const descriptor of descriptors) {
        const { map: mapPath } = await read<{ map: string }>(descriptor);
        const map = await read<Pick<GameView, "cols" | "rows" | "mapId" | "tiles">>(
          resolve(dirname(descriptor), mapPath),
        );
        const key = terrainMapKey(map);
        if (keys.has(key)) continue;
        keys.add(key);
        const plan = {
          ...mapDimensions(map),
          pixelScale: 64 / battlefieldSpriteSize,
          tiles: map.tiles.map((tile) => ({
            ...tile,
            ...terrainTileView(tile, map.mapId, artwork),
          })),
        };
        for (const tile of plan.tiles) {
          const name = tile.terrainFrame?.url;
          if (!name) throw new Error(`${map.mapId}/${tile.id}: missing terrain artwork`);
          if (frames.has(name)) continue;
          const path = resolve(root, "public", pixelSprites[tile.frameKey]!);
          this.addWatchFile(path);
          // Keep build-time raster clipping identical to the canvas fallback.
          const frame = createCanvas(64, 64);
          frame.getContext("2d").drawImage(await loadImage(path), 0, 0);
          frames.set(name, frame);
        }
        const dimensions = terrainMapDimensions(plan);
        const canvas = createCanvas(dimensions.width, dimensions.height);
        const context = canvas.getContext("2d");
        context.imageSmoothingEnabled = false;
        context.scale(plan.pixelScale, plan.pixelScale);
        paintTerrainTiles(context, plan.tiles, frames);
        const reference = this.emitFile({
          type: "asset",
          name: `terrain-${map.mapId}.png`,
          source: canvas.toBuffer("image/png"),
        });
        entries.push(
          `${JSON.stringify(key)}: { ...${JSON.stringify(dimensions)}, url: import.meta.ROLLUP_FILE_URL_${reference} }`,
        );
      }
      console.log(`Prebuilt terrain for ${entries.length} authored maps`);
      return `export default {${entries.join(",\n")}};`;
    },
  };
}
