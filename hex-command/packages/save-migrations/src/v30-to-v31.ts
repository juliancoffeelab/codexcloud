import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
/** Version 31 adds optional local progress identity and move undo. Old saves need neither. */
export function migrateV30ToV31(input: unknown): Save<31> {
  const value = readSave(input, 30);
  return { ...value, version: 31 };
}
