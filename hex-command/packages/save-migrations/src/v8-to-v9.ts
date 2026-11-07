import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import historical from "./legacy/data/toll-road-v8.json";

/** Preserve resolved play while replacing the old preplaced village encounter. */
export function migrateV8ToV9(input: unknown): Save<9> {
  const value = readSave(input, 8);
  if (value.version !== 8 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-eight save");
  const old = value.checkpoint.state;
  if (old.mapId !== historical.mapId || old.mapRevision !== historical.fromRevision)
    return { ...value, version: 9 };
  if (
    !Array.isArray(old.units) ||
    !old.units.every(isRecord) ||
    !Array.isArray(old.tiles) ||
    !old.tiles.every(isRecord) ||
    !Array.isArray(old.factions) ||
    !old.factions.every(isRecord) ||
    !isRecord(old.campaign) ||
    !Array.isArray(old.campaign.marks)
  )
    throw new Error("Invalid historical Toll Road checkpoint");
  const marks = [...old.campaign.marks];
  const met = marks.includes("event:village-contact");
  const memory = Array.isArray(old.conditionMemory) ? [...old.conditionMemory] : [];
  const add = (mark: string) => {
    if (!marks.includes(mark)) marks.push(mark);
  };
  if (
    met ||
    marks.includes(`fallen:${historical.sellaName}`) ||
    (Array.isArray(old.campaign.fallenCompanions) &&
      old.campaign.fallenCompanions.includes(historical.sellaName))
  )
    add(historical.sellaSpawn);
  if (met) {
    if (old.tiles.some((tile) => tile.id === historical.camp && tile.owner === "player"))
      add("event:foothold-established");
  }
  if (memory.includes(historical.swampMemory)) add("event:swamp-sighting");
  else {
    const index = marks.indexOf("event:swamp-sighting");
    if (index >= 0) marks.splice(index, 1);
  }
  const units = old.units
    .filter((unit) => met || unit.name !== historical.sellaName)
    .map((unit) => {
      if (unit.name === historical.sellaName && unit.faction === historical.oldSellaFaction)
        return { ...unit, faction: "player" };
      if (unit.id === historical.casterId && unit.faction === historical.casterFaction)
        return { ...unit, name: historical.casterName };
      return { ...unit };
    });
  const factions = old.factions
    .filter((faction) => faction.id !== historical.oldSellaFaction)
    .map((faction) => {
      if (!isRecord(faction.relations)) return { ...faction };
      const relations = { ...faction.relations };
      delete relations[historical.oldSellaFaction];
      return { ...faction, relations };
    });
  return {
    ...value,
    version: 9,
    checkpoint: {
      ...value.checkpoint,
      state: {
        ...old,
        mapRevision: historical.toRevision,
        units,
        factions,
        conditionMemory: memory.filter((key) => key !== historical.oldLightMemory),
        campaign: { ...old.campaign, marks },
      },
    },
  };
}
