import { execFileSync } from "node:child_process";
import { terrainPrebuild } from "./scripts/terrain-prebuild";
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

const gameDataRoot = fileURLToPath(new URL("./rust/data/", import.meta.url));

export default defineConfig({
  base: "./",
  define: {
    "import.meta.env.VITE_BUILD_COMMIT": JSON.stringify(
      process.env.HEX_COMMAND_SOURCE_COMMIT?.slice(0, 7) ??
        execFileSync("git", ["rev-parse", "--short", "HEAD"], { encoding: "utf8" }).trim(),
    ),
  },
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
