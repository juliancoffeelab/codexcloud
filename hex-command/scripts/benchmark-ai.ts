import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { POLICY_IDS } from "../src/game/ai/policies";
import { ARMY_IDS, LANDSCAPES, runArena, runCampaign } from "../src/game/ai/simulation";
import type { ArenaConfig, CampaignConfig, PhaseTiming } from "../src/game/ai/simulation";

// Performance work only: never part of npm test. Hash the complete result,
// including every command, combat, ownership change and final state.
const args = process.argv.slice(2);
const option = (flag: string, fallback: string) => {
  const index = args.indexOf(flag);
  return index < 0 ? fallback : args[index + 1]!;
};
const output = resolve(option("--out", "artifacts/ai-performance/benchmark.json"));
const baselinePath = option("--compare", "");
const baseline = baselinePath
  ? (JSON.parse(readFileSync(baselinePath, "utf8")) as {
      matches: { config: unknown; hash: string }[];
    })
  : undefined;
const configs: (
  { kind: "arena"; config: ArenaConfig } | { kind: "campaign"; config: CampaignConfig }
)[] = [];
for (const [index, army] of ARMY_IDS.entries())
  for (const landscape of LANDSCAPES)
    for (const policy of POLICY_IDS)
      configs.push({
        kind: "arena",
        config: {
          seed: 17,
          armies: { player: army, enemy: ARMY_IDS[(index + 1) % ARMY_IDS.length]! },
          landscape,
          player: policy,
          enemy: policy,
          first: "player",
          maxRounds: 30,
          recruitLimit: 1,
        },
      });
for (const campaign of ["lantern-road", "green-accord", "ashen-oath", "borrowed-tide"] as const)
  for (const policy of POLICY_IDS)
    configs.push({
      kind: "campaign",
      config: {
        seed: 17,
        campaign,
        player: policy,
        enemy: "classic",
        diplomacy: "restrained",
        maxRounds: 30,
        recruitLimit: 1,
      },
    });
const selected = configs.slice(0, Number(option("--limit", String(configs.length))));
const started = performance.now();
const matches = [];
const phases: PhaseTiming[] = [];
mkdirSync(dirname(output), { recursive: true });
for (const entry of selected) {
  const begin = performance.now();
  const result =
    entry.kind === "arena"
      ? runArena(entry.config, (phase) => phases.push(phase))
      : runCampaign(entry.config);
  const elapsedMs = performance.now() - begin;
  const hash = createHash("sha256").update(JSON.stringify(result)).digest("hex");
  const expected = baseline?.matches.find(
    (match) => JSON.stringify(match.config) === JSON.stringify(entry.config),
  );
  if (baseline && (!expected || expected.hash !== hash))
    throw new Error(`Decision/result mismatch: ${JSON.stringify(entry.config)}`);
  matches.push({ ...entry, elapsedMs, hash });
  console.log(
    JSON.stringify({ completed: matches.length, total: selected.length, ...entry, elapsedMs }),
  );
}
const report = {
  elapsedMs: performance.now() - started,
  matches,
  policyTimings: POLICY_IDS.map((policy) => {
    const sample = phases.filter((phase) => phase.policy === policy);
    return {
      policy,
      phases: sample.length,
      totalMs: sample.reduce((sum, phase) => sum + phase.elapsedMs, 0),
    };
  }),
};
writeFileSync(output, JSON.stringify(report, null, 2));
console.log(
  JSON.stringify({ output, elapsedMs: report.elapsedMs, policyTimings: report.policyTimings }),
);
