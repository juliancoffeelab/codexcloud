#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { toolchainCommand, toolchainEnvironment } from "./toolchain-env.mjs";
const [command, ...args] = process.argv.slice(2);
if (!command) {
  console.error("Usage: with-toolchain.mjs <command> [arguments]");
  process.exit(1);
}
const result = spawnSync(toolchainCommand(command), args, {
  env: toolchainEnvironment(),
  stdio: "inherit",
});
if (result.error) {
  console.error(result.error.message);
  if (result.error.code === "ENOENT")
    console.error(
      `${command} could not start after tool discovery. Check the executable and interpreter on PATH; supplied cloud tools are resolved by scripts/toolchain/toolchain-env.mjs.`,
    );
}
process.exit(result.status ?? 1);
