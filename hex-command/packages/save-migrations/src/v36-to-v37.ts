import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
/** Collapse the old manual snapshot variant into snapshot-only turn history. */
export function migrateV36ToV37(input: unknown): Save<37> {
  const value = readSave(input, 36);
  if (value.version !== 36 || !isRecord(value.progress) || !isRecord(value.progress.entry))
    throw new Error("Invalid version-thirty-six save");
  const old = value.progress.entry;
  if (old.kind === "recovery")
    return {
      ...value,
      version: 37,
      progress: { ...value.progress, entry: { ...old, kind: "flash" } },
    };
  if (old.kind !== "manual" && old.kind !== "turn")
    throw new Error("Invalid historical checkpoint kind");
  const { undo: _undo, ...rest } = value;
  return {
    ...rest,
    version: 37,
    progress: { entry: { ...old, kind: "turn", manual: old.kind === "manual" } },
  };
}
