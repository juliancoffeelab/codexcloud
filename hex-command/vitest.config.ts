import { defineConfig } from "vitest/config";
import { terrainPrebuild } from "./scripts/terrain-prebuild";
import { frontendAliases } from "./scripts/frontend-paths";
import { availableParallelism } from "node:os";

export default defineConfig({
  plugins: [terrainPrebuild()],
  resolve: { alias: frontendAliases },
  test: {
    environment: "node",
    setupFiles: ["tests/setup-wasm.ts"],
    pool: "threads",
    bail: 0,
    include: ["tests/**/*.test.ts"],
    fileParallelism: true,
    fsModuleCache: true,
    maxWorkers: Math.min(4, availableParallelism()),
    testTimeout: 5_000,
    hookTimeout: 5_000,
    restoreMocks: true,
    unstubGlobals: true,
  },
});
