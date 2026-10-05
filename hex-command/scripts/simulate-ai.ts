import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { execFileSync, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { POLICY_IDS, seededRandom } from "../src/game/ai/policies";
import type { PolicyId } from "../src/game/ai/policies";
import { ARMY_IDS, LANDSCAPES, runArena, runCampaign } from "../src/game/ai/simulation";
import type { ArenaResult, CampaignResult, Landscape, ArmyId } from "../src/game/ai/simulation";

const args = process.argv.slice(2);
const flags = new Set([
  "--mode",
  "--seeds",
  "--seed",
  "--policies",
  "--armies",
  "--landscapes",
  "--campaigns",
  "--rounds",
  "--recruits",
  "--recruitment",
  "--workers",
  "--out",
  "--trace",
  "--replay",
  "--help",
]);
for (let i = 0; i < args.length; i++) {
  const flag = args[i]!;
  if (!flags.has(flag)) throw new Error(`Unknown option: ${flag}`);
  if (!["--trace", "--help"].includes(flag) && (!args[++i] || args[i]!.startsWith("--")))
    throw new Error(`Missing value for ${flag}`);
}
const value = (flag: string, fallback: string) => {
  const index = args.indexOf(flag);
  return index < 0 ? fallback : (args[index + 1] ?? fallback);
};
if (args.includes("--help")) {
  console.log(`Usage: npm run ai:simulate -- [options]
  --mode arena|factorial|campaign default arena
  --seeds N --seed N              count (default 2) and first seed (default 1)
  --policies classic,rush,mixed   policies to compare
  --armies human,woodland,undead,coastal
  --landscapes open,wooded,crossings
  --campaigns lantern-road,green-accord,ashen-oath,borrowed-tide
  --rounds N --recruits N         bounded rounds (default 30), same recruits/phase (default 1)
  --recruitment policy|rush|mixed keep composition fixed to isolate planner differences
  --workers N                    independent seed workers (default 1, maximum 8)
  --out PATH                     JSON report plus readable Markdown report
  --trace                        retain phase commands and full endpoint states
  --replay PATH                  rerun a saved report and require identical outcomes
Arena: every unordered policy pair, both seats and both initiatives; self-play both initiatives.
Factorial: cross each planner with both recruitment styles, then compare every pair.
Campaign: full campaigns from chapter one, earned carryover, no retries. Caps stay unresolved.`);
  process.exit(0);
}
const integer = (flag: string, fallback: number, min: number, max: number) => {
  const number = Number(value(flag, String(fallback)));
  if (!Number.isSafeInteger(number) || number < min || number > max)
    throw new Error(`${flag} must be ${min}..${max}`);
  return number;
};
const select = <T extends string>(flag: string, options: readonly T[]): T[] => {
  const selected = value(flag, options.join(","))
    .split(",")
    .map((entry) => entry.trim());
  if (
    !selected.length ||
    selected.some((entry) => !options.includes(entry as T)) ||
    new Set(selected).size !== selected.length
  )
    throw new Error(`Invalid ${flag}: expected unique entries from ${options.join(",")}`);
  return selected as T[];
};
const hash = createHash("sha256");
for (const file of readdirSync(new URL("../src/game/", import.meta.url), { recursive: true })
  .filter((file): file is string => typeof file === "string" && file.endsWith(".ts"))
  .sort()) {
  hash.update(file + "\0");
  hash.update(readFileSync(new URL(`../src/game/${file}`, import.meta.url)));
}
hash.update(readFileSync(new URL("./simulate-ai.ts", import.meta.url)));
const fingerprint = hash.digest("hex");
type Outcome = ReturnType<typeof arenaSummary> | ReturnType<typeof campaignSummary>;
function arenaSummary(result: ArenaResult) {
  return {
    kind: "arena" as const,
    config: result.config,
    winner: result.winner,
    reason: result.reason,
    rounds: result.rounds,
    objectives: result.objectives,
    casualties: result.phases.flatMap((phase) => phase.losses),
    recruits: result.phases.flatMap((phase) =>
      phase.actions
        .filter((action) => action.kind === "recruit")
        .map((action) => ({ side: phase.side, type: action.type })),
    ),
    promotions: result.phases.flatMap((phase) =>
      phase.actions
        .filter((action) => action.kind === "advance")
        .map((action) => ({ side: phase.side, type: action.type })),
    ),
    stateHash: createHash("sha256").update(JSON.stringify(result.final)).digest("hex"),
  };
}
function campaignSummary(result: CampaignResult) {
  return {
    kind: "campaign" as const,
    config: result.config,
    completed: result.completed,
    missions: result.missions.map((mission) => ({
      id: mission.id,
      winner: mission.winner,
      round: mission.round,
      goals: mission.goals,
      casualties: mission.phases.flatMap((phase) => phase.losses),
      recruits: mission.phases.flatMap((phase) =>
        phase.actions
          .filter((action) => action.kind === "recruit")
          .map((action) => ({ side: phase.side, type: action.type })),
      ),
      promotions: mission.phases.flatMap((phase) =>
        phase.actions
          .filter((action) => action.kind === "advance")
          .map((action) => ({ side: phase.side, type: action.type })),
      ),
      stateHash: createHash("sha256").update(JSON.stringify(mission.final)).digest("hex"),
    })),
  };
}
if (args.includes("--replay")) {
  const path = value("--replay", "");
  const bytes = readFileSync(path);
  const report = JSON.parse(
    (path.endsWith(".gz") ? gunzipSync(bytes) : bytes).toString("utf8"),
  ) as {
    fingerprint: string;
    outcomes: Outcome[];
  };
  if (report.fingerprint !== fingerprint)
    throw new Error(
      "Rules/policies/runner changed; replay on the recorded source before comparing outcomes.",
    );
  for (const outcome of report.outcomes) {
    const actual =
      outcome.kind === "arena"
        ? arenaSummary(runArena(outcome.config))
        : campaignSummary(runCampaign(outcome.config));
    if (JSON.stringify(actual) !== JSON.stringify(outcome))
      throw new Error(`Replay mismatch: ${JSON.stringify(outcome.config)}`);
  }
  console.log(JSON.stringify({ verified: true, games: report.outcomes.length, fingerprint }));
  process.exit(0);
}
const mode = value("--mode", "arena");
if (!["arena", "factorial", "campaign"].includes(mode)) throw new Error("Unknown simulation mode");
const composition = value("--recruitment", "policy");
if (!["policy", "rush", "mixed"].includes(composition))
  throw new Error("Unknown recruitment style");
const recruitment = composition === "policy" ? undefined : (composition as "rush" | "mixed");
if (mode === "factorial" && recruitment)
  throw new Error("Factorial mode assigns both recruitment styles; omit --recruitment");
const policies = select("--policies", POLICY_IDS);
const output = resolve(value("--out", "artifacts/ai-simulation.json"));
if (!output.endsWith(".json")) throw new Error("--out must end in .json");
mkdirSync(dirname(output), { recursive: true });
const seeds = integer("--seeds", 2, 1, 10000),
  firstSeed = integer("--seed", 1, 0, 2147483647);
const maxRounds = integer("--rounds", 30, 1, 1000),
  recruitLimit = integer("--recruits", 1, 0, 20);
const workers = Math.min(integer("--workers", 1, 1, 8), seeds);
const outcomes: Outcome[] = [];
const traces: (ArenaResult | CampaignResult)[] = [];
const started = performance.now();
const partial = output.replace(/\.json$/, ".partial.json");
const append = (result: ArenaResult | CampaignResult) => {
  outcomes.push("winner" in result ? arenaSummary(result) : campaignSummary(result));
  if (args.includes("--trace")) traces.push(result);
  if (outcomes.length % 10 === 0) {
    writeFileSync(
      partial,
      JSON.stringify({ schema: 1, status: "running", mode, fingerprint, outcomes }),
    );
    console.error(
      `${outcomes.length} games; ${((performance.now() - started) / 1000).toFixed(1)}s`,
    );
  }
};
if (workers > 1) {
  const forwarded: string[] = [];
  for (let i = 0; i < args.length; i++) {
    if (["--workers", "--seed", "--seeds", "--out"].includes(args[i]!)) i++;
    else forwarded.push(args[i]!);
  }
  const jobs = Array.from({ length: workers }, (_, index) => {
    const offset = Math.floor((index * seeds) / workers);
    const count = Math.floor(((index + 1) * seeds) / workers) - offset;
    const path = output.replace(/\.json$/, `.worker-${index}.json`);
    return { offset, count, path };
  });
  await Promise.all(
    jobs.map(
      (job) =>
        new Promise<void>((done, fail) => {
          const child = spawn(
            process.execPath,
            [
              "--import",
              "tsx",
              fileURLToPath(import.meta.url),
              ...forwarded,
              "--workers",
              "1",
              "--seed",
              String(firstSeed + job.offset),
              "--seeds",
              String(job.count),
              "--out",
              job.path,
            ],
            { cwd: new URL("../", import.meta.url), stdio: ["ignore", "ignore", "inherit"] },
          );
          child.once("error", fail);
          child.once("exit", (code) =>
            code === 0 ? done() : fail(new Error(`Seed worker failed (${code}): ${job.path}`)),
          );
        }),
    ),
  );
  for (const job of jobs) {
    const shard = JSON.parse(readFileSync(job.path, "utf8")) as {
      status: string;
      fingerprint: string;
      seeds: { first: number; count: number };
      outcomes: Outcome[];
      traces?: (ArenaResult | CampaignResult)[];
    };
    if (
      shard.status !== "complete" ||
      shard.fingerprint !== fingerprint ||
      shard.seeds.first !== firstSeed + job.offset ||
      shard.seeds.count !== job.count
    )
      throw new Error(`Invalid seed worker report: ${job.path}`);
    outcomes.push(...shard.outcomes);
    traces.push(...(shard.traces ?? []));
  }
} else if (mode !== "campaign") {
  const armies = select("--armies", ARMY_IDS),
    landscapes = select("--landscapes", LANDSCAPES);
  const bots = policies.flatMap((policy) =>
    mode === "factorial"
      ? (["rush", "mixed"] as const).map((style) => ({ policy, style }))
      : [{ policy, style: recruitment }],
  );
  for (let seed = firstSeed; seed < firstSeed + seeds; seed++)
    for (const army of armies)
      for (const landscape of landscapes)
        for (let a = 0; a < bots.length; a++)
          for (let b = a; b < bots.length; b++) {
            const left = bots[a]!,
              right = bots[b]!;
            const seats: [typeof left, typeof right][] =
              a === b
                ? [[left, right]]
                : [
                    [left, right],
                    [right, left],
                  ];
            for (const [player, enemy] of seats)
              for (const first of ["player", "enemy"] as const)
                append(
                  runArena({
                    seed,
                    army,
                    landscape,
                    player: player.policy,
                    enemy: enemy.policy,
                    first,
                    maxRounds,
                    recruitLimit,
                    ...(mode === "factorial"
                      ? {
                          recruitmentBySide: { player: player.style!, enemy: enemy.style! },
                        }
                      : recruitment
                        ? { recruitment }
                        : {}),
                  }),
                );
          }
} else {
  const campaigns = select("--campaigns", [
    "lantern-road",
    "green-accord",
    "ashen-oath",
    "borrowed-tide",
  ] as const);
  for (let seed = firstSeed; seed < firstSeed + seeds; seed++)
    for (const campaign of campaigns)
      for (const player of policies)
        for (const enemy of policies)
          append(
            runCampaign({
              campaign,
              player,
              enemy,
              seed,
              maxRounds,
              recruitLimit,
              ...(recruitment ? { recruitment } : {}),
            }),
          );
}

function botLabel(config: ReturnType<typeof arenaSummary>["config"], side: "player" | "enemy") {
  const style = config.recruitmentBySide?.[side] ?? config.recruitment;
  return style ? `${config[side]}/${style}` : config[side];
}
const labels = [
  ...new Set(
    outcomes.flatMap((outcome) =>
      outcome.kind === "arena"
        ? [botLabel(outcome.config, "player"), botLabel(outcome.config, "enemy")]
        : [],
    ),
  ),
].sort();
const identities = new Set(
  outcomes.map((outcome) => JSON.stringify([outcome.kind, outcome.config])),
);
if (identities.size !== outcomes.length) throw new Error("Duplicate simulation configurations");
const arenaBots = mode === "factorial" ? policies.length * 2 : policies.length;
const expected =
  mode === "campaign"
    ? seeds *
      select("--campaigns", [
        "lantern-road",
        "green-accord",
        "ashen-oath",
        "borrowed-tide",
      ] as const).length *
      policies.length ** 2
    : seeds *
      select("--armies", ARMY_IDS).length *
      select("--landscapes", LANDSCAPES).length *
      2 *
      arenaBots ** 2;
if (outcomes.length !== expected)
  throw new Error(`Incomplete coverage: ${outcomes.length}/${expected}`);

function arenaRows(results: Outcome[]) {
  const rows = [];
  for (const policy of labels)
    for (const opponent of labels) {
      const games = results.filter(
        (entry): entry is ReturnType<typeof arenaSummary> =>
          entry.kind === "arena" &&
          ((botLabel(entry.config, "player") === policy &&
            botLabel(entry.config, "enemy") === opponent) ||
            (botLabel(entry.config, "enemy") === policy &&
              botLabel(entry.config, "player") === opponent)),
      );
      if (!games.length) continue;
      // Self-play uses the player seat as reference, avoiding double-counted symmetric wins.
      const win = (game: (typeof games)[number]) =>
        game.winner !== null &&
        (policy === opponent
          ? game.winner === "player"
          : botLabel(game.config, game.winner) === policy);
      const wins = games.filter(win).length,
        unresolved = games.filter((game) => game.winner === null).length;
      const losses = games.length - wins - unresolved;
      // Bootstrap entire seed blocks; seat/initiative repeats aren't independent samples.
      const blocks = [...new Set(games.map((game) => game.config.seed))].map((seed) => {
        const block = games.filter((game) => game.config.seed === seed);
        return {
          total: block.length,
          margin: block.reduce(
            (sum, game) => sum + (game.winner === null ? 0 : win(game) ? 1 : -1),
            0,
          ),
        };
      });
      let margin95: [number, number] | null = null;
      if (blocks.length >= 2) {
        const random = seededRandom(firstSeed),
          samples = [];
        for (let i = 0; i < 1000; i++) {
          let count = 0,
            margin = 0;
          for (let j = 0; j < blocks.length; j++) {
            const block = blocks[Math.floor(random() * blocks.length)]!;
            count += block.total;
            margin += block.margin;
          }
          samples.push(margin / count);
        }
        samples.sort((a, b) => a - b);
        margin95 = [samples[24]!, samples[974]!];
      }
      rows.push({
        policy,
        opponent,
        games: games.length,
        wins,
        losses,
        unresolved,
        winRate: wins / games.length,
        decisiveWinRate: wins + losses ? wins / (wins + losses) : null,
        completionRate: (wins + losses) / games.length,
        netCompletedWins: (wins - losses) / games.length,
        seedBlocks: blocks.length,
        netCompletedWins95: margin95,
        meanRounds: games.reduce((sum, game) => sum + game.rounds, 0) / games.length,
      });
    }
  return rows;
}
const rows = mode !== "campaign" ? arenaRows(outcomes) : [];
const strata =
  mode !== "campaign"
    ? ARMY_IDS.flatMap((army: ArmyId) =>
        LANDSCAPES.map((landscape: Landscape) => ({
          army,
          landscape,
          rows: arenaRows(
            outcomes.filter(
              (outcome) =>
                outcome.kind === "arena" &&
                outcome.config.army === army &&
                outcome.config.landscape === landscape,
            ),
          ),
        })).filter((stratum) => stratum.rows.length),
      )
    : [];
const armyCompositions = ARMY_IDS.flatMap((army) =>
  labels.map((bot) => {
    const totals: Record<string, { recruited: number; lost: number }> = {};
    let games = 0;
    for (const outcome of outcomes) {
      if (outcome.kind !== "arena" || outcome.config.army !== army) continue;
      for (const side of ["player", "enemy"] as const) {
        if (botLabel(outcome.config, side) !== bot) continue;
        games++;
        for (const unit of outcome.recruits.filter((unit) => unit.side === side))
          (totals[unit.type] ??= { recruited: 0, lost: 0 }).recruited++;
        for (const unit of outcome.casualties.filter((unit) => unit.side === side))
          (totals[unit.type] ??= { recruited: 0, lost: 0 }).lost++;
      }
    }
    return { army, bot, games, units: totals };
  }),
).filter((entry) => entry.games);
let sourceCommit = "unknown";
try {
  sourceCommit = execFileSync("git", ["rev-parse", "HEAD"], {
    encoding: "utf8",
    cwd: new URL("../", import.meta.url),
  }).trim();
} catch {
  /* Fingerprint still captures all rule/policy source bytes. */
}
const report = {
  schema: 1,
  status: "complete",
  mode,
  fingerprint,
  sourceCommit,
  elapsedSeconds: (performance.now() - started) / 1000,
  seeds: { first: firstSeed, count: seeds },
  rows,
  strata,
  armyCompositions,
  seatDiagnostics:
    mode !== "campaign"
      ? labels.map((policy) => {
          const games = outcomes.filter(
            (entry): entry is ReturnType<typeof arenaSummary> =>
              entry.kind === "arena" &&
              botLabel(entry.config, "player") === policy &&
              botLabel(entry.config, "enemy") === policy,
          );
          return {
            policy,
            games: games.length,
            playerWins: games.filter((game) => game.winner === "player").length,
            enemyWins: games.filter((game) => game.winner === "enemy").length,
            firstWins: games.filter((game) => game.winner === game.config.first).length,
            secondWins: games.filter((game) => game.winner && game.winner !== game.config.first)
              .length,
            unresolved: games.filter((game) => game.winner === null).length,
          };
        })
      : [],
  outcomes,
  ...(args.includes("--trace") ? { traces } : {}),
};
writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
writeFileSync(
  partial,
  JSON.stringify({ schema: 1, status: "complete", mode, fingerprint, outcomes }),
);
const table = [
  "| Policy | Opponent | Games | Wins | Losses | Unresolved | Mean rounds |",
  "|---|---|---:|---:|---:|---:|---:|",
  ...rows.map(
    (row) =>
      `| ${row.policy} | ${row.opponent} | ${row.games} | ${row.wins} | ${row.losses} | ${row.unresolved} | ${row.meanRounds.toFixed(1)} |`,
  ),
];
const campaignTable = [
  "| Campaign | Player policy | Enemy policy | Seed | Completed | Recorded chapters |",
  "|---|---|---|---:|---|---|",
  ...outcomes
    .filter((outcome): outcome is ReturnType<typeof campaignSummary> => outcome.kind === "campaign")
    .map(
      (outcome) =>
        `| ${outcome.config.campaign} | ${outcome.config.player} | ${outcome.config.enemy} | ${outcome.config.seed} | ${outcome.completed} | ${outcome.missions.map((mission) => `${mission.id}: ${mission.winner ?? "unresolved"}@${mission.round}`).join("; ")} |`,
    ),
];
const markdown = `# Seeded AI ${mode} results\n\nSource: \`${sourceCommit}\`. Rules/policy fingerprint: \`${fingerprint}\`.\n\n${outcomes.length} games, seeds ${firstSeed}–${firstSeed + seeds - 1}, ${report.elapsedSeconds.toFixed(1)} seconds.\n\n${(mode !== "campaign" ? table : campaignTable).join("\n")}\n\nCaps are unresolved, not draws or defeats. Arena seats and initiative are counterbalanced on identical paired terrain/rosters/resources. Confidence intervals in JSON resample whole seed blocks; they describe this finite scenario distribution and these policies, not optimal play or human balance. Campaigns retain authored asymmetry, earn their carryover, and receive no retries. Combat itself remains deterministic. Full per-roster/per-landscape strata and actual casualties/recruitment are in JSON.\n`;
writeFileSync(output.replace(/\.json$/, "") + ".md", markdown);
console.log(
  JSON.stringify({ output, games: outcomes.length, elapsedSeconds: report.elapsedSeconds, rows }),
);
