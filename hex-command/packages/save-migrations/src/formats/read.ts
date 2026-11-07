import type { Save, Version, History, Fragments, NativeSave1, JavascriptSave1 } from "./history";
import document from "./readers.json";
interface Shape {
  kind: string;
  value?: string | number | boolean;
  members?: number[];
  element?: number;
  items?: number[];
  properties?: Record<string, { type: number; optional: boolean }>;
  index?: number;
}
const shapes = document.shapes as Shape[];
function matches(value: unknown, index: number): boolean {
  const s = shapes[index];
  if (!s) throw new Error("Invalid frozen reader");
  switch (s.kind) {
    case "unknown":
      return true;
    case "never":
      return false;
    case "undefined":
      return value === undefined;
    case "null":
      return value === null;
    case "literal":
      return value === s.value;
    case "string":
      return typeof value === "string";
    case "number":
      return typeof value === "number" && Number.isFinite(value);
    case "boolean":
      return typeof value === "boolean";
    case "union":
      return s.members!.some((member) => matches(value, member));
    case "tuple":
      return (
        Array.isArray(value) &&
        value.length === s.items!.length &&
        s.items!.every((item, i) => matches(value[i], item))
      );
    case "array":
      return Array.isArray(value) && value.every((v) => matches(v, s.element!));
    case "object": {
      if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
      const record = value as Record<string, unknown>;
      if (
        !Object.entries(s.properties!).every(([key, p]) =>
          Object.hasOwn(record, key) ? matches(record[key], p.type) : p.optional,
        )
      )
        return false;
      return (
        s.index === undefined || Object.keys(record).every((key) => matches(record[key], s.index!))
      );
    }
    default:
      throw new Error("Unknown frozen reader kind");
  }
}
function read(value: unknown, name: keyof typeof document.entries): unknown {
  if (!matches(value, document.entries[name])) throw new Error(`Invalid historical ${name}`);
  return value;
}
/** This assertion follows validation of the complete generated historical graph. */
export function readSave<V extends Version>(value: unknown, version: V): Save<V> {
  return read(value, `Save${version}`) as Save<V>;
}
export function readHistory<V extends 1 | 2 | 3 | 4>(value: unknown, version: V): History<V> {
  return read(value, `History${version}`) as History<V>;
}

export function readFragment<K extends keyof Fragments>(value: unknown, key: K): Fragments[K] {
  const type = shapes[document.entries.Fragments]!.properties![key]!.type;
  if (!matches(value, type)) throw new Error(`Invalid frozen fragment ${key}`);
  return value as Fragments[K];
}

export function readVersionOne(value: unknown, kind: "native"): NativeSave1;
export function readVersionOne(value: unknown, kind: "javascript"): JavascriptSave1;
export function readVersionOne(
  value: unknown,
  kind: "native" | "javascript",
): NativeSave1 | JavascriptSave1 {
  return read(value, kind === "native" ? "Native1" : "Javascript1") as
    NativeSave1 | JavascriptSave1;
}
