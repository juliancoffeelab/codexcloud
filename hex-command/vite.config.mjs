import { execFileSync } from "node:child_process";
import { relative } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { frontendAliases } from "./scripts/frontend-paths.mjs";
import { terrainPrebuild } from "./scripts/terrain-prebuild.mjs";
const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const frontendOutput = "_build/default/frontend/output";
export default defineConfig({
  plugins: [terrainPrebuild()],
  optimizeDeps: { entries: ["index.html"] },
  base: "./",
  define: {
    "import.meta.env.VITE_BUILD_COMMIT": JSON.stringify(
      process.env.HEX_COMMAND_SOURCE_COMMIT?.slice(0, 7) ??
        execFileSync("git", ["rev-parse", "--short", "HEAD"], { encoding: "utf8" }).trim(),
    ),
  },
  server: {
    watch: {
      ignored: (path) => {
        const local = relative(projectRoot, path).replaceAll("\\", "/");
        if (/^(?:\.cache|rust\/target|tmp|dist)(?:\/|$)/.test(local)) return true;
        // Watch emitted frontend code, not Dune's downloaded compiler/package trees.
        return (
          local.startsWith("_build/") &&
          local !== frontendOutput &&
          !local.startsWith(frontendOutput + "/") &&
          !frontendOutput.startsWith(local + "/")
        );
      },
    },
  },
  resolve: { alias: frontendAliases },
  build: {
    target: "es2022",
  },
});
