const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;
const safeKey = (key: string) => !["__proto__", "constructor", "prototype"].includes(key);
export function validUndoStep(value: unknown): boolean {
  if (
    !record(value) ||
    typeof value.unitId !== "string" ||
    !Number.isSafeInteger(value.round) ||
    !Array.isArray(value.changes)
  )
    return false;
  return value.changes.every(
    (change: unknown) =>
      record(change) &&
      ["set", "remove"].includes(String(change.op)) &&
      Array.isArray(change.path) &&
      change.path.length > 0 &&
      change.path.every((part: unknown) => typeof part === "string" && safeKey(part)) &&
      (change.op !== "set" || Object.hasOwn(change, "value")),
  );
}
