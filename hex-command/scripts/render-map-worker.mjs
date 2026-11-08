// A fresh worker owns one generated runtime and one decoded artwork cache.
import { parentPort, workerData } from "node:worker_threads";
import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
const { root, output } = workerData;
try {
  const client = await import(pathToFileURL(output).href);
  await client.initialize(await readFile(`${root}/rust/pkg/backend.wasm`));
  parentPort.on("message", async () => {
    const started = performance.now();
    try {
      await client.main(root);
      parentPort.postMessage({
        type: "rendered",
        ms: performance.now() - started,
        dependencies: client.dependencies(),
      });
    } catch (error) {
      parentPort.postMessage({
        type: "render-error",
        error: error.stack ?? String(error),
        dependencies: client.dependencies(),
      });
    }
  });
  parentPort.postMessage({ type: "ready" });
} catch (error) {
  parentPort.postMessage({ type: "startup-error", error: error.stack ?? String(error) });
}
