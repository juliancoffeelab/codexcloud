import type { Save, State, Condition, Rule } from "./formats/history";
import { readSave } from "./formats/read";
import boards from "./legacy/data/tiles-v26.json";
/** Rekey historical references; coordinates remain separate geometric facts. */
export function migrateV26ToV27(input: unknown): Save<27> {
  const value = readSave(input, 26),
    old = value.checkpoint.state;
  const ids: Record<string, string> | undefined = boards[old.mapId as keyof typeof boards];
  if (!ids) throw new Error("Unknown historical board");
  function tile(input: string): string {
    if (!ids![input]) throw new Error("Unknown historical tile");
    return ids![input]!;
  }
  function condition(c: Condition<26>): Condition<27> {
    if (c.kind === "all" || c.kind === "any")
      return { ...c, conditions: c.conditions.map(condition) };
    if ((c.kind === "owner" || c.kind === "proximity") && c.tile !== undefined)
      return { ...c, tile: tile(c.tile) };
    return c;
  }
  const rule = (r: Rule<26>): Rule<27> => ({ ...r, condition: condition(r.condition) });
  const unit = <T extends { tile: string }>(u: T): T => ({ ...u, tile: tile(u.tile) });
  const memory = (text: string) => {
    for (const prefix of ["fog:", "light:"])
      if (text.startsWith(prefix)) return prefix + tile(text.slice(prefix.length));
    return text;
  };
  // Only these legacy marker formats embed geometric references. Event, wave
  // and character identifiers remain opaque even if they contain coordinates.
  const marker = (text: string) => {
    if (text.startsWith("sealed:")) return "sealed:" + tile(text.slice(7));
    if (text.startsWith("charge:"))
      return text.replace(/:(\d+,\d+)$/, (_, coordinate: string) => ":" + tile(coordinate));
    if (text.startsWith("ritual:") || text.startsWith("spawn:ritual:"))
      return text.replace(
        /:(mark-|death-u[1-9]\d*-)(\d+,\d+)(?=:[^:]+$|$)/g,
        (_, source: string, coordinate: string) => ":" + source + tile(coordinate),
      );
    return text;
  };
  const state: State<27> = {
    ...old,
    tiles: old.tiles.map((t) => ({
      ...t,
      id: tile(t.id),
      ...(t.fog ? { fog: rule(t.fog) } : {}),
      ...(t.light?.when ? { light: { ...t.light, when: rule(t.light.when) } } : {}),
    })),
    units: old.units.map(unit),
    groundItems: old.groundItems.map(unit),
    factions: old.factions.map((f) => ({
      ...f,
      ...(f.transfer ? { transfer: { ...f.transfer, when: rule(f.transfer.when) } } : {}),
    })),
    ...(old.conditionMemory ? { conditionMemory: old.conditionMemory.map(memory) } : {}),
    ...(old.campaign
      ? {
          campaign: {
            ...old.campaign,
            marks: old.campaign.marks.map(marker),
            ...(old.campaign.holds
              ? {
                  holds: Object.fromEntries(
                    Object.entries(old.campaign.holds).map(([key, counter]) => [
                      tile(key),
                      counter,
                    ]),
                  ),
                }
              : {}),
            ...(old.campaign.evacuated
              ? { evacuated: old.campaign.evacuated.map((e) => ({ ...e, unit: unit(e.unit) })) }
              : {}),
          },
        }
      : {}),
  };
  return { ...value, version: 27, checkpoint: { ...value.checkpoint, state } };
}
