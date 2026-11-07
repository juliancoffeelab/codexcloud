/** Frozen data-only backward differences; no application or gameplay dependencies. */
import type { UndoChange } from "../formats/native-v39";
const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;
const safeKey = (key: string) => !["__proto__", "constructor", "prototype"].includes(key);

/** Backward data differences, with unchanged terrain/rosters shared instead of serialized again. */
export function stateChanges(
  current: unknown,
  previous: unknown,
  path: string[] = [],
): UndoChange[] {
  if (current === previous) return [];
  if (
    record(current) &&
    record(previous) &&
    Array.isArray(current) === Array.isArray(previous) &&
    (!Array.isArray(current) || current.length === (previous as unknown as unknown[]).length)
  ) {
    const changes: UndoChange[] = [];
    for (const key of Object.keys(current))
      if (
        current[key] !== undefined &&
        (!Object.hasOwn(previous, key) || previous[key] === undefined)
      )
        changes.push({ op: "remove", path: [...path, key] });
    for (const key of Object.keys(previous)) {
      if (previous[key] === undefined) continue;
      changes.push(...stateChanges(current[key], previous[key], [...path, key]));
    }
    return changes;
  }
  return [{ op: "set", path, value: previous }];
}

export function applyAt(value: unknown, change: UndoChange, depth: number): unknown {
  const key = change.path[depth];
  if (!record(value) || key === undefined || !safeKey(key)) throw new Error("Invalid undo path");
  const array = Array.isArray(value);
  if (array && (!/^(0|[1-9][0-9]*)$/.test(key) || Number(key) >= value.length))
    throw new Error("Invalid undo index");
  const copy: Record<string, unknown> = array
    ? ([...value] as unknown as Record<string, unknown>)
    : { ...value };
  if (depth === change.path.length - 1) {
    if (change.op === "set") copy[key] = change.value;
    else {
      if (array || !Object.hasOwn(copy, key)) throw new Error("Invalid undo removal");
      delete copy[key];
    }
  } else {
    if (!Object.hasOwn(value, key)) throw new Error("Missing undo parent");
    copy[key] = applyAt(value[key], change, depth + 1);
  }
  return copy;
}

/** Apply old data patches, then convert complete states through one semantic reader. */
export function convertStateAndUndo<T>(
  current: unknown,
  undo: import("../formats/native-v39").UndoStep[] | undefined,
  convert: (value: unknown) => T,
): { state: T; undo?: import("../formats/native-v39").UndoStep[] } {
  let previous = current;
  let converted = convert(previous);
  const state = converted;
  if (undo === undefined) return { state };
  const history = [...undo]
    .reverse()
    .map((step) => {
      for (const change of step.changes) previous = applyAt(previous, change, 0);
      const next = convert(previous);
      const changes = stateChanges(converted, next);
      converted = next;
      return { ...step, changes };
    })
    .reverse();
  return { state, undo: history };
}
