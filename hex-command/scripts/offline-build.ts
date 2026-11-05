import { createHash } from "node:crypto";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { resolve, relative } from "node:path";
import type { Plugin } from "vite";

/** Describe the actual release, including copied public artwork/audio and generated WASM. */
export function offlineBuild(): Plugin {
  let output: string;
  return {
    name: "offline-release",
    apply: "build",
    configResolved(config) {
      output = resolve(config.root, config.build.outDir);
    },
    async closeBundle() {
      const files = (await readdir(output, { recursive: true, withFileTypes: true }))
        .filter(
          (entry) =>
            entry.isFile() && !["offline-worker.js", "_headers", "_redirects"].includes(entry.name),
        )
        .map((entry) =>
          relative(output, resolve(entry.parentPath, entry.name)).replaceAll("\\", "/"),
        )
        .sort();
      const template = await readFile(new URL("./offline-worker.js", import.meta.url), "utf8");
      const hash = createHash("sha256").update(template);
      for (const name of files) {
        hash.update(name);
        hash.update(await readFile(resolve(output, name)));
      }
      const worker = template
        .replace("__RELEASE__", JSON.stringify(hash.digest("hex").slice(0, 24)))
        .replace("__ASSETS__", JSON.stringify(files));
      await writeFile(resolve(output, "offline-worker.js"), worker);
    },
  };
}
