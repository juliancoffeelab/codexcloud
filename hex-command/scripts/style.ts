import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "../.agents/skills/rust-toolchain/scripts/toolchain-env";

const args = process.argv.slice(2);
if (args.some((arg) => arg !== "--write")) {
  console.error("Usage: npm run style, or npm run format to apply formatting.");
  process.exit(1);
}
const write = args.includes("--write");
const cwd = fileURLToPath(new URL("../", import.meta.url));
const files = [
  "src",
  "scripts/*.{ts,js}",
  "tests/*.ts",
  "*.config.ts",
  "*.json",
  "README.md",
  "AGENTS.md",
  ".agents/skills/**/*.{md,json,ts}",
  "TODO.md",
  "pixel/art/factions.json",
  "pixel/art/units/lineups/*.json",
  "pixel/art/units/*/*/palette.json",
  "pixel/art/terrain/*/palette.json",
  "pixel/art/portraits/*/palette.json",
  "art/styled/**/*.{json,md}",
  "docs/ai-simulation.md",
  "docs/save-deployment.md",
  "rust/README.md",
  "rust/data/**/*.{json,md}",
  "design/static-terrain-art.md",
];
const checks = [
  {
    name: "Prettier",
    command: process.execPath,
    args: [
      fileURLToPath(new URL("../node_modules/prettier/bin/prettier.cjs", import.meta.url)),
      write ? "--write" : "--check",
      "--cache",
      "--cache-location",
      "node_modules/.cache/hex-command-prettier",
      ...files,
    ],
  },
  {
    name: "Rust formatting",
    command: "cargo",
    args: ["fmt", "--manifest-path", "rust/Cargo.toml", ...(write ? [] : ["--check"])],
  },
  ...(!write
    ? [
        { name: "Unstaged whitespace", command: "git", args: ["diff", "--check", "--", "."] },
        {
          name: "Staged whitespace",
          command: "git",
          args: ["diff", "--cached", "--check", "--", "."],
        },
      ]
    : []),
];
let failed = false;
for (const check of checks) {
  const result = spawnSync(check.command, check.args, {
    cwd,
    stdio: "inherit",
    env: toolchainEnvironment(),
  });
  if (result.status !== 0 || result.error) {
    failed = true;
    console.error(`FAIL: ${check.name}`);
    if (result.error) console.error(result.error.message);
  }
}
process.exit(failed ? 1 : 0);
