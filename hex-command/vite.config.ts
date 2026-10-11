import { defineConfig } from "vite";
import { frontendAliases } from "./scripts/frontend-paths";
export default defineConfig({
  base: "./",
  resolve: { alias: frontendAliases },
  build: { target: "es2022" },
});
