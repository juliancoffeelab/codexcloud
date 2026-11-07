import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Version seven had inventory, but no items on the battlefield. */
export function migrateV7ToV8(input: unknown): Save<8> {
  const value = readSave(input, 7);
  if (value.version !== 7 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-seven save");
  return {
    ...value,
    version: 8,
    checkpoint: { ...value.checkpoint, state: { ...value.checkpoint.state, groundItems: [] } },
  };
}
