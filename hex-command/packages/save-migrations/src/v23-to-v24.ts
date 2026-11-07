import type { Save } from "./formats/history";
import { readSave, readFragment } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import defaults from "./legacy/data/base-factions-v23.json";

/** Make formerly implicit phase armies explicit without replacing played state. */
export function migrateV23ToV24(input: unknown): Save<24> {
  const value = readSave(input, 23);
  if (value.version !== 23 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-three save");
  const old = value.checkpoint.state;
  if (!Array.isArray(old.factions) || !old.factions.every(isRecord))
    throw new Error("Invalid historical factions");
  const factions = [...old.factions];
  for (const faction of defaults) {
    if (!factions.some((entry) => entry.id === faction.id))
      factions.push(readFragment([faction], "factions24")[0]!);
  }
  return {
    ...value,
    version: 24,
    checkpoint: { ...value.checkpoint, state: { ...old, factions } },
  };
}
