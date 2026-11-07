import type { State, UndoChange, UndoStep } from "../../rust/pkg/hex_command";
import type { SavedUndoEntry } from "../game/undo";
import type { GameState } from "../game/types";
import { immutable, type Immutable } from "../game/immutable";

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

function applyAt(value: unknown, change: UndoChange, depth: number): unknown {
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

/** Structural sharing keeps decoding proportional to changed paths, rather than world size per move. */
export function decodeUndo(state: Immutable<State>, history: UndoStep[]): SavedUndoEntry[] {
  if (!history.length) return [];
  let current: unknown = immutable(state);
  const entries: SavedUndoEntry[] = [];
  for (const step of [...history].reverse()) {
    for (const change of step.changes) current = applyAt(current, change, 0);
    entries.push({
      changes: step.changes,
      round: step.round,
      unitId: step.unitId,
      state: immutable(current) as Immutable<GameState>,
    });
  }
  return entries.reverse();
}

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
