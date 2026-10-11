import { defineConfig } from "vitest/config";
import { frontendAliases } from "./scripts/frontend-paths";
import { availableParallelism } from "node:os";

export default defineConfig({
  resolve: { alias: frontendAliases },
  test: {
    environment: "node",
    setupFiles: ["tests/setup-wasm.ts"],
    pool: "threads",
    bail: 0,
    include: ["tests/**/*.test.ts"],
    fileParallelism: true,
    maxWorkers: Math.min(4, availableParallelism()),
    testTimeout: 5_000,
    hookTimeout: 5_000,
    restoreMocks: true,
    unstubGlobals: true,
  },
});
