import Benchmark, { maxTime } from "./benchmark-options.mjs";
import { spawn } from "node:child_process";
import { readFile, writeFile, mkdir } from "node:fs/promises";
import { createInterface } from "node:readline";
import { resolve, dirname } from "node:path";
const args = process.argv.slice(2),
  option = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
if (!option("--positions")) throw Error("Supply --positions FILE");
const root = resolve(import.meta.dirname, ".."),
  binary = resolve(option("--binary", "rust/target/release/hex-command-sim"));
const positions = JSON.parse(await readFile(resolve(option("--positions"))));
const child = spawn(binary, ["--data-dir", resolve(root, "rust/data"), "--phase-stdio"], {
  stdio: ["pipe", "pipe", "inherit"],
});
const lines = createInterface({ input: child.stdout })[Symbol.asyncIterator]();
const suite = new Benchmark.Suite(),
  results = [];
let failure;
for (const scope of ["arena", "campaign"]) {
  const candidates = positions.filter((p) => p.campaign === (scope === "campaign"));
  const selected = args.includes("--full") ? candidates : candidates.slice(0, 1);
  if (!selected.length) continue;
  for (const planner of ["full", "queued"]) {
    const name = `${scope}/${planner}`;
    if (option("--case") && !name.includes(option("--case"))) continue;
    const payloads = selected.map(
      ({ label, campaign, ...p }) =>
        JSON.stringify({
          ...p,
          policy: "balanced",
          blocks: { recruiter: "balanced", campaign: "mission", planner },
        }) + "\n",
    );
    const call = async () => {
      for (const payload of payloads) {
        child.stdin.write(payload);
        const result = await lines.next();
        if (result.done) throw Error("Native process terminated");
      }
    };
    suite.add(name, {
      defer: true,
      fn(deferred) {
        call().then(
          () => deferred.resolve(),
          (error) => {
            failure = error;
            suite.abort();
            deferred.resolve();
          },
        );
      },
    });
  }
}
try {
  await new Promise((done, reject) =>
    suite
      .on("cycle", (e) => {
        const b = e.target;
        console.log(String(b));
        results.push({
          name: b.name,
          hz: b.hz,
          stats: b.stats,
          times: b.times,
          count: b.count,
          cycles: b.cycles,
        });
      })
      .on("error", (e) => reject(e.target.error))
      .on("complete", () => (failure ? reject(failure) : done()))
      .run({ async: true }),
  );
  const out = resolve(option("--out", "tmp/policy-benchmarkjs.json"));
  await mkdir(dirname(out), { recursive: true });
  await writeFile(
    out,
    JSON.stringify(
      {
        framework: `Benchmark.js ${Benchmark.version}`,
        options: { maxTime },
        operation: "Entire policy-position corpus via persistent native stdio",
        results,
      },
      null,
      2,
    ) + "\n",
  );
} finally {
  child.stdin.end();
  child.kill();
}
