import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { execFileSync } from "node:child_process";
import { arenaMatchups } from "../src/game/ai/matchups";
import { POLICY_IDS } from "../src/game/ai/policies";
import { runArena } from "../src/game/ai/simulation";
import type { PhaseTiming, ArenaResult } from "../src/game/ai/simulation";

// A bounded timing sample, not a balance tournament. Run only on explicit request.
const output = resolve(process.argv[2] ?? "artifacts/ai-timing.json");
mkdirSync(dirname(output), { recursive: true });
const configs = arenaMatchups({
  seed: 101,
  maxRounds: 15,
  recruitLimit: 1,
  armies: ["human"],
  landscapes: ["open"],
  policies: POLICY_IDS,
});
const matches: { elapsedMs: number; timings: PhaseTiming[]; result: ArenaResult }[] = [];
const sourceCommit = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
const sampleStarted = performance.now();
for (const config of configs) {
  const timings: PhaseTiming[] = [];
  const started = performance.now();
  const result = runArena(config, (timing) => timings.push(timing));
  const elapsedMs = performance.now() - started;
  matches.push({ elapsedMs, timings, result });
  // Preserve complete match traces immediately; interruption cannot erase earlier matches.
  writeFileSync(
    output,
    JSON.stringify({ sourceCommit, status: "running", planned: configs.length, matches }, null, 2),
  );
  console.log(
    JSON.stringify({
      match: matches.length,
      total: configs.length,
      player: config.player,
      enemy: config.enemy,
      first: config.first,
      seconds: +(elapsedMs / 1000).toFixed(3),
      rounds: result.rounds,
      reason: result.reason,
    }),
  );
}
function distribution(values: number[]) {
  const sorted = [...values].sort((a, b) => a - b);
  const percentile = (fraction: number) =>
    sorted[Math.max(0, Math.ceil(sorted.length * fraction) - 1)]!;
  return {
    count: values.length,
    mean: values.reduce((sum, value) => sum + value, 0) / values.length,
    median: percentile(0.5),
    p90: percentile(0.9),
    min: sorted[0],
    max: sorted.at(-1),
  };
}
const policyTimings = POLICY_IDS.map((policy) => {
  const phases = matches
    .flatMap((match) => match.timings)
    .filter((phase) => phase.policy === policy);
  return {
    policy,
    millisecondsPerTurn: distribution(phases.map((phase) => phase.elapsedMs)),
    millisecondsPerStartingUnit:
      phases.reduce((sum, phase) => sum + phase.elapsedMs, 0) /
      phases.reduce((sum, phase) => sum + phase.units, 0),
  };
});
const matchups = POLICY_IDS.flatMap((player, a) =>
  POLICY_IDS.slice(a).map((enemy) => {
    const sample = matches.filter(
      ({ result: { config } }) =>
        (config.player === player && config.enemy === enemy) ||
        (config.player === enemy && config.enemy === player),
    );
    return {
      policies: [player, enemy],
      seconds: distribution(sample.map((match) => match.elapsedMs / 1000)),
      rounds: distribution(sample.map((match) => match.result.rounds)),
      capped: sample.filter((match) => match.result.reason === "cap").length,
    };
  }),
);
const report = {
  sourceCommit,
  status: "complete",
  planned: configs.length,
  elapsedSeconds: (performance.now() - sampleStarted) / 1000,
  policyTimings,
  matchups,
  matches,
};
writeFileSync(output, JSON.stringify(report, null, 2));
console.log(
  JSON.stringify({ output, elapsedSeconds: report.elapsedSeconds, policyTimings, matchups }),
);
