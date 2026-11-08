import assert from "node:assert/strict";
import { Worker } from "node:worker_threads";
import { pathToFileURL } from "node:url";

export async function workerResult(worker, timeoutMs = 5000) {
  let timeout;
  try {
    return await new Promise((resolve, reject) => {
      timeout = setTimeout(
        () =>
          reject(
            new Error(`worker result deadline exceeded (${timeoutMs} ms)`),
          ),
        timeoutMs,
      );
      worker.once("message", resolve);
      worker.once("error", reject);
      worker.once("exit", (code) =>
        reject(new Error(`worker exited before result: ${code}`)),
      );
    });
  } finally {
    clearTimeout(timeout);
    await worker.terminate();
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const result = await workerResult(
    new Worker(
      new URL("../demos/async-search/tests/worker-node.mjs", import.meta.url),
    ),
  );
  assert.deepEqual(result, { kind: "passed", chars: 2 });
  console.log(
    "worker: ordinary compiled bindings await, borrow and clean up in a separate realm",
  );
}
