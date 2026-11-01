/** JSON-lines transport for the configured Sites operations. Tool results stay in memory. */
import readline from "node:readline";
import { pathToFileURL } from "node:url";
import publishSite, { prepareSite } from "./publish-sites-call.js";

export async function runSiteOperation(tools, options, operations = { prepareSite, publishSite }) {
  if (!["prepare", "deploy"].includes(options.mode)) throw new Error("Expected prepare or deploy");
  const receipt = await operations.prepareSite(tools, {
    cwd: options.cwd,
    message: options.message,
  });
  if (!receipt?.project_id || !receipt?.version_id)
    throw new Error("Preparation returned no release receipt");
  return options.mode === "prepare" ? receipt : operations.publishSite(tools, receipt);
}

async function main() {
  const mode = process.argv[2];
  if (mode === "--help") {
    process.stdout.write(
      "Usage: node scripts/site-operation.mjs prepare|deploy\nJSON-lines tool transport on stdin/stdout; see .agents/skills/publish/operations.md.\n",
    );
    return;
  }
  if (!["prepare", "deploy"].includes(mode)) throw new Error("Expected prepare or deploy");
  // A PTY is required for persistent stdin in the execution tool. Disable its echo
  // so credential-bearing tool responses cannot be echoed into a later request.
  if (process.stdin.isTTY) process.stdin.setRawMode(true);
  const input = readline.createInterface({ input: process.stdin, terminal: false });
  const pending = new Map();
  let next = 0;
  const send = (value) => process.stdout.write(JSON.stringify(value) + "\n");
  input.on("line", (line) => {
    try {
      const response = JSON.parse(line);
      const waiter = pending.get(response.id);
      if (!waiter) return;
      pending.delete(response.id);
      if (response.error) waiter.reject(new Error(response.error));
      else waiter.resolve(response.result);
    } catch {
      for (const waiter of pending.values()) waiter.reject(new Error("Invalid tool response JSON"));
      pending.clear();
    }
  });
  input.on("close", () => {
    for (const waiter of pending.values()) waiter.reject(new Error("Tool transport closed"));
    pending.clear();
  });
  const tools = new Proxy(
    {},
    {
      get: (_target, name) => (args) =>
        new Promise((resolve, reject) => {
          const id = ++next;
          pending.set(id, { resolve, reject });
          send({ type: "tool", id, name, args });
        }),
    },
  );
  try {
    const result = await runSiteOperation(tools, { mode, cwd: process.cwd() });
    send({ type: "result", result });
  } catch (error) {
    send({ type: "error", error: error.message });
    process.exitCode = 1;
  } finally {
    if (process.stdin.isTTY) process.stdin.setRawMode(false);
    input.close();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    process.stdout.write(JSON.stringify({ type: "error", error: error.message }) + "\n");
    process.exitCode = 1;
  });
}
