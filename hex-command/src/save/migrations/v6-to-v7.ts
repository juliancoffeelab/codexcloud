import { isRecord, type BattleSave } from "../schema";
import historical from "./legacy/data/factions-v6.json";

/** Convert the old implicit animal army into explicit, frozen faction data. */
export function migrateV6ToV7(value: Record<string, unknown>): BattleSave {
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
  const factions = structuredClone(state.factions ?? []) as unknown[];
  for (const faction of conversion?.factions ?? []) {
    if (!factions.some((existing) => isRecord(existing) && existing.id === faction.id))
      factions.push(structuredClone(faction));
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
  } as unknown as BattleSave;
}
