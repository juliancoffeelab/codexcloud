import { spawn } from "node:child_process";
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
  "docs/*.{md,json}",
  "rust/README.md",
  "rust/data/**/*.{json,md}",
];
const checks = [
  {
    name: "OCaml/Reason formatting",
    command: process.execPath,
    args: ["--import", "tsx", "scripts/frontend-format.ts", ...(write ? ["--write"] : [])],
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
const results = await Promise.all(
  checks.map(
    (check) =>
      new Promise<boolean>((resolve) => {
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
