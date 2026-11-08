import { defineConfig } from "vitest/config";
import { terrainPrebuild } from "./scripts/terrain-prebuild.mjs";
import { frontendAliases } from "./scripts/frontend-paths.mjs";
import { availableParallelism } from "node:os";
export default defineConfig({
  plugins: [terrainPrebuild()],
  resolve: { alias: frontendAliases },
  test: {
    environment: "node",
    setupFiles: ["tests/setup-wasm.mjs"],
    pool: "threads",
    bail: 0,
    include: ["tests/**/*.test.mjs"],
    fileParallelism: true,
    fsModuleCache: true,
    maxWorkers: Math.min(4, availableParallelism()),
    testTimeout: 0,
    hookTimeout: 0,
    restoreMocks: true,
    unstubGlobals: true,
  },
});
