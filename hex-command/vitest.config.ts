import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    pool: "threads",
    bail: 0,
    include: ["tests/**/*.test.ts"],
    fileParallelism: true,
    maxWorkers: 2,
    testTimeout: 5_000,
    hookTimeout: 5_000,
    restoreMocks: true,
    unstubGlobals: true,
  },
});
