import { defineConfig } from "vite";
import { frontendAliases } from "./scripts/frontend-paths";
import { terrainPrebuild } from "./scripts/terrain-prebuild";

export default defineConfig({
  plugins: [terrainPrebuild()],
  base: "./",
  resolve: { alias: frontendAliases },
  build: { target: "es2022" },
});
