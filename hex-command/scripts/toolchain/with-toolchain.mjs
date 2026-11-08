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
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
