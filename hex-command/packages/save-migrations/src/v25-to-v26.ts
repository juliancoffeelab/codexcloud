import type { Save, State, Condition, Rule, Unit, Veteran } from "./formats/history";
import { readSave } from "./formats/read";
import frozen from "./legacy/data/characters-v25.json";

/** Replace display-name references with campaign-scoped character identities. */
export function migrateV25ToV26(input: unknown): Save<26> {
  const value = readSave(input, 25),
    old = value.checkpoint.state;
  const campaignId = old.campaign?.id ?? frozen.maps[old.mapId as keyof typeof frozen.maps];
  const registry = frozen.campaigns[campaignId as keyof typeof frozen.campaigns];
  function id(name: string): string {
    if (!registry) throw new Error("Unknown historical character");
    const aliases: Record<string, string> = registry.aliases;
    if (aliases[name]) return aliases[name]!;
    if (registry.characters.some((c) => c.id === name)) return name;
    throw new Error(`Unknown historical character ${name}`);
  }
  type WithoutName<T> = T extends unknown ? Omit<T, "name"> : never;
  function named<T extends { name?: string; character?: string }>(unit: T): Omit<T, "name"> {
    const { name, ...rest } = unit;
    const character =
      unit.character !== undefined ? id(unit.character) : name !== undefined ? id(name) : undefined;
    return { ...rest, ...(character ? { character } : {}) };
  }
  function condition(c: Condition<25>): Condition<26> {
    if (c.kind === "all" || c.kind === "any")
      return { ...c, conditions: c.conditions.map(condition) };
    if (c.kind === "hasUnits") return named(c);
    if (c.kind === "noUnits") return named(c);
    if (c.kind === "proximity") return named(c);
    return c;
  }
  const rule = (r: Rule<25>): Rule<26> => ({ ...r, condition: condition(r.condition) });
  const state: State<26> = {
    ...old,
    units: old.units.map((u) => named(u)),
    tiles: old.tiles.map((t) => ({
      ...t,
      ...(t.fog ? { fog: rule(t.fog) } : {}),
      ...(t.light?.when ? { light: { ...t.light, when: rule(t.light.when) } } : {}),
    })),
    factions: old.factions.map((f) => ({
      ...f,
      ...(f.transfer ? { transfer: { ...f.transfer, when: rule(f.transfer.when) } } : {}),
    })),
    ...(old.campaign
      ? {
          campaign: {
            ...old.campaign,
            ...(old.campaign.evacuated
              ? { evacuated: old.campaign.evacuated.map((e) => ({ ...e, unit: named(e.unit) })) }
              : {}),
            ...(old.campaign.veterans
              ? { veterans: old.campaign.veterans.map((v) => named(v)) }
              : {}),
            ...(old.campaign.storyVeterans
              ? { storyVeterans: old.campaign.storyVeterans.map((v) => named(v)) }
              : {}),
            ...(old.campaign.fallenCompanions
              ? { fallenCompanions: old.campaign.fallenCompanions.map(id) }
              : {}),
            marks: old.campaign.marks.map((m) =>
              m.startsWith("fallen:") ? `fallen:${id(m.slice(7))}` : m,
            ),
            dialogue: old.campaign.dialogue.map((line) => {
              const character = line.character
                ? id(line.character)
                : registry && line.speaker
                  ? (registry.aliases as Record<string, string>)[line.speaker]
                  : undefined;
              return character ? { ...line, character, speaker: "" } : line;
            }),
          },
        }
      : {}),
  };
  return { ...value, version: 26, checkpoint: { ...value.checkpoint, state } };
}
