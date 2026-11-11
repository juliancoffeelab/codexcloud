import { execFileSync } from "node:child_process";
import { relative } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import { frontendAliases, assertBuildProfile } from "./scripts/frontend-paths.mjs";
import { terrainPrebuild } from "./scripts/terrain-prebuild.mjs";
import { artworkPlugin } from "./scripts/artwork-plugin.mjs";
const projectRoot = fileURLToPath(new URL(".", import.meta.url));
const watchedOutputs = [
  "_build/default/frontend/output",
  "_build/default/wasm",
  "_build/default/artwork",
];
export default defineConfig(({ command, isPreview }) => {
  if (!isPreview) assertBuildProfile(command === "build" ? "release" : "dev");
  return {
    plugins: [artworkPlugin(), terrainPrebuild()],
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
            !watchedOutputs.some(
              (output) =>
                local === output ||
                local.startsWith(output + "/") ||
                output.startsWith(local + "/"),
            )
          );
        },
      },
    },
    resolve: { alias: frontendAliases },
    build: {
      target: "es2022",
    },
  };
});
