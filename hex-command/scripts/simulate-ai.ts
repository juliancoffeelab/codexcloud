/** npm entry point only: scheduling, execution and reports belong to the Rust CLI. */
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { resolve } from "node:path";

const child = spawn(
  resolve(
    fileURLToPath(new URL("../", import.meta.url)),
    process.env.CARGO_TARGET_DIR ?? "rust/target",
    "release",
    "hex-command-sim" + (process.platform === "win32" ? ".exe" : ""),
  ),
  process.argv.slice(2),
  { cwd: fileURLToPath(new URL("../", import.meta.url)), stdio: "inherit" },
);
const forward = (signal: NodeJS.Signals) => child.kill(signal);
process.on("SIGINT", forward);
process.on("SIGTERM", forward);
child.once("error", (error) => {
  console.error(`Cannot launch native simulation: ${error.message}`);
  process.exitCode = 1;
});
child.once("exit", (code, signal) => {
  process.off("SIGINT", forward);
  process.off("SIGTERM", forward);
  process.exitCode = code ?? (signal === "SIGINT" ? 130 : signal === "SIGTERM" ? 143 : 1);
});
