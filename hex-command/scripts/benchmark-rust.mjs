import Benchmark, { maxTime } from "./benchmark-options.mjs";
import { execFileSync, spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
const args = process.argv.slice(2),
  option = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
const cwd = resolve(import.meta.dirname, "..");
const binary = resolve(
  option(
    "--binary",
    resolve(
      cwd,
      process.env.CARGO_TARGET_DIR ?? "rust/target",
      "release",
      `hex-command-sim${process.platform === "win32" ? ".exe" : ""}`,
    ),
  ),
);
const alternate = args.includes("--alternate");
const schedule = JSON.parse(
  execFileSync(
    binary,
    [
      "--schedule-only",
      "--mode",
      "full",
      "--all-pairs",
      "--policies",
      "classic,rush,balanced,rotated,situational,predictor",
      "--campaign-players",
      "classic,rush,balanced,rotated,situational,predictor",
      "--campaign-enemies",
      "classic,rush,balanced,rotated,situational,predictor",
      "--seeds",
      "1",
      "--seed",
      alternate ? "29" : "17",
    ],
    { cwd, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  ),
);
const representatives = /* @__PURE__ */ new Map();
for (const config of schedule.configs) {
  if (config.campaign) {
    if (
      config.enemy !== (alternate ? "balanced" : "classic") ||
      config.diplomacy !== (alternate ? "pragmatic" : "restrained")
    )
      continue;
  } else if (
    config.first !== (alternate ? "enemy" : "player") ||
    config.armies?.player === config.armies?.enemy
  )
    continue;
  const key = config.campaign
    ? `${config.campaign}:${config.player}`
    : `${config.armies?.player}:${config.landscape}:${config.player}`;
  if (!representatives.has(key)) representatives.set(key, config);
}
const skip = Number(option("--skip", "0")),
  limit = Number(option("--limit", args.includes("--full") ? "60" : "1"));
if (
  !Number.isSafeInteger(skip) ||
  skip < 0 ||
  !Number.isSafeInteger(limit) ||
  limit < 1 ||
  limit > 60
)
  throw new Error("--skip must be nonnegative; --limit must be 1..60");
const configs = [...representatives.values()].slice(skip, skip + limit);
const output = resolve(option("--out", "artifacts/ai-performance/native-sample.json"));
const child = spawn(binary, ["--stdio"], { cwd, stdio: ["pipe", "pipe", "inherit"] });
const lines = createInterface({ input: child.stdout })[Symbol.asyncIterator]();
const suite = new Benchmark.Suite(),
  matches = [];
const baseline = option("--compare")
  ? JSON.parse(readFileSync(resolve(option("--compare")), "utf8"))
  : null;
let failure;
for (const [index, config] of configs.entries()) {
  const payload = JSON.stringify(config) + "\n";
  suite.add(`native match/${index}`, {
    defer: true,
    fn(deferred) {
      child.stdin.write(payload);
      lines.next().then(
        (result) => {
          if (result.done) {
            failure = Error("Native process terminated");
            suite.abort();
          }
          deferred.resolve();
        },
        (error) => {
          failure = error;
          suite.abort();
          deferred.resolve();
        },
      );
    },
  });
}
try {
  await new Promise((done, reject) =>
    suite
      .on("cycle", (e) => {
        const b = e.target,
          index = Number(b.name.split("/").at(-1));
        console.log(String(b));
        const entry = {
          config: configs[index],
          hz: b.hz,
          stats: b.stats,
          times: b.times,
          count: b.count,
          cycles: b.cycles,
        };
        const previous = baseline?.matches?.find(
          (r) => JSON.stringify(r.config) === JSON.stringify(entry.config),
        );
        if (previous?.stats) {
          const saved = new Benchmark();
          saved.stats = previous.stats;
          entry.comparison = b.compare(saved);
        }
        matches.push(entry);
      })
      .on("error", (e) => reject(e.target.error))
      .on("complete", () => (failure ? reject(failure) : done()))
      .run({ async: true }),
  );
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(
    output,
    JSON.stringify(
      {
        framework: `Benchmark.js ${Benchmark.version}`,
        options: { maxTime },
        binary,
        operation:
          "Native match over persistent stdio, including request/response transport; no outcome hashing",
        matches,
      },
      null,
      2,
    ) + "\n",
  );
} finally {
  child.stdin.end();
  child.kill();
}
