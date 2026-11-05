/** JSON-lines transport for the configured Sites operations. Tool results stay in memory. */
import readline from "node:readline";
import { pathToFileURL } from "node:url";
import { spawn } from "node:child_process";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { createHash } from "node:crypto";
import publishSite, { prepareSite } from "./publish-sites-call.js";

export async function runSiteOperation(tools, options, operations = { prepareSite, publishSite }) {
  if (!["prepare", "deploy"].includes(options.mode)) throw new Error("Expected prepare or deploy");
  let receipt;
  try {
    receipt = await operations.prepareSite(tools, options);
    if (!receipt?.project_id || !receipt?.version_id)
      throw new Error("Preparation returned no release receipt");
    if (options.mode === "prepare") return receipt;
    try {
      return await operations.publishSite(tools, receipt);
    } catch (error) {
      // A deleted saved version invalidates only that exact cached receipt.
      if (!receipt.reused || !/not found|unknown version|does not exist/i.test(error.message))
        throw error;
      receipt = await operations.prepareSite(tools, {
        ...options,
        refresh: true,
        source_commit: receipt.source_commit,
      });
      return await operations.publishSite(tools, receipt);
    }
  } catch (error) {
    const url = error.url ?? receipt?.live_url;
    if (options.mode !== "deploy" || !url) throw error;
    // Keep the known address useful without presenting the old publication as
    // the requested release. Only native success sets deployed:true.
    return {
      url,
      deployed: false,
      error: error.message,
      source_commit: error.source_commit ?? receipt?.source_commit,
    };
  }
}

async function main() {
  const [mode, ...flags] = process.argv.slice(2);
  if (mode === "--help") {
    process.stdout.write(
      "Usage: node scripts/site-operation.mjs prepare|deploy [--preview PR]\nJSON-lines tool transport on stdin/stdout; see .agents/skills/publish/operations.md.\n",
    );
    return;
  }
  if (!["prepare", "deploy"].includes(mode)) throw new Error("Expected prepare or deploy");
  let preview;
  if (flags.length) {
    if (flags.length !== 2 || flags[0] !== "--preview" || !/^[1-9][0-9]*$/.test(flags[1]) || !Number.isSafeInteger(Number(flags[1])))
      throw new Error("Expected --preview followed by a positive PR number");
    preview = Number(flags[1]);
  }
  // Serialize the complete save/deploy sequence, not just the build: another
  // invocation must not advance the Site branch between our push and save.
  if (process.env.HEX_COMMAND_SITE_LOCKED !== "1") {
    const key = createHash("sha256").update(`${process.cwd()}:${preview ?? "main"}`).digest("hex");
    const child = spawn(
      "flock",
      [
        "-x",
        join(tmpdir(), `hex-command-site-${key}.lock`),
        process.execPath,
        process.argv[1],
        mode,
        ...flags,
      ],
      {
        stdio: "inherit",
        env: { ...process.env, HEX_COMMAND_SITE_LOCKED: "1" },
      },
    );
    process.exitCode = await new Promise((resolve, reject) => {
      child.on("error", reject);
      child.on("close", (code) => resolve(code ?? 1));
    });
    return;
  }
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
    const result = await runSiteOperation(tools, { mode, cwd: process.cwd(), preview });
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
