import { RustRunner } from "./rust-runner";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

import { isDeepStrictEqual } from "node:util";
import { runArena, runCampaign, ARMY_IDS, LANDSCAPES } from "../src/game/ai/simulation";
import type { ArenaConfig, CampaignConfig } from "../src/game/ai/simulation";
import { POLICY_IDS } from "../src/game/ai/policies";
const args = process.argv.slice(2);
const option = (key: string, fallback: string) => {
  const i = args.indexOf(key);
  return i < 0 ? fallback : args[i + 1]!;
};
const output = resolve(option("--out", "artifacts/ai-performance/rust-sample.json"));
mkdirSync(resolve(output, ".."), { recursive: true });
const native = new RustRunner();
const configs: (ArenaConfig | CampaignConfig)[] = [];
for (const [index, army] of ARMY_IDS.entries())
  for (const landscape of LANDSCAPES)
    for (const policy of POLICY_IDS)
      configs.push({
        seed: 17,
        armies: { player: army, enemy: ARMY_IDS[(index + 1) % ARMY_IDS.length]! },
        landscape,
        player: policy,
        enemy: policy,
        first: "player",
        maxRounds: 30,
        recruitLimit: 1,
      });
for (const campaign of ["lantern-road", "green-accord", "ashen-oath", "borrowed-tide"] as const)
  for (const policy of POLICY_IDS)
    configs.push({
      seed: 17,
      campaign,
      player: policy,
      enemy: "classic",
      diplomacy: "restrained",
      maxRounds: 30,
      recruitLimit: 1,
    });
if (args.includes("--alternate")) {
  for (const config of configs) {
    config.seed = 29;
    config.enemy = POLICY_IDS[(POLICY_IDS.indexOf(config.player) + 1) % POLICY_IDS.length]!;
    if ("campaign" in config) config.diplomacy = "pragmatic";
    else config.first = "enemy";
  }
}
function difference(a: unknown, b: unknown, path = ""): string | undefined {
  if (isDeepStrictEqual(a, b)) return;
  if (a && b && typeof a === "object" && typeof b === "object") {
    for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
      const d = difference(
        (a as Record<string, unknown>)[k],
        (b as Record<string, unknown>)[k],
        `${path}/${k}`,
      );
      if (d) return d;
    }
  }
  return `${path}: TS=${JSON.stringify(a)} Rust=${JSON.stringify(b)}`;
}
const results = [];
try {
  for (const config of configs.slice(
    Number(option("--skip", "0")),
    Number(option("--skip", "0")) + Number(option("--limit", "60")),
  )) {
    const started = performance.now();
    const reference = "campaign" in config ? runCampaign(config) : runArena(config);
    const typescriptMs = performance.now() - started;
    const nativeStart = performance.now();
    const actual = await native.run(config);
    const rustMs = performance.now() - nativeStart;
    const diff = difference(reference, actual);
    results.push({ config, typescriptMs, rustMs, identical: !diff });
    console.log(JSON.stringify(results.at(-1)));
    if (diff) {
      writeFileSync(output.replace(/\.json$/, ".expected.json"), JSON.stringify(reference));
      writeFileSync(output.replace(/\.json$/, ".actual.json"), JSON.stringify(actual));
      throw new Error(diff);
    }
  }
  writeFileSync(
    output,
    JSON.stringify(
      {
        matches: results,
        typescriptMs: results.reduce((s, r) => s + r.typescriptMs, 0),
        rustMs: results.reduce((s, r) => s + r.rustMs, 0),
      },
      null,
      2,
    ),
  );
} finally {
  native.close();
}
