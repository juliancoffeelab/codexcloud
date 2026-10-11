import { defineConfig } from "vite";
import { terrainPrebuild } from "./scripts/terrain-prebuild";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  base: "./",
  plugins: [svelte(), terrainPrebuild()],
  build: { target: "es2022" },
});
