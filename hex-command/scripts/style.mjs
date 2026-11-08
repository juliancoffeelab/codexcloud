import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";
const args = process.argv.slice(2);
if (
  args.some((arg) => arg !== "--write" && arg !== "--check") ||
  (args.includes("--write") && args.includes("--check"))
) {
  console.error("Usage: npm run format to apply formatting, or npm run format:check.");
  process.exit(1);
}
const write = !args.includes("--check");
const cwd = fileURLToPath(new URL("../", import.meta.url));
const files = [
  "src",
  "packages/save-migrations/src",
  "scripts/*.{mjs,js}",
  "tests/*.mjs",
  "*.config.mjs",
  "*.json",
  "AGENTS.md",
  ".agents/skills/**/*.{md,json,mjs}",
  "pixel/art/factions.json",
  "pixel/art/units/lineups/*.json",
  "pixel/art/units/*/*/palette.json",
  "pixel/art/terrain/*/palette.json",
  "pixel/art/portraits/*/palette.json",
  "docs/*.{md,json}",
  "rust/README.md",
  "rust/data/**/*.{json,md}",
];
const checks = [
  {
    name: "OCaml/Reason formatting",
    command: process.execPath,
    args: ["scripts/frontend-format.mjs", ...(write ? ["--write"] : [])],
  },
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
    args: ["fmt", "--all", "--manifest-path", "rust/Cargo.toml", ...(write ? [] : ["--check"])],
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
const results = await Promise.all(
  checks.map(
    (check) =>
      new Promise((resolve) => {
        const child = spawn(check.command, check.args, {
          cwd,
          stdio: "inherit",
          env: toolchainEnvironment(),
        });
        let failed = false;
        child.on("error", (error) => {
          failed = true;
          console.error(error.message);
        });
        child.on("close", (code, signal) => {
          const passed = !failed && code === 0 && !signal;
          if (!passed) console.error(`FAIL: ${check.name}`);
          resolve(passed);
        });
      }),
  ),
);
process.exitCode = results.every(Boolean) ? 0 : 1;
