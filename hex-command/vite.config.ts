import { defineConfig } from "vite";
import { terrainPrebuild } from "./scripts/terrain-prebuild";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

const gameDataRoot = fileURLToPath(new URL("./rust/data/", import.meta.url));

export default defineConfig({
  base: "./",
  worker: { format: "es" },
  optimizeDeps: { include: ["phaser"] },
  plugins: [
    {
      name: "rust-runtime-data-boundary",
      enforce: "pre",
      load(id) {
        if (id.split("?")[0]?.startsWith(gameDataRoot))
          this.error(
            "Runtime game data must come from the Rust catalog, not a source JSON import.",
          );
      },
    },
    svelte(),
    terrainPrebuild(),
  ],
  build: { target: "es2022" },
});
