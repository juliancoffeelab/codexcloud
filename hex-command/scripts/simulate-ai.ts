import { RustRunner } from "./rust-runner";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { execFileSync, spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { availableParallelism } from "node:os";
import { ResultJournal, readDetail } from "./ai-records";
import type { DetailRecord } from "./ai-records";
import { POLICY_IDS, seededRandom } from "../src/game/ai/policies";
import type { PolicyId } from "../src/game/ai/policies";
import { ARMY_IDS, LANDSCAPES, runArena, runCampaign } from "../src/game/ai/simulation";
import type { ArenaResult, CampaignResult } from "../src/game/ai/simulation";
import { arenaMatchups, campaignMatchups } from "../src/game/ai/matchups";
import { DIPLOMACY_IDS } from "../src/game/ai/diplomacy";

const args = process.argv.slice(2);
const flags = new Set([
  "--engine",
  "--mode",
  "--seeds",
  "--seed",
  "--policies",
  "--armies",
  "--landscapes",
  "--campaigns",
  "--rounds",
  "--recruits",
  "--diplomacy",
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
  --engine typescript|rust        default rust; native Rust requires ai:rust:build
  --mode arena|campaign|full      default arena; full runs both schedules
  --seeds N --seed N              count (default 2) and first seed (default 1)
  --policies classic,rush,mixed   policies to compare
  --armies human,woodland,undead,coastal
  --landscapes open,wooded,crossings,islands
  --campaigns lantern-road,green-accord,ashen-oath,borrowed-tide
  --rounds N --recruits N         bounded rounds (default 30), same recruits/phase (default 1)
  --diplomacy pragmatic,restrained campaign behavior settings (default both)
  --workers N                    bounded worker pool (default available CPU quota, maximum 8)
  --out PATH                     JSON report plus readable Markdown report
  --trace                        additionally retain raw runner results; detailed phases always saved
  --replay PATH                  rerun a saved report and require identical outcomes
Arena: every race/policy entrant pair, both seats and both initiatives; self-play both initiatives.
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
const backend = value("--engine", "rust");
if (!["typescript", "rust"].includes(backend)) throw new Error("Unknown simulation engine");
const native = backend === "rust" ? new RustRunner() : undefined;
if (native) process.once("exit", () => native.close());
const hash = createHash("sha256");
for (const file of readdirSync(new URL("../src/game/", import.meta.url), { recursive: true })
  .filter((file): file is string => typeof file === "string" && file.endsWith(".ts"))
  .sort()) {
  hash.update(file + "\0");
  hash.update(readFileSync(new URL(`../src/game/${file}`, import.meta.url)));
}
hash.update(readFileSync(new URL("./simulate-ai.ts", import.meta.url)));
hash.update(readFileSync(new URL("./ai-records.ts", import.meta.url)));
if (backend === "rust") {
  for (const file of readdirSync(new URL("../rust/src/", import.meta.url)).sort()) {
    hash.update(file + "\0");
    hash.update(readFileSync(new URL(`../rust/src/${file}`, import.meta.url)));
  }
  for (const file of ["Cargo.toml", "Cargo.lock", "data/rules.json"])
    hash.update(readFileSync(new URL(`../rust/${file}`, import.meta.url)));
}
const fingerprint = hash.digest("hex");
type DetailedOutcome = ReturnType<typeof arenaSummary> | ReturnType<typeof campaignSummary>;
type Outcome = ReturnType<typeof compactOutcome>;
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
    phases: result.phases,
    initial: result.initial,
    final: result.final,
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
      phases: mission.phases,
      initial: mission.initial,
      final: mission.final,
      stateHash: createHash("sha256").update(JSON.stringify(mission.final)).digest("hex"),
    })),
  };
}
function compactOutcome(outcome: DetailedOutcome, detail: DetailRecord, elapsedMs: number) {
  if (outcome.kind === "arena") {
    const { phases: _phases, initial: _initial, final: _final, ...summary } = outcome;
    return { ...summary, detail, elapsedMs };
  }
  return {
    ...outcome,
    missions: outcome.missions.map(({ phases: _phases, initial: _initial, final, ...mission }) => ({
      ...mission,
      escalated: final.campaign?.refuge?.escalated ?? false,
    })),
    detail,
    elapsedMs,
  };
}
if (args.includes("--replay")) {
  const path = value("--replay", "");
  const bytes = readFileSync(path);
  const report = JSON.parse(
    (path.endsWith(".gz") ? gunzipSync(bytes) : bytes).toString("utf8"),
  ) as {
    fingerprint: string;
    schema?: number;
    outcomes: (Outcome | DetailedOutcome)[];
  };
  if (report.fingerprint !== fingerprint)
    throw new Error(
      "Rules/policies/runner changed; replay on the recorded source before comparing outcomes.",
    );
  for (const outcome of report.outcomes) {
    const expected =
      "detail" in outcome
        ? readDetail<DetailedOutcome>(dirname(resolve(path)), outcome.detail)
        : outcome;
    const actual =
      expected.kind === "arena"
        ? arenaSummary(native ? await native.run(expected.config) : runArena(expected.config))
        : campaignSummary(
            native ? await native.run(expected.config) : runCampaign(expected.config),
          );
    if (JSON.stringify(actual) !== JSON.stringify(expected))
      throw new Error(`Replay mismatch: ${JSON.stringify(outcome.config)}`);
  }
  console.log(JSON.stringify({ verified: true, games: report.outcomes.length, fingerprint }));
  process.exit(0);
}
const mode = value("--mode", "arena");
if (!["arena", "campaign", "full"].includes(mode))
  throw new Error(
    "Unknown simulation mode; use arena, campaign or full. Recruitment belongs to each policy.",
  );
const diplomacy = select("--diplomacy", DIPLOMACY_IDS);
const armies = select("--armies", ARMY_IDS);
const landscapes = select("--landscapes", LANDSCAPES);
const campaigns = select("--campaigns", [
  "lantern-road",
  "green-accord",
  "ashen-oath",
  "borrowed-tide",
] as const);
const policies = select("--policies", POLICY_IDS);
const output = resolve(value("--out", "artifacts/ai-simulation.json"));
if (!output.endsWith(".json")) throw new Error("--out must end in .json");
mkdirSync(dirname(output), { recursive: true });
const seeds = integer("--seeds", 2, 1, 10000),
  firstSeed = integer("--seed", 1, 0, 2147483647);
const maxRounds = integer("--rounds", 30, 1, 1000),
  recruitLimit = integer("--recruits", 1, 0, 20);
function defaultWorkers() {
  let count = availableParallelism();
  try {
    const [quota, period] = readFileSync("/sys/fs/cgroup/cpu.max", "utf8").trim().split(/\s+/);
    if (quota !== "max")
      count = Math.min(count, Math.max(1, Math.floor(Number(quota) / Number(period))));
  } catch {
    /* Non-Linux hosts use availableParallelism. */
  }
  return Math.min(count, 8);
}
const workers = integer("--workers", defaultWorkers(), 1, 8);
const outcomes: Outcome[] = [];
const traceFiles: string[] = [];
const started = performance.now();
const partial = output.replace(/\.json$/, ".partial.json");
const journal =
  workers === 1 ? new ResultJournal(output.replace(/\.json$/, ".details.jsonl")) : undefined;
const traceJournal =
  workers === 1 && args.includes("--trace")
    ? new ResultJournal(output.replace(/\.json$/, ".trace.jsonl"))
    : undefined;
if (traceJournal) traceFiles.push(traceJournal.file);
const checkpoint = () =>
  writeFileSync(
    partial,
    JSON.stringify({
      schema: 3,
      status: "running",
      mode,
      fingerprint,
      completed: outcomes.length,
      elapsedSeconds: (performance.now() - started) / 1000,
      outcomes,
    }),
  );
const append = (result: ArenaResult | CampaignResult, elapsedMs: number) => {
  const detail = "winner" in result ? arenaSummary(result) : campaignSummary(result);
  outcomes.push(compactOutcome(detail, journal!.append(detail), elapsedMs));
  traceJournal?.append(result);
  if (outcomes.length % 10 === 0) {
    checkpoint();
    console.error(
      `${outcomes.length} games; ${((performance.now() - started) / 1000).toFixed(1)}s`,
    );
  }
};
if (workers > 1) {
  const forwarded: string[] = [];
  for (let i = 0; i < args.length; i++) {
    if (["--workers", "--seed", "--seeds", "--out", "--mode", "--landscapes"].includes(args[i]!))
      i++;
    else forwarded.push(args[i]!);
  }
  const jobs: {
    mode: "arena" | "campaign";
    seed: number;
    landscape?: (typeof LANDSCAPES)[number];
    path: string;
  }[] = [];
  for (let seed = firstSeed; seed < firstSeed + seeds; seed++) {
    if (mode !== "campaign")
      for (const landscape of landscapes)
        jobs.push({
          mode: "arena",
          seed,
          landscape,
          path: output.replace(/\.json$/, `.arena-${landscape}-seed-${seed}.json`),
        });
    if (mode !== "arena")
      jobs.push({
        mode: "campaign",
        seed,
        path: output.replace(/\.json$/, `.campaign-seed-${seed}.json`),
      });
  }
  const active = new Set<ReturnType<typeof spawn>>();
  let next = 0;
  const runJob = (job: (typeof jobs)[number]) =>
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
          String(job.seed),
          "--seeds",
          "1",
          "--mode",
          job.mode,
          ...(job.landscape ? ["--landscapes", job.landscape] : []),
          "--out",
          job.path,
        ],
        { cwd: new URL("../", import.meta.url), stdio: ["ignore", "ignore", "inherit"] },
      );
      active.add(child);
      child.once("error", fail);
      child.once("exit", (code) => {
        active.delete(child);
        if (code === 0) done();
        else fail(new Error(`Simulation worker failed (${code}): ${job.path}`));
      });
    });
  try {
    await Promise.all(
      Array.from({ length: Math.min(workers, jobs.length) }, async () => {
        while (next < jobs.length) await runJob(jobs[next++]!);
      }),
    );
  } catch (error) {
    for (const child of active) child.kill("SIGTERM");
    throw error;
  }
  for (const job of jobs) {
    const shard = JSON.parse(readFileSync(job.path, "utf8")) as {
      status: string;
      fingerprint: string;
      seeds: { first: number; count: number };
      outcomes: Outcome[];
      traceFiles?: string[];
    };
    if (
      shard.status !== "complete" ||
      shard.fingerprint !== fingerprint ||
      shard.seeds.first !== job.seed ||
      shard.seeds.count !== 1
    )
      throw new Error(`Invalid seed worker report: ${job.path}`);
    outcomes.push(...shard.outcomes);
    traceFiles.push(...(shard.traceFiles ?? []));
  }
} else {
  for (let seed = firstSeed; seed < firstSeed + seeds; seed++) {
    const limits = { seed, maxRounds, recruitLimit, policies };
    if (mode !== "campaign")
      for (const config of arenaMatchups({ ...limits, armies, landscapes })) {
        const begin = performance.now();
        const result = native ? await native.run(config) : runArena(config);
        append(result, performance.now() - begin);
      }
    if (mode !== "arena")
      for (const config of campaignMatchups({ ...limits, campaigns, diplomacy })) {
        const begin = performance.now();
        const result = native ? await native.run(config) : runCampaign(config);
        append(result, performance.now() - begin);
      }
  }
}
journal?.close();
traceJournal?.close();
native?.close();

function botLabel(config: ReturnType<typeof arenaSummary>["config"], side: "player" | "enemy") {
  return `${config.armies[side]}/${config[side]}`;
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
const expected =
  seeds *
  ((mode !== "arena" ? campaigns.length * policies.length ** 2 * diplomacy.length : 0) +
    (mode !== "campaign" ? landscapes.length * 2 * (armies.length * policies.length) ** 2 : 0));
if (outcomes.length !== expected)
  throw new Error(`Incomplete coverage: ${outcomes.length}/${expected}`);

function arenaRows(results: Outcome[]) {
  const rows = [];
  for (const policy of labels)
    for (const opponent of labels) {
      const games = results.filter(
        (entry): entry is Extract<Outcome, { kind: "arena" }> =>
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
    ? armies.flatMap((playerArmy) =>
        armies.flatMap((enemyArmy) =>
          landscapes
            .map((landscape) => ({
              playerArmy,
              enemyArmy,
              landscape,
              rows: arenaRows(
                outcomes.filter(
                  (outcome) =>
                    outcome.kind === "arena" &&
                    outcome.config.armies.player === playerArmy &&
                    outcome.config.armies.enemy === enemyArmy &&
                    outcome.config.landscape === landscape,
                ),
              ),
            }))
            .filter((stratum) => stratum.rows.length),
        ),
      )
    : [];
const campaignStrata =
  mode !== "arena"
    ? campaigns.flatMap((campaign) =>
        policies.flatMap((player) =>
          policies.flatMap((enemy) =>
            diplomacy.map((alignment) => {
              const games = outcomes.filter(
                (outcome): outcome is Extract<Outcome, { kind: "campaign" }> =>
                  outcome.kind === "campaign" &&
                  outcome.config.campaign === campaign &&
                  outcome.config.player === player &&
                  outcome.config.enemy === enemy &&
                  outcome.config.diplomacy === alignment,
              );
              return {
                campaign,
                player,
                enemy,
                diplomacy: alignment,
                games: games.length,
                completed: games.filter((game) => game.completed).length,
                defeated: games.filter((game) => game.missions.at(-1)?.winner === "enemy").length,
                unresolved: games.filter((game) => game.missions.at(-1)?.winner === null).length,
                escalated: games.filter((game) =>
                  game.missions.some((mission) => mission.escalated),
                ).length,
                missions: [
                  ...new Set(games.flatMap((game) => game.missions.map((mission) => mission.id))),
                ].map((id) => {
                  const chapters = games.flatMap((game) =>
                    game.missions.filter((mission) => mission.id === id),
                  );
                  return {
                    id,
                    attempted: chapters.length,
                    won: chapters.filter((chapter) => chapter.winner === "player").length,
                    defeated: chapters.filter((chapter) => chapter.winner === "enemy").length,
                    unresolved: chapters.filter((chapter) => chapter.winner === null).length,
                  };
                }),
              };
            }),
          ),
        ),
      )
    : [];
const armyCompositions = labels.map((bot) => {
  const totals: Record<string, { recruited: number; lost: number; promoted: number }> = {};
  let games = 0;
  for (const outcome of outcomes) {
    if (outcome.kind !== "arena") continue;
    for (const side of ["player", "enemy"] as const) {
      if (botLabel(outcome.config, side) !== bot) continue;
      games++;
      for (const unit of outcome.recruits.filter((unit) => unit.side === side))
        (totals[unit.type] ??= { recruited: 0, lost: 0, promoted: 0 }).recruited++;
      for (const unit of outcome.casualties.filter((unit) => unit.side === side))
        (totals[unit.type] ??= { recruited: 0, lost: 0, promoted: 0 }).lost++;
      for (const unit of outcome.promotions.filter((unit) => unit.side === side))
        (totals[unit.type] ??= { recruited: 0, lost: 0, promoted: 0 }).promoted++;
    }
  }
  return { bot, games, units: totals };
});
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
  schema: 3,
  status: "complete",
  mode,
  fingerprint,
  sourceCommit,
  engine: backend,
  elapsedSeconds: (performance.now() - started) / 1000,
  seeds: { first: firstSeed, count: seeds },
  rows,
  strata,
  campaignStrata,
  armyCompositions,
  seatDiagnostics:
    mode !== "campaign"
      ? labels.map((policy) => {
          const games = outcomes.filter(
            (entry): entry is Extract<Outcome, { kind: "arena" }> =>
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
  detailFormat: "jsonl-byte-ranges-v1",
  ...(args.includes("--trace") ? { traceFiles } : {}),
};
writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
writeFileSync(
  partial,
  JSON.stringify({
    schema: 3,
    status: "complete",
    mode,
    fingerprint,
    completed: outcomes.length,
    outcomes,
  }),
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
  "| Campaign | Player policy | Enemy policy | Diplomacy | Seed | Completed | Recorded chapters |",
  "|---|---|---|---|---:|---|---|",
  ...outcomes
    .filter(
      (outcome): outcome is Extract<Outcome, { kind: "campaign" }> => outcome.kind === "campaign",
    )
    .map(
      (outcome) =>
        `| ${outcome.config.campaign} | ${outcome.config.player} | ${outcome.config.enemy} | ${outcome.config.diplomacy} | ${outcome.config.seed} | ${outcome.completed} | ${outcome.missions.map((mission) => `${mission.id}: ${mission.winner ?? "unresolved"}@${mission.round}`).join("; ")} |`,
    ),
];
const tables =
  mode === "full" ? [...table, "", ...campaignTable] : mode === "arena" ? table : campaignTable;
const markdown = `# Seeded AI ${mode} results\n\nSource: \`${sourceCommit}\`. Rules/policy fingerprint: \`${fingerprint}\`.\n\n${outcomes.length} games, seeds ${firstSeed}–${firstSeed + seeds - 1}, ${report.elapsedSeconds.toFixed(1)} seconds.\n\n${tables.join("\n")}\n\nCaps are unresolved, not draws or defeats. Arena seats and initiative are counterbalanced on identical paired terrain/resources, with race-specific rosters. Confidence intervals in JSON resample whole seed blocks; they describe this finite scenario distribution and these policies, not optimal play or human balance. Campaigns retain authored asymmetry, earn their carryover, and receive no retries. Combat itself remains deterministic. Statistics and compact outcomes are in this report's JSON; complete phase commands, combats, troop states, ownership, goals and endpoint states are in its adjacent result journals, indexed by each outcome's detail reference.\n`;
writeFileSync(output.replace(/\.json$/, "") + ".md", markdown);
console.log(
  JSON.stringify({ output, games: outcomes.length, elapsedSeconds: report.elapsedSeconds, rows }),
);
