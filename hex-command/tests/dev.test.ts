import { spawn } from "node:child_process";
import { once } from "node:events";
import { readFileSync } from "node:fs";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import assert from "node:assert/strict";
import { test } from "vitest";

// Real subprocess fixtures check ownership without invoking any toolchains,
// servers or browser. Readiness is event-driven; timers only keep workers alive.
async function fixture(initialFailure = false) {
  const directory = await mkdtemp(join(tmpdir(), "hex-command-dev-"));
  for (const path of ["scripts", "bin", "node_modules/vite/bin"])
    await mkdir(join(directory, path), { recursive: true });
  await writeFile(join(directory, "package.json"), '{"type":"module"}');
  await writeFile(
    join(directory, "scripts/dev.ts"),
    await readFile(new URL("../scripts/dev.ts", import.meta.url)),
  );
  const worker = `
import { spawn } from 'node:child_process';
const name = process.argv[1].endsWith('/dune') ? 'dune' : 'vite';
if (name === 'dune' && !process.argv.includes('--watch')) {
  if (process.env.INITIAL_FAILURE && process.argv.includes('build')) process.exit(3);
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
  await writeFile(join(directory, "bin/dune"), `#!${process.execPath}\n${worker}`, { mode: 0o755 });
  await writeFile(join(directory, "bin/npm"), "#!/bin/sh\nexit 0\n", { mode: 0o755 });
  await writeFile(join(directory, "node_modules/vite/bin/vite.js"), worker);
  const child = spawn(
    process.execPath,
    ["--import", import.meta.resolve("tsx"), "scripts/dev.ts", "--port", "5199"],
    {
      cwd: directory,
      env: {
        ...process.env,
        PATH: `${join(directory, "bin")}:${process.env.PATH}`,
        INITIAL_FAILURE: initialFailure ? "1" : "",
      },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const closed = once(child, "close");
  let output = "";
  let stdout = "";
  type Worker = { name: string; pid: number; helper: number; args: string[] };
  const workers: Worker[] = [];
  const forceStop = () => {
    for (const worker of workers) {
      try {
        process.kill(-worker.pid, "SIGKILL");
      } catch (error) {
        if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error;
      }
    }
    child.kill("SIGKILL");
  };
  // Independent of the supervisor under test, including readiness/shutdown waits.
  const lifetime = setTimeout(() => {
    output += "Development fixture exceeded its four-second lifetime.";
    forceStop();
  }, 4_000);
  let readyResolve: () => void;
  const ready = new Promise<void>((resolve) => {
    readyResolve = resolve;
  });
  child.stdout.setEncoding("utf8");
  child.stdout.on("data", (chunk: string) => {
    output += chunk;
    stdout += chunk;
    let newline: number;
    while ((newline = stdout.indexOf("\n")) !== -1) {
      const line = stdout.slice(0, newline);
      stdout = stdout.slice(newline + 1);
      if (line.startsWith("READY ")) workers.push(JSON.parse(line.slice(6)) as Worker);
    }
    if (workers.length === 2) readyResolve();
  });
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk: string) => {
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
      const watchdog = setTimeout(forceStop, 1_000);
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

function assertStopped(pid: number): void {
  // A killed descendant can briefly remain a zombie until the OS reaps it.
  try {
    process.kill(pid, 0);
    if (process.platform === "linux") {
      const status = readFileSync(`/proc/${pid}/stat`, "utf8");
      assert.equal(status.slice(status.lastIndexOf(")") + 2, status.lastIndexOf(")") + 3), "Z");
    } else assert.fail(`Worker ${pid} remains alive`);
  } catch (error) {
    if (
      (error as NodeJS.ErrnoException).code !== "ESRCH" &&
      (error as NodeJS.ErrnoException).code !== "ENOENT"
    )
      throw error;
  }
}

test.skipIf(process.platform === "win32")(
  "Development forwards Vite options and releases both process trees on interruption",
  async () => {
    const dev = await fixture();
    try {
      await dev.ready();
      assert.deepEqual(dev.workers.find(({ name }) => name === "vite")!.args.slice(-2), [
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
        process.kill(dev.workers.find(({ name }) => name === service)!.pid, "SIGUSR1");
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

test.skipIf(process.platform === "win32")(
  "Initial preparation failure starts neither watcher nor server",
  async () => {
    const dev = await fixture(true);
    try {
      assert.equal((await dev.closed)[0], 1);
      assert.equal(dev.workers.length, 0);
      assert.match(dev.output(), /Preparing OCaml stopped \(exit 3\)/);
    } finally {
      await dev.cleanup();
    }
  },
);
