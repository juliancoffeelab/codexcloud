import { fileURLToPath } from "node:url";
/** Source tools and tests resolve emitted OCaml modules and foreign data only. */
export const frontendAliases = Object.fromEntries(
  [
    ["@frontend", "../_build/default/frontend/output/frontend"],
    ["melange", "../_build/default/frontend/output/node_modules/melange"],
    ["@wasm/hex_command", "../rust/pkg/hex_command.js"],
    ["@assets/pixel-art", "../src/presentation/pixel-art.ts"],
    ["@save-history", "../src/save/migration/legacy/data"],
  ].map(([name, path]) => [name, fileURLToPath(new URL(path!, import.meta.url))]),
);
