/** Bounded native samples; Rust owns definitions, schedule generation and execution. */
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { createInterface } from "node:readline";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";

type Config = {
  seed: number;
  player: string;
  enemy: string;
  campaign?: string;
  diplomacy?: string;
  armies?: { player: string; enemy: string };
  landscape?: string;
  first?: string;
};
const args = process.argv.slice(2);
const option = (flag: string, fallback?: string) => {
  const index = args.indexOf(flag);
  return index < 0 ? fallback : args[index + 1];
};
const cwd = fileURLToPath(new URL("../", import.meta.url));
const binary = resolve(cwd, "rust/target/release/hex-command-sim");
const alternate = args.includes("--alternate");
const schedule = JSON.parse(
  execFileSync(
    binary,
    ["--schedule-only", "--mode", "full", "--seeds", "1", "--seed", alternate ? "29" : "17"],
    { cwd, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  ),
) as { configs: Config[] };
const representatives = new Map<string, Config>();
for (const config of schedule.configs) {
  if (config.campaign) {
    if (
      config.enemy !== (alternate ? "mixed" : "classic") ||
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
  limit = Number(option("--limit", "60"));
if (
  !Number.isSafeInteger(skip) ||
  skip < 0 ||
  !Number.isSafeInteger(limit) ||
  limit < 1 ||
  limit > 60
)
  throw new Error("--skip must be nonnegative; --limit must be 1..60");
const configs = [...representatives.values()].slice(skip, skip + limit);
const output = resolve(option("--out", "artifacts/ai-performance/native-sample.json")!);
const baseline = option("--compare")
  ? (JSON.parse(readFileSync(resolve(option("--compare")!), "utf8")) as {
      matches: { config: Config; hash: string }[];
    })
  : undefined;
if (
  baseline &&
  !isDeepStrictEqual(
    baseline.matches.map((entry) => entry.config),
    configs,
  )
)
  throw new Error("Comparison requires identical sample configurations");
const processHandle = spawn(binary, ["--stdio"], { cwd, stdio: ["pipe", "pipe", "inherit"] });
const lines = createInterface({ input: processHandle.stdout })[Symbol.asyncIterator]();
let failure: Error | undefined;
processHandle.once("error", (error) => {
  failure = error;
});
const matches: { config: Config; rustMs: number; hash: string }[] = [];
try {
  for (const [index, config] of configs.entries()) {
    const started = performance.now();
    processHandle.stdin.write(JSON.stringify(config) + "\n");
    const result = await lines.next();
    if (result.done) throw failure ?? new Error("Native sample process terminated");
    const parsed: unknown = JSON.parse(result.value);
    const hash = createHash("sha256").update(JSON.stringify(parsed)).digest("hex");
    const entry = { config, rustMs: performance.now() - started, hash };
    matches.push(entry);
    if (baseline && baseline.matches[index]?.hash !== hash)
      throw new Error(`Complete result changed at sample ${index + 1}`);
    console.log(JSON.stringify(entry));
  }
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(
    output,
    JSON.stringify(
      { matches, rustMs: matches.reduce((sum, entry) => sum + entry.rustMs, 0) },
      null,
      2,
    ) + "\n",
  );
} finally {
  processHandle.stdin.end();
  processHandle.kill();
}
