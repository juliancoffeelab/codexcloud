/** Asset I/O adapter. All static terrain policy comes from emitted OCaml. */
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { build } from "esbuild";
import type { Canvas } from "@napi-rs/canvas";
import type { Plugin } from "vite";
import type * as PlanAPI from "../frontend/Terrain_plan";
import type * as ArtAPI from "../frontend/Art";
import type { Tile } from "../rust/pkg/hex_command";
import { frontendAliases } from "./frontend-paths";

const virtualId = "virtual:terrain-prebuilt";
type Renderer = { Terrain_plan: typeof PlanAPI; Art: typeof ArtAPI };

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
      if (!production) return "export default {};";
      const frontend = resolve(root, "_build/default/frontend/output/frontend");
      // Bundle compiler output once so Node resolves precisely the same metadata
      // aliases as Vite. This wrapper contains no rendering or key policy.
      const compiled = await build({
        stdin: {
          contents: `export * as Terrain_plan from ${JSON.stringify(resolve(frontend, "Terrain_plan.js"))}; export * as Art from ${JSON.stringify(resolve(frontend, "Art.js"))};`,
          resolveDir: root,
        },
        bundle: true,
        write: false,
        metafile: true,
        platform: "node",
        format: "esm",
        target: "es2022",
        alias: frontendAliases,
      });
      for (const path of Object.keys(compiled.metafile!.inputs)) {
        if (path !== "<stdin>") this.addWatchFile(resolve(root, path));
      }
      const source = compiled.outputFiles[0];
      if (!source) throw new Error("Missing compiled OCaml terrain renderer");
      const { Terrain_plan, Art } = (await import(
        `data:text/javascript;base64,${Buffer.from(source.contents).toString("base64")}`
      )) as Renderer;
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
      const artwork = Art.current();
      const frames: Record<string, Canvas> = Object.create(null);
      const entries: string[] = [];
      const keys = new Set<string>();
      for (const descriptor of descriptors) {
        const { map: mapPath } = await read<{ map: string }>(descriptor);
        const map = await read<{ cols: number; rows: number; mapId: string; tiles: Tile[] }>(
          resolve(dirname(descriptor), mapPath),
        );
        const plan = Terrain_plan.prepare(map.mapId, map.cols, map.rows, map.tiles, artwork);
        if (keys.has(plan.key)) continue;
        keys.add(plan.key);
        for (const tile of plan.tiles) {
          const frame = tile.frame;
          if (!frame) throw new Error(`${map.mapId}/${tile.id}: missing terrain artwork`);
          if (frames[frame.url]) continue;
          const path = resolve(root, "public", frame.url.replace(/^(?:\.\/|\/)+/, ""));
          this.addWatchFile(path);
          const image = createCanvas(frame.width, frame.height);
          image.getContext("2d").drawImage(await loadImage(path), 0, 0);
          frames[frame.url] = image;
        }
        const dimensions = Terrain_plan.dimensions(plan);
        const canvas = createCanvas(dimensions.width, dimensions.height);
        const context = canvas.getContext("2d");
        Terrain_plan.configure(context, plan.pixel_scale);
        Terrain_plan.paint(context, plan.tiles, frames);
        const reference = this.emitFile({
          type: "asset",
          name: `terrain-${map.mapId}.png`,
          source: canvas.toBuffer("image/png"),
        });
        const sprite = {
          width: dimensions.width,
          height: dimensions.height,
          displayWidth: dimensions.display_width,
          displayHeight: dimensions.display_height,
        };
        entries.push(
          `${JSON.stringify(plan.key)}: { ...${JSON.stringify(sprite)}, url: import.meta.ROLLUP_FILE_URL_${reference} }`,
        );
      }
      console.log(`Prebuilt terrain for ${entries.length} authored maps`);
      return `export default {${entries.join(",\n")}};`;
    },
  };
}
