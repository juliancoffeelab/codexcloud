import { isRecord } from "../schema";
/** Collapse the old manual snapshot variant into snapshot-only turn history. */
export function migrateV36ToV37(value: Record<string, unknown>): Record<string, unknown> {
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
