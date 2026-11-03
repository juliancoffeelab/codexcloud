import { isRecord } from "../schema";
import frozen from "./legacy/data/events-v27.json";

interface HistoricalMember {
  oldId: string;
  unitType: string;
  character?: string;
  role?: string;
  items: string[];
}
interface Recipe {
  eventIds: string[];
  oldEventIds: string[];
  aliases: Record<string, string>;
  choiceFlags: Record<string, string>;
  spawns: {
    event: string;
    area: string[];
    faction: string;
    aiGroup?: string | null;
    units: HistoricalMember[];
  }[];
  waves: string[];
  holds: { event: string; tile: string; predicate: number }[];
  ritual: null | {
    id: string;
    sources: string[];
    radius: number;
    light: unknown;
    units: HistoricalMember[];
    aiGroup: string;
  };
  recruitmentGrants: Record<string, Record<string, { unit: string; cost?: number }[]>>;
  removedTransfers: string[];
}
const history = frozen as Record<string, Recipe>;

/** Convert historical progress records into active events, explicit flags and mark instances. */
export function migrateV27ToV28(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 27 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-seven save");
  const old = value.checkpoint.state;
  if (!isRecord(old.campaign) || !((old.mapId as string) in history))
    return { ...value, version: 28 };
  const recipe = history[String(old.mapId)]!;
  const campaign = old.campaign;
  if (
    !Array.isArray(campaign.marks) ||
    !Array.isArray(old.units) ||
    !Array.isArray(old.tiles) ||
    !Array.isArray(old.factions)
  )
    throw new Error("Invalid historical event state");
  const marks = new Set(campaign.marks as string[]);
  const flags = new Set((campaign.choices ?? []) as string[]);
  const active = new Set(recipe.eventIds);
  const aliases: Record<string, string> = recipe.aliases;
  const choices: Record<string, string> = recipe.choiceFlags;
  for (const id of recipe.oldEventIds)
    if (marks.has(`event:${id}`)) {
      active.delete(aliases[id] ?? id);
      flags.add(id);
      if (choices[id]) flags.add(choices[id]!);
    }
  for (const id of recipe.waves) if (marks.has(`wave:${id}`)) active.delete(id);
  const recruitment: Record<string, { unit: string; cost?: number }[]> = {};
  for (const [event, pools] of Object.entries(recipe.recruitmentGrants))
    if (marks.has(`event:${event}`)) {
      for (const [faction, units] of Object.entries(
        pools as Record<string, { unit: string; cost?: number }[]>,
      )) {
        const pool = (recruitment[faction] ??= []);
        for (const unit of units) if (!pool.some((u) => u.unit === unit.unit)) pool.push(unit);
      }
    }
  const pending: { area: string[]; faction: string; aiGroup?: string | null; units: unknown[] }[] =
    [];
  const member = (source: {
    oldId: string;
    unitType: string;
    character?: string;
    role?: string;
    items: string[];
  }) => {
    const { oldId: _old, ...unit } = source;
    return unit;
  };
  for (const group of recipe.spawns)
    if (marks.has(`event:${group.event}`)) {
      const units = group.units
        .filter((u) => !marks.has(`spawn:${group.event}:${u.oldId}`))
        .map(member);
      if (units.length)
        pending.push({ area: group.area, faction: group.faction, aiGroup: group.aiGroup, units });
    }
  const units: Record<string, unknown>[] = old.units.map((input) => {
    if (!isRecord(input)) throw new Error("Invalid historical unit");
    return { ...input, ...(input.type === "captives" ? { role: "captive" } : {}) };
  });
  if (old.mapId === "lantern-ferry") {
    if (marks.has("event:ford-shout")) {
      flags.add("ferryman-saved");
      flags.add("ferryman-alive");
    } else if (Number(old.round) >= 16) {
      flags.add("ferryman-dead");
      active.delete("ferryman-death");
    }
    const bears = units.filter((u) => u.type === "bear" && u.faction === "wildlife");
    if (marks.has("spawn:quarry-bear:member-1") && bears.length === 1)
      Object.assign(bears[0]!, { character: "quarry-bear" });
  }
  for (const unit of units)
    if (unit.side === "player" && typeof unit.character === "string")
      active.delete(`rescue-${unit.character}`);
  const holds = recipe.holds.flatMap((h) => {
    const counters = isRecord(campaign.holds) ? campaign.holds : {};
    const counter = counters[h.tile];
    return isRecord(counter)
      ? [
          {
            event: h.event,
            predicate: h.predicate,
            turns: counter.heldTurns,
            lastRound: counter.lastCountedRound,
          },
        ]
      : [];
  });
  const mapMarks: { id: number; type: string; tile: string; light: unknown }[] = [];
  const ritual = recipe.ritual;
  if (ritual) {
    let sequence = 1;
    const source = (marker: string) => {
      const body = marker.split(":").slice(3).join(":");
      if (body.startsWith("mark-")) return body.slice(5);
      const dead = /^death-u[1-9]\d*-(.+)$/.exec(body);
      if (!dead) throw new Error("Invalid historical ritual group");
      return dead[1]!;
    };
    const area = (id: string) => {
      const tiles = old.tiles as Record<string, unknown>[];
      const origin = tiles.find((t) => t.id === id);
      if (!origin) throw new Error("Unknown historical ritual location");
      const distance = (t: Record<string, unknown>) => {
        const r = Number(t.row) - Number(origin.row);
        const q =
          Number(t.col) -
          Math.floor(Number(t.row) / 2) -
          Number(origin.col) +
          Math.floor(Number(origin.row) / 2);
        return (Math.abs(q) + Math.abs(r) + Math.abs(q + r)) / 2;
      };
      return tiles
        .map((t, index) => ({ t, index, d: distance(t) }))
        .filter((v) => v.d <= ritual.radius)
        .sort((a, b) => a.d - b.d || a.index - b.index)
        .map((v) => String(v.t.id));
    };
    for (const marker of marks) {
      if (marker.startsWith(`charge:${ritual.id}:`))
        mapMarks.push({
          id: sequence++,
          type: "captive-death",
          tile: marker.split(":").slice(3).join(":"),
          light: ritual.light,
        });
      if (marker.startsWith(`ritual:${ritual.id}:`)) {
        const members = ritual.units
          .filter((u) => !marks.has(`spawn:${marker}:${u.oldId}`))
          .map(member);
        if (members.length)
          pending.push({
            area: area(source(marker)),
            faction: "enemy",
            aiGroup: ritual.aiGroup,
            units: members,
          });
      }
    }
    for (const [index, id] of ["first-toll", "second-toll", "third-toll"].entries()) {
      const tile = (old.tiles as Record<string, unknown>[]).find(
        (t) => t.id === ritual.sources[index],
      );
      if (tile?.terrain === "ritual-spent") {
        active.delete(id);
        flags.add(id);
      }
    }
  }
  const condition = (input: unknown): unknown => {
    if (Array.isArray(input)) return input.map(condition);
    if (!isRecord(input)) return input;
    if (input.kind === "event") return { kind: "flag", flag: input.event };
    return Object.fromEntries(Object.entries(input).map(([k, v]) => [k, condition(v)]));
  };
  return {
    ...value,
    version: 28,
    checkpoint: {
      ...value.checkpoint,
      state: {
        ...old,
        units,
        tiles: (old.tiles as unknown[]).map(condition),
        factions: (old.factions as Record<string, unknown>[]).map((f) => {
          if (!recipe.removedTransfers.includes(String(f.id))) return condition(f);
          const { transfer: _removed, ...rest } = f;
          return condition(rest);
        }),
        campaign: {
          ...campaign,
          fallenCompanions: [
            ...new Set([
              ...(Array.isArray(campaign.fallenCompanions) ? campaign.fallenCompanions : []),
              ...[...marks].filter((m) => m.startsWith("fallen:")).map((m) => m.slice(7)),
            ]),
          ],
          marks: [],
          holds: {},
          evacuated: [],
          choices: [...flags],
          scripts: {
            active: [...active],
            holds,
            marks: mapMarks,
            pending,
            nextMark: mapMarks.length + 1,
            recruitment,
          },
        },
      },
    },
  };
}
