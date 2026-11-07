/** Bake presentation assets offline. Runtime map state still comes only from Rust. */
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { createCanvas, loadImage, type Canvas } from "@napi-rs/canvas";
import type { Plugin } from "vite";
import { currentArtwork } from "../src/presentation/art";
import {
  terrainPageInterior,
  terrainPageGutter,
  terrainMapKey,
  terrainAssetKey,
  terrainMapDimensions,
  terrainMapTiles,
  paintTerrainTiles,
} from "../src/presentation/terrain-map";
import { battlefieldSpriteSize, mapDimensions } from "../src/ui/geometry";
import { pixelSprites } from "../src/presentation/pixel-art";
import type { GameView, TileView } from "../src/game/view";

type MapSource = Pick<GameView, "mapId" | "cols" | "rows"> & {
  tiles: (Omit<TileView, "id" | "location"> & { id?: string })[];
  mapRevision?: number;
};
const virtualId = "virtual:terrain-prebuilt";

export function terrainPrebuild(): Plugin {
  let root: string, production: boolean;
  const served = new Map<string, Buffer>();
  return {
    name: "terrain-prebuild",
    configResolved(config) {
      root = config.root;
      production = config.command === "build";
    },
    configureServer(instance) {
      instance.middlewares.use((req, res, next) => {
        const bytes = served.get(req.url?.split("?")[0] ?? "");
        if (!bytes) return next();
        res.setHeader("Content-Type", "image/png");
        res.setHeader("Cache-Control", "no-cache");
        res.end(bytes);
      });
    },
    resolveId(id) {
      if (id === virtualId) return "\0" + virtualId;
    },
    async load(id) {
      if (id !== "\0" + virtualId) return;
      const read = async <T>(path: string): Promise<T> => {
        this.addWatchFile(path);
        return JSON.parse(await readFile(path, "utf8"));
      };
      const content = resolve(root, "rust/data");
      const manifest = await read<{ terrain: string; scenarios: string[]; campaigns: string[] }>(
        resolve(content, "manifest.json"),
      );
      const definitions = await read<
        { id: string; role: import("../rust/pkg/hex_command").TerrainRole }[]
      >(resolve(content, manifest.terrain));
      const roles = Object.fromEntries(definitions.map((t) => [t.id, t.role]));
      const descriptors = manifest.scenarios.map((path) => resolve(content, path));
      for (const path of manifest.campaigns) {
        const file = resolve(content, path);
        const campaign = await read<{ stages: string[] }>(file);
        descriptors.push(...campaign.stages.map((stage) => resolve(dirname(file), stage)));
      }
      this.addWatchFile(resolve(root, "src/presentation/pixel-art.ts"));
      const artwork = currentArtwork(),
        frames = new Map<string, Canvas>();
      const entries = new Map<string, string>(),
        identities = new Map<string, string>();
      for (const descriptor of descriptors) {
        const stage = await read<{ map: string; scripts?: string }>(descriptor);
        const map = await read<MapSource>(resolve(dirname(descriptor), stage.map));
        const state = {
          ...map,
          tiles: map.tiles.map((tile) => ({
            ...tile,
            id: `${tile.col},${tile.row}`,
            location: tile.id,
          })),
        };
        const key = terrainMapKey(state),
          assetKey = terrainAssetKey(`${state.mapId}:${state.mapRevision ?? 1}`);
        if (identities.has(assetKey) && identities.get(assetKey) !== key)
          throw new Error(`Ambiguous prebaked map: ${state.mapId}`);
        identities.set(assetKey, key);
        if (entries.has(assetKey)) continue;
        const plan = {
          ...mapDimensions(state),
          pixelScale: 64 / battlefieldSpriteSize,
          tiles: terrainMapTiles(state.tiles, state.mapId, artwork, roles),
        };
        for (const tile of plan.tiles) {
          const frame = tile.terrainFrame;
          if (!frame) throw new Error(`Missing terrain artwork: ${state.mapId}/${tile.id}`);
          if (frames.has(frame.url)) continue;
          const path = resolve(root, "public", pixelSprites[tile.frameKey]!);
          this.addWatchFile(path);
          const canvas = createCanvas(frame.width, frame.height);
          canvas.getContext("2d").drawImage(await loadImage(path), 0, 0);
          frames.set(frame.url, canvas);
        }
        const dimensions = terrainMapDimensions(plan);
        const pages: string[] = [];
        for (let y = 0; y < dimensions.height; y += terrainPageInterior)
          for (let x = 0; x < dimensions.width; x += terrainPageInterior) {
            const interiorWidth = Math.min(terrainPageInterior, dimensions.width - x),
              interiorHeight = Math.min(terrainPageInterior, dimensions.height - y),
              gutter = terrainPageGutter;
            const width = interiorWidth + gutter * 2,
              height = interiorHeight + gutter * 2;
            const canvas = createCanvas(width, height),
              ctx = canvas.getContext("2d");
            ctx.imageSmoothingEnabled = false;
            ctx.translate(gutter - x, gutter - y);
            ctx.beginPath();
            ctx.rect(0, 0, dimensions.width, dimensions.height);
            ctx.clip();
            ctx.scale(plan.pixelScale, plan.pixelScale);
            paintTerrainTiles(ctx, plan.tiles, frames);
            const bytes = canvas.toBuffer("image/png");
            let url: string;
            if (production) {
              const ref = this.emitFile({
                type: "asset",
                name: `terrain-${state.mapId}-${x}-${y}.png`,
                source: bytes,
              });
              url = `import.meta.ROLLUP_FILE_URL_${ref}`;
            } else {
              const path = `/@terrain/${createHash("sha256").update(bytes).digest("hex").slice(0, 24)}.png`;
              served.set(path, bytes);
              url = JSON.stringify(path);
            }
            pages.push(
              `{...${JSON.stringify({ x, y, interiorWidth, interiorHeight, gutter, width, height })},url:${url}}`,
            );
          }
        entries.set(
          assetKey,
          `${JSON.stringify(assetKey)}:{...${JSON.stringify(dimensions)},pages:[${pages.join(",")} ]}`,
        );
      }
      return `export default {${[...entries.values()].join(",\n")}};`;
    },
  };
}
