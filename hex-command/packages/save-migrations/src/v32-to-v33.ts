import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
/** Version 33 distinguishes an adopted saved position from a historical turn start. */
export function migrateV32ToV33(input: unknown): Save<33> {
  const value = readSave(input, 32);
  return { ...value, version: 33 };
}
