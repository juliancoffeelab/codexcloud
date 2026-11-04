import { isRecord } from "../schema";
import frozen from "./legacy/data/events-v29.json";
interface Change {
  state: boolean;
  stage: number;
  turn: number;
}
interface Recipe {
  active: string[];
  previousEventIds: string[];
  objectives: Record<string, string>;
  required: string[];
  oldEvents: string[];
  waves: string[];
  repeat: string[];
  retireOnRestore: string[];
  timers: { event: string; source: string; tile: string; turns: number }[];
  seals: Record<string, string>;
  named: string[];
  firstSeen: Record<string, number>;
  failureUnits: Record<string, string[]>;
  departedWhenFlag: Record<string, string>;
  defeatOnMissing: { character: string; message: string }[];
  branch: null | { flag: string; recruitment: Record<string, unknown[]> };
  baseRecruitment: Record<string, { unit: string; cost?: number }[]>;
}
const recipes = frozen as Record<string, Recipe>;
/** Pure conversion of remembered legacy events and counters to general event state. */
export function migrateV29ToV30(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 29 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-nine save");
  const setup =
    isRecord(value.setup) && isRecord(value.setup.enemy)
      ? {
          ...value.setup,
          enemy: {
            ...value.setup.enemy,
            diplomacy:
              value.setup.enemy.diplomacy === "restrained"
                ? "diplomatic"
                : value.setup.enemy.diplomacy === "pragmatic"
                  ? "unrestricted"
                  : value.setup.enemy.diplomacy,
          },
        }
      : value.setup;
  const state = value.checkpoint.state;
  if (!isRecord(state.campaign))
    return { ...value, ...(setup === undefined ? {} : { setup }), version: 30 };
  const old = state.campaign;
  const recipe = recipes[String(state.mapId)];
  if (!recipe || !Array.isArray(state.units) || !isRecord(old.flags) || !Array.isArray(old.marks))
    throw new Error("Invalid historical campaign events");
  const stage = Number(old.mission),
    turn = Number(state.round);
  const marks = new Set(old.marks as string[]);
  const flags = structuredClone(old.flags) as Record<string, Change[]>;
  const add = (id: string, atStage = stage, atTurn = turn) => {
    if (!flags[id]?.at(-1)?.state)
      flags[id] = [...(flags[id] ?? []), { state: true, stage: atStage, turn: atTurn }];
  };
  for (const [from, to] of [
    ["green-dain-dead", "dain-dead"],
    ["refuge-war", "peace-lost"],
    ["refuge-peace", "peace-kept"],
  ]) {
    if (flags[from!]) {
      flags[to!] = [...(flags[to!] ?? []), ...flags[from!]!].sort(
        (a, b) => a.stage - b.stage || a.turn - b.turn,
      );
      delete flags[from!];
    }
  }
  const existing = isRecord(old.scripts) ? structuredClone(old.scripts) : null;
  const previous = new Set(recipe.previousEventIds);
  const active = new Set(
    existing && Array.isArray(existing.active)
      ? [
          ...existing.active.filter((id) => recipe.active.includes(String(id))),
          ...recipe.active.filter((id) => !previous.has(id)),
        ]
      : recipe.active,
  );
  const failed = new Set<string>();
  const fail = (id: string) => {
    const owner = Object.keys(recipe.objectives).find((e) => recipe.objectives[e] === id);
    if (owner && active.has(owner)) {
      active.delete(owner);
      failed.add(id);
    }
  };
  for (const id of recipe.oldEvents)
    if (marks.has(`event:${id}`)) {
      active.delete(id);
      add(id);
    }
  for (const id of recipe.waves) if (marks.has(`wave:${id}`)) active.delete(id);
  for (const [tile, id] of Object.entries(recipe.seals))
    if (marks.has(`sealed:${tile}`)) {
      active.delete(id);
      add(id);
    }
  const incoming = Array.isArray(old.fallenCompanions) ? (old.fallenCompanions as string[]) : [];
  const veterans = Array.isArray(old.storyVeterans) ? old.storyVeterans : [];
  const alive = new Set(state.units.filter(isRecord).map((u) => u.character));
  const departed = (who: string) => !!flags[recipe.departedWhenFlag[who] ?? ""]?.at(-1)?.state;
  const diedNow = new Set<string>();
  for (const who of new Set([
    ...recipe.named,
    ...incoming,
    ...[...marks].filter((m) => m.startsWith("fallen:")).map((m) => m.slice(7)),
  ])) {
    if (alive.has(who) || departed(who)) continue;
    const presentAtEntry = veterans.some((v) => isRecord(v) && v.character === who);
    const prior =
      incoming.includes(who) && !presentAtEntry && (recipe.firstSeen[who] ?? stage) < stage;
    if (!prior) diedNow.add(who);
    add(`${who}-dead`, prior ? Math.max(0, stage - 1) : stage, prior ? 0 : turn);
    for (const id of recipe.failureUnits[who] ?? []) fail(id);
  }
  if (isRecord(old.refuge) && old.refuge.escalated === true) {
    add("peace-lost");
    fail("refuge-peace");
    active.delete("peace-lost");
  }
  if (flags["ferryman-dead"]?.at(-1)?.state) fail("ford-shout");
  if (flags.medicine?.at(-1)?.state) add("medicine-explained");
  if (state.winner === "player") {
    if (recipe.required.includes("refuge-peace") && flags["peace-lost"]?.at(-1)?.state) {
      active.delete("refuge-war");
      fail("refuge-peace");
    } else for (const id of recipe.required) active.delete(id);
    active.delete("chapter-victory");
  }
  for (const id of recipe.active)
    if (id.startsWith("goal-") && !active.has(id) && !failed.has(recipe.objectives[id] ?? id))
      add(`${String(state.mapId)}-${id}`);
  const holds =
    existing && Array.isArray(existing.holds)
      ? existing.holds
      : recipe.timers.flatMap((timer) => {
          const source =
            timer.source === "holds" && isRecord(old.holds)
              ? old.holds[timer.tile]
              : old[timer.source];
          return isRecord(source)
            ? [
                {
                  event: timer.event,
                  predicate: 0,
                  turns: Math.min(Number(source.heldTurns), timer.turns),
                  lastRound: Number(source.lastCountedRound),
                },
              ]
            : [];
        });
  const recruitment: Record<string, unknown[]> =
    existing && isRecord(existing.recruitment)
      ? Object.fromEntries(
          Object.entries(existing.recruitment).filter((entry): entry is [string, unknown[]] =>
            Array.isArray(entry[1]),
          ),
        )
      : {};
  for (const [faction, extra] of Object.entries(recruitment)) {
    const pool = [...(recipe.baseRecruitment[faction] ?? [])];
    for (const entry of extra) {
      if (
        isRecord(entry) &&
        typeof entry.unit === "string" &&
        !pool.some((r) => r.unit === entry.unit)
      )
        pool.push(entry as { unit: string; cost?: number });
    }
    recruitment[faction] = pool;
  }
  if (recipe.branch && flags[recipe.branch.flag]?.at(-1)?.state)
    for (const [faction, pool] of Object.entries(recipe.branch.recruitment))
      recruitment[faction] = pool;
  for (const id of recipe.repeat) active.add(id);
  const { refuge: _r, vigil: _v, holds: _h, ...campaign } = old;
  const missing = recipe.defeatOnMissing.find(
    (entry) => !alive.has(entry.character) && !departed(entry.character),
  );
  return {
    ...value,
    ...(setup === undefined ? {} : { setup }),
    version: 30,
    ...(missing ? { pendingEnemy: false } : {}),
    checkpoint: {
      ...value.checkpoint,
      state: {
        ...state,
        ...(missing ? { winner: "enemy" } : {}),
        campaign: {
          ...campaign,
          ...(missing
            ? {
                completed: false,
                outcome:
                  state.winner === "enemy" && campaign.outcome ? campaign.outcome : missing.message,
              }
            : {}),
          flags,
          marks: [],
          fallenCompanions: incoming.filter((id) => !diedNow.has(id)),
          scripts: {
            ...(existing ?? { marks: [], pending: [], nextMark: 1 }),
            active: [...active],
            failed: [...failed],
            holds,
            recruitment,
          },
        },
      },
    },
  };
}
