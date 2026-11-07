import { isRecord } from "../schema";

function hex(value: string): { col: number; row: number } {
  const match = /^(-?\d+),(-?\d+)$/.exec(value);
  if (!match) throw new Error("Invalid historical hex address");
  const col = Number(match[1]),
    row = Number(match[2]);
  if (!Number.isSafeInteger(col) || !Number.isSafeInteger(row))
    throw new Error("Invalid historical hex coordinates");
  return { col, row };
}

/** Interpret only version-38 address fields. Text and presentation keys remain text. */
export function migrateV38ToV39(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 38 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirty-eight save");
  function convert(input: unknown, path: string[]): unknown {
    if (typeof input === "string" && path.at(-1) === "tile") {
      if (path.includes("fog") || path.includes("light")) {
        return /^-?\d+,-?\d+$/.test(input) ? { hex: hex(input) } : { location: input };
      }
      return hex(input);
    }
    if (Array.isArray(input)) {
      if (path.at(-1) === "area")
        return input.map((entry) => {
          if (typeof entry !== "string") throw new Error("Invalid historical spawn area");
          return hex(entry);
        });
      return input.map((entry, index) => convert(entry, [...path, String(index)]));
    }
    if (!isRecord(input)) return input;
    return Object.fromEntries(
      Object.entries(input).map(([key, entry]) => [key, convert(entry, [...path, key])]),
    );
  }
  const checkpoint = { ...value.checkpoint, state: convert(value.checkpoint.state, []) };
  const result: Record<string, unknown> = { ...value, version: 39, checkpoint };
  if (value.undo !== undefined) {
    if (!Array.isArray(value.undo)) throw new Error("Invalid historical undo");
    result.undo = value.undo.map((step) => {
      if (!isRecord(step) || !Array.isArray(step.changes))
        throw new Error("Invalid historical undo step");
      return {
        ...step,
        changes: step.changes.map((change) => {
          if (
            !isRecord(change) ||
            !Array.isArray(change.path) ||
            !change.path.every((key) => typeof key === "string")
          )
            throw new Error("Invalid historical undo change");
          return change.op === "set"
            ? { ...change, value: convert(change.value, change.path) }
            : { ...change };
        }),
      };
    });
  }
  return result;
}
