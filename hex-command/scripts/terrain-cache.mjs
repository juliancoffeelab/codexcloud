import { createHash, randomUUID } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";

export function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

// Each entry is one checksummed, atomically replaced file. A missing or damaged
// entry is a cache miss; cache storage must never be required to build the game.
export function terrainCache(directory) {
  let warned = false;
  const warn = (error) => {
    if (!warned) console.warn(`Terrain cache unavailable: ${error.message}`);
    warned = true;
  };
  return {
    async read(id, key) {
      try {
        const { checksum, payload } = JSON.parse(
          await readFile(join(directory, `${id}.json`), "utf8"),
        );
        if (typeof payload !== "string" || digest(payload) !== checksum) return;
        const entry = JSON.parse(payload);
        if (entry.key !== key) return;
        const pages = entry.pages;
        if (!Array.isArray(pages) || !pages.length) return;
        return pages.map(({ metadata, bytes }) => {
          const data = Buffer.from(bytes, "base64");
          if (
            !metadata ||
            !["x", "y", "interiorWidth", "interiorHeight", "gutter", "width", "height"].every(
              (name) => Number.isFinite(metadata[name]),
            ) ||
            !data.subarray(0, 8).equals(Buffer.from("89504e470d0a1a0a", "hex"))
          )
            throw new Error("Invalid terrain cache page");
          return { metadata, bytes: data };
        });
      } catch (error) {
        if (error.code && error.code !== "ENOENT") warn(error);
      }
    },
    async write(id, key, pages) {
      const path = join(directory, `${id}.json`);
      const temporary = `${path}.${randomUUID()}.tmp`;
      try {
        const payload = JSON.stringify({
          key,
          pages: pages.map(({ metadata, bytes }) => ({
            metadata,
            bytes: Buffer.from(bytes).toString("base64"),
          })),
        });
        await mkdir(directory, { recursive: true });
        await writeFile(temporary, JSON.stringify({ checksum: digest(payload), payload }), {
          flag: "wx",
        });
        await rename(temporary, path);
      } catch (error) {
        warn(error);
      } finally {
        await rm(temporary, { force: true }).catch(() => {});
      }
    },
  };
}
