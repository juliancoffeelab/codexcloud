import { isRecord } from "../schema";
import frozen from "./legacy/data/characters-v25.json";

/** Replace historical display-name references with campaign-scoped character IDs. */
export function migrateV25ToV26(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 25 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-five save");
  const old = value.checkpoint.state;
  const campaignId = isRecord(old.campaign)
    ? old.campaign.id
    : frozen.maps[old.mapId as keyof typeof frozen.maps];
  const registry = frozen.campaigns[campaignId as keyof typeof frozen.campaigns];
  const id = (name: unknown): string => {
    if (typeof name !== "string" || !registry) throw new Error("Unknown historical character");
    const alias = (registry.aliases as Record<string, string>)[name];
    if (alias) return alias;
    if (registry.characters.some((character) => character.id === name)) return name;
    throw new Error(`Unknown historical character ${name}`);
  };
  const unit = (input: unknown): Record<string, unknown> => {
    if (!isRecord(input)) throw new Error("Invalid historical unit");
    const { name, ...rest } = input;
    const character =
      input.character != null ? id(input.character) : name != null ? id(name) : undefined;
    return { ...rest, ...(character ? { character } : {}) };
  };
  const condition = (input: unknown): unknown => {
    if (!isRecord(input)) throw new Error("Invalid historical condition");
    const out = { ...input };
    if (["hasUnits", "noUnits", "proximity"].includes(String(input.kind)) && input.name != null) {
      out.character = id(input.name);
      delete out.name;
    }
    if (Array.isArray(input.conditions)) out.conditions = input.conditions.map(condition);
    return out;
  };
  const rule = (input: unknown): unknown => {
    if (!isRecord(input)) return input;
    return { ...input, condition: condition(input.condition) };
  };
  if (!Array.isArray(old.units) || !Array.isArray(old.tiles) || !Array.isArray(old.factions))
    throw new Error("Invalid historical battle");
  const state: Record<string, unknown> = {
    ...old,
    units: old.units.map(unit),
    tiles: old.tiles.map((input) => {
      if (!isRecord(input)) throw new Error("Invalid historical tile");
      return {
        ...input,
        ...(input.fog ? { fog: rule(input.fog) } : {}),
        ...(isRecord(input.light) && input.light.when
          ? { light: { ...input.light, when: rule(input.light.when) } }
          : {}),
      };
    }),
    factions: old.factions.map((input) => {
      if (!isRecord(input)) throw new Error("Invalid historical faction");
      return {
        ...input,
        ...(isRecord(input.transfer)
          ? { transfer: { ...input.transfer, when: rule(input.transfer.when) } }
          : {}),
      };
    }),
  };
  if (isRecord(old.campaign)) {
    const campaign = old.campaign;
    const names = (input: unknown) => {
      if (!Array.isArray(input)) throw new Error("Invalid historical companions");
      return input.map(id);
    };
    const veterans = (input: unknown) => {
      if (!Array.isArray(input)) throw new Error("Invalid historical veterans");
      return input.map(unit);
    };
    if (!Array.isArray(campaign.dialogue) || !Array.isArray(campaign.marks))
      throw new Error("Invalid historical campaign");
    state.campaign = {
      ...campaign,
      ...(Array.isArray(campaign.evacuated)
        ? {
            evacuated: campaign.evacuated.map((record) => {
              if (!isRecord(record)) throw new Error("Invalid historical evacuation");
              return { ...record, unit: unit(record.unit) };
            }),
          }
        : {}),
      ...(campaign.veterans != null ? { veterans: veterans(campaign.veterans) } : {}),
      ...(campaign.storyVeterans != null
        ? { storyVeterans: veterans(campaign.storyVeterans) }
        : {}),
      ...(campaign.fallenCompanions != null
        ? { fallenCompanions: names(campaign.fallenCompanions) }
        : {}),
      marks: campaign.marks.map((mark) =>
        typeof mark === "string" && mark.startsWith("fallen:")
          ? `fallen:${id(mark.slice(7))}`
          : mark,
      ),
      dialogue: campaign.dialogue.map((line) => {
        if (!isRecord(line)) throw new Error("Invalid historical dialogue");
        const character =
          line.character != null
            ? id(line.character)
            : registry && typeof line.speaker === "string"
              ? (registry.aliases as Record<string, string>)[line.speaker]
              : undefined;
        return character ? { ...line, character, speaker: "" } : line;
      }),
    };
  }
  return { ...value, version: 26, checkpoint: { ...value.checkpoint, state } };
}
