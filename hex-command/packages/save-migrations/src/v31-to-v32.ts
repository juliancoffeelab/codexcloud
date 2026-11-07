import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
/** Version 32 replaces repeated full-world undo snapshots with compact backward changes. */
export function migrateV31ToV32(input: unknown): Save<32> {
  const value = readSave(input, 31);
  const { undo: _discardedUndo, ...save } = value;
  return { ...save, version: 32 };
}
