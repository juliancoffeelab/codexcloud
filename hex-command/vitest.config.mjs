import { defineConfig } from "vitest/config";
import { terrainPrebuild } from "./scripts/terrain-prebuild.mjs";
import { frontendAliases, assertBuildProfile, wasmPath } from "./scripts/frontend-paths.mjs";
import { availableParallelism } from "node:os";
assertBuildProfile();
export default defineConfig({
  plugins: [terrainPrebuild()],
  resolve: { alias: frontendAliases },
  test: {
    env: { HEX_COMMAND_WASM_PATH: wasmPath("backend.wasm") },
    environment: "node",
    setupFiles: ["tests/setup-wasm.mjs"],
    pool: "threads",
    bail: 0,
    include: ["tests/**/*.test.mjs"],
    fileParallelism: true,
    fsModuleCache: true,
    maxWorkers: Math.min(2, availableParallelism()),
    testTimeout: 0,
    hookTimeout: 0,
    restoreMocks: true,
    unstubGlobals: true,
  },
});
