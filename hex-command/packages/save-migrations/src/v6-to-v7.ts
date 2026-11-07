import type { Save } from "./formats/history";
import { readSave, readFragment } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import historical from "./legacy/data/factions-v6.json";

/** Convert the old implicit animal army into explicit, frozen faction data. */
export function migrateV6ToV7(input: unknown): Save<7> {
  const value = readSave(input, 6);
  const checkpoint = value.checkpoint;
  if (
    !isRecord(checkpoint) ||
    !isRecord(checkpoint.state) ||
    !Array.isArray(checkpoint.state.units)
  )
    throw new Error("Invalid version-six native save");
  const state = checkpoint.state;
  if (!Array.isArray(state.units)) throw new Error("Invalid version-six units");
  if (state.factions !== undefined && !Array.isArray(state.factions))
    throw new Error("Invalid version-six factions");
  const conversion = historical.maps.find(
    (map) => map.mapId === state.mapId && map.mapRevision === state.mapRevision,
  );
  const factions: import("./formats/history").Faction<7>[] = structuredClone(state.factions ?? []);
  for (const faction of conversion?.factions ?? []) {
    if (!factions.some((existing) => isRecord(existing) && existing.id === faction.id))
      factions.push(readFragment([faction], "factions7")[0]!);
  }
  const units = state.units.map((unit) => {
    if (
      conversion &&
      isRecord(unit) &&
      unit.side === "enemy" &&
      unit.faction === undefined &&
      ((typeof unit.aiGroup === "string" && conversion.wildlifeGroups.includes(unit.aiGroup)) ||
        (typeof unit.type === "string" && conversion.wildlifeTypes.includes(unit.type)))
    )
      return { ...unit, faction: "wildlife" };
    return unit;
  });
  return {
    ...value,
    version: 7,
    checkpoint: { ...checkpoint, state: { ...state, factions, units } },
  };
}
