import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { Worker } from "node:worker_threads";
import { readFile } from "node:fs/promises";
import { build } from "esbuild";
import { frontendAliases, frontendModulePath } from "./frontend-paths.mjs";
import { digest } from "./terrain-cache.mjs";
const virtualId = "virtual:terrain-prebuilt";
function terrainPrebuild() {
  let root,
    production = false;
  const served = new Map();
  return {
    name: "terrain-prebuild",
    configResolved(config) {
      root = config.root;
      production = config.command === "build";
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const bytes = served.get(req.url?.split("?")[0] ?? "");
        if (!bytes) return next();
        res.setHeader("Content-Type", "image/png");
        res.setHeader("Cache-Control", "no-cache");
        res.end(bytes);
      });
    },
    resolveId(id) {
      if (id === virtualId) return "\0" + id;
    },
    async load(id) {
      if (id !== "\0" + virtualId) return;
      const compiled = await build({
        stdin: {
          contents: `export {initialize} from ${JSON.stringify(resolve(root, "rust/pkg/bindings.runtime.mjs"))}; export * as Terrain_plan from ${JSON.stringify(frontendModulePath("Terrain_plan"))}; export * as Terrain_assets from ${JSON.stringify(frontendModulePath("Terrain_assets"))};`,
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
      for (const path of Object.keys(compiled.metafile.inputs))
        if (path !== "<stdin>") this.addWatchFile(resolve(root, path));
      // Hash the painter's executable dependency graph separately from the full
      // catalog bundle: changing one map or sprite must not invalidate all maps.
      const painter = await build({
        entryPoints: [frontendModulePath("Terrain_painter")],
        bundle: true,
        write: false,
        metafile: true,
        platform: "node",
        format: "esm",
        target: "es2022",
        alias: frontendAliases,
      });
      const rendererInputs = [
        // Include the exported paint/dimensions forwarding code too, without
        // hashing catalog or unrelated artwork into every map's fingerprint.
        frontendModulePath("Terrain_plan"),
        "scripts/terrain-prepare-worker.mjs",
        "scripts/terrain-cache.mjs",
        "package-lock.json",
      ];
      for (const path of [...Object.keys(painter.metafile.inputs), ...rendererInputs])
        this.addWatchFile(resolve(root, path));
      const rendererKey = digest(
        Buffer.concat([
          Buffer.from(`${process.platform}:${process.arch}:${process.versions.node}:`),
          painter.outputFiles[0].contents,
          ...(await Promise.all(rendererInputs.map((path) => readFile(resolve(root, path))))),
        ]),
      );
      const wasmPath = resolve(root, "rust/pkg/backend.wasm");
      this.addWatchFile(wasmPath);
      const prepared = await new Promise((resolvePrepared, reject) => {
        const worker = new Worker(resolve(root, "scripts/terrain-prepare-worker.mjs"), {
          workerData: { root, source: compiled.outputFiles[0].contents, wasmPath, rendererKey },
        });
        worker.once("error", reject);
        worker.once("message", (message) => {
          worker.terminate();
          if (message.error) reject(new Error(message.error));
          else resolvePrepared(message);
        });
        worker.once("exit", (code) => {
          if (code !== 0) reject(new Error(`Terrain preparation worker exited (${code})`));
        });
      });
      for (const path of prepared.watched) this.addWatchFile(path);
      const entries = [];
      for (const asset of prepared.assets) {
        const pages = [];
        for (const page of asset.pages) {
          const bytes = Buffer.from(page.bytes);
          let url;
          if (production) {
            const reference = this.emitFile({
              type: "asset",
              name: `terrain-${asset.mapId}-${page.metadata.x}-${page.metadata.y}.png`,
              source: bytes,
            });
            url = `import.meta.ROLLUP_FILE_URL_${reference}`;
          } else {
            const path = `/@terrain/${createHash("sha256").update(bytes).digest("hex").slice(0, 24)}.png`;
            served.set(path, bytes);
            url = JSON.stringify(path);
          }
          pages.push(`{...${JSON.stringify(page.metadata)},url:${url}}`);
        }
        entries.push(
          `${JSON.stringify(asset.key)}:{...${JSON.stringify(asset.sprite)},pages:[${pages.join(",")}]}`,
        );
      }
      console.log(
        `Prebuilt game terrain for ${entries.length} authored maps (${prepared.cached} cached)`,
      );
      return `export default {${entries.join(",\n")}};`;
    },
  };
}
export { terrainPrebuild };
