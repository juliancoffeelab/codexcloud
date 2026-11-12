import { spawn } from "node:child_process";
import { once } from "node:events";
import { readFileSync } from "node:fs";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import assert from "node:assert/strict";
import { test } from "vitest";
async function fixture() {
  const directory = await mkdtemp(join(tmpdir(), "hex-command-dev-"));
  for (const path of ["scripts", "bin", "node_modules/vite/bin"])
    await mkdir(join(directory, path), { recursive: true });
  await writeFile(join(directory, "package.json"), '{"type":"module"}');
  await writeFile(
    join(directory, "scripts/dev.mjs"),
    await readFile(new URL("../scripts/dev.mjs", import.meta.url)),
  );
  const worker = `
import { spawn } from 'node:child_process';
const name = process.argv[1].endsWith('/dune') ? 'dune' : 'vite';
if (name === 'dune' && !process.argv.includes('--watch')) {
  process.exit(0);
}
const helper = spawn(process.execPath, ['-e', 'setInterval(() => {}, 60000)']);
helper.on('spawn', () => console.log('READY ' + JSON.stringify({name, pid:process.pid, helper:helper.pid, args:process.argv.slice(2)})));
const hold = setInterval(() => {}, 60000);
process.on('SIGTERM', () => {
  clearInterval(hold);
  helper.kill('SIGTERM');
  helper.on('close', () => process.exit(0));
});
process.on('SIGUSR1', () => process.exit(7));
`;
  await writeFile(
    join(directory, "bin/dune"),
    `#!${process.execPath}
${worker}`,
    { mode: 493 },
  );
  await writeFile(join(directory, "node_modules/vite/bin/vite.js"), worker);
  const child = spawn(process.execPath, ["scripts/dev.mjs", "--port", "5199"], {
    cwd: directory,
    env: {
      ...process.env,
      PATH: `${join(directory, "bin")}:${process.env.PATH}`,
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
  const closed = once(child, "close");
  let output = "";
  let stdout = "";
  const workers = [];
  const forceStop = () => {
    for (const worker2 of workers) {
      try {
        process.kill(-worker2.pid, "SIGKILL");
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    }
    child.kill("SIGKILL");
  };
  const lifetime = setTimeout(() => {
    output += "Development fixture exceeded its four-second lifetime.";
    forceStop();
  }, 4e3);
  let readyResolve;
  const ready = new Promise((resolve) => {
    readyResolve = resolve;
  });
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    output += chunk;
    stdout += chunk;
    let newline;
    while ((newline = stdout.indexOf("\n")) !== -1) {
      const line = stdout.slice(0, newline);
      stdout = stdout.slice(newline + 1);
      if (line.startsWith("READY ")) workers.push(JSON.parse(line.slice(6)));
    }
    if (workers.length === 2) readyResolve();
  });
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => {
    output += chunk;
  });
  return {
    child,
    closed,
    workers,
    output: () => output,
    ready: () =>
      Promise.race([
        ready,
        closed.then(() => {
          throw new Error(output);
        }),
      ]),
    async cleanup() {
      const watchdog = setTimeout(forceStop, 1e3);
      if (child.exitCode === null && child.signalCode === null) child.kill("SIGTERM");
      try {
        await closed;
      } finally {
        clearTimeout(lifetime);
        clearTimeout(watchdog);
        forceStop();
      }
      await rm(directory, { recursive: true, force: true });
    },
  };
}
function assertStopped(pid) {
  try {
    process.kill(pid, 0);
    if (process.platform === "linux") {
      const status = readFileSync(`/proc/${pid}/stat`, "utf8");
      assert.equal(status.slice(status.lastIndexOf(")") + 2, status.lastIndexOf(")") + 3), "Z");
    } else assert.fail(`Worker ${pid} remains alive`);
  } catch (error) {
    if (error.code !== "ESRCH" && error.code !== "ENOENT") throw error;
  }
}
test.skipIf(process.platform === "win32")(
  "Development forwards Vite options and releases both process trees on interruption",
  async () => {
    const dev = await fixture();
    try {
      await dev.ready();
      assert.deepEqual(dev.workers.find(({ name }) => name === "vite").args.slice(-2), [
        "--port",
        "5199",
      ]);
      dev.child.kill("SIGINT");
      assert.equal((await dev.closed)[0], 130);
      for (const worker of dev.workers) {
        assertStopped(worker.pid);
        assertStopped(worker.helper);
      }
    } finally {
      await dev.cleanup();
    }
  },
);
for (const service of ["dune", "vite"])
  test.skipIf(process.platform === "win32")(
    `A stopped ${service} fails development and releases its sibling`,
    async () => {
      const dev = await fixture();
      try {
        await dev.ready();
        process.kill(dev.workers.find(({ name }) => name === service).pid, "SIGUSR1");
        assert.equal((await dev.closed)[0], 1);
        assert.match(dev.output(), /stopped \(exit 7\)/);
        for (const worker of dev.workers) {
          assertStopped(worker.pid);
          assertStopped(worker.helper);
        }
      } finally {
        await dev.cleanup();
      }
    },
  );
