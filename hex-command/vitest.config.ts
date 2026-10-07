import { defineConfig } from "vitest/config";
import { availableParallelism } from "node:os";

export default defineConfig({
  test: {
    environment: "node",
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
