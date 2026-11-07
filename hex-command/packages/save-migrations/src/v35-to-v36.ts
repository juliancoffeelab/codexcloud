import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
/** Version 36 adds explicitly requested manual checkpoints; existing facts are unchanged. */
export function migrateV35ToV36(input: unknown): Save<36> {
  const value = readSave(input, 35);
  if (value.version !== 35) throw new Error("Invalid version-thirty-five save");
  return { ...value, version: 36 };
}
