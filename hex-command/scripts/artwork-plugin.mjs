import { readFileSync, readdirSync } from "node:fs";
import { resolve, relative } from "node:path";
export function artworkPlugin() {
  let path, root, sprites, production;
  const load = () => {
    sprites = new Map(
      Object.entries(JSON.parse(readFileSync(path, "utf8"))).map(([name, bytes]) => [
        name,
        Buffer.from(bytes, "base64"),
      ]),
    );
  };
  return {
    name: "pixel-artwork",
    config() {
      return { build: { copyPublicDir: false } };
    },
    configResolved(config) {
      root = config.root;
      production = config.command === "build";
      path = resolve(root, "_build/default/artwork/sprites.json");
    },
    buildStart() {
      this.addWatchFile(path);
      load();
      if (!production) return;
      for (const [name, source] of sprites)
        this.emitFile({ type: "asset", fileName: `assets/sprites/${name}`, source });
      // Retired generated sprites may remain in an existing checkout. Package
      // authored public files, and take generated sprites only from the rule.
      const publicRoot = resolve(root, "public");
      const walk = (directory) => {
        for (const entry of readdirSync(directory, { withFileTypes: true })) {
          const file = resolve(directory, entry.name);
          const name = relative(publicRoot, file).replaceAll("\\", "/");
          if (name === "assets/sprites") continue;
          if (entry.isDirectory()) walk(file);
          else this.emitFile({ type: "asset", fileName: name, source: readFileSync(file) });
        }
      };
      walk(publicRoot);
    },
    configureServer(server) {
      const reload = (file) => {
        if (file === path) {
          load();
          server.ws.send({ type: "full-reload" });
        }
      };
      server.watcher.on("change", reload);
      server.watcher.on("add", reload);
      server.middlewares.use((req, res, next) => {
        const name = req.url?.split("?")[0]?.match(/^\/assets\/sprites\/([^/]+)$/)?.[1];
        if (!name) return next();
        const bytes = sprites?.get(name);
        if (!bytes) {
          res.statusCode = 404;
          return res.end();
        }
        res.setHeader("Content-Type", "image/png");
        res.end(bytes);
      });
    },
  };
}
